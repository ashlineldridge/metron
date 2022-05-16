use std::fmt::Display;
use std::time::{Duration, Instant};

use anyhow::Context;
use hyper::Client;
use hyper::Uri;
use hyper_tls::HttpsConnector;
use thiserror::Error;
use tokio::sync::mpsc;

use super::plan;
use super::report;
use super::Config;
use super::Report;
use super::Signaller;

pub struct Profiler {
    config: Config,
}

#[derive(Error, Debug)]
#[non_exhaustive]
pub enum Error {
    #[error("Received unexpected HTTP status {status}")]
    HttpResponse { status: StatusCode, report: Report },

    #[error("Could not perform HTTP request")]
    HttpRequest {
        source: anyhow::Error,
        report: Report,
    },

    #[error(transparent)]
    Unexpected(#[from] anyhow::Error),
}

impl Error {
    pub fn partial_report(&self) -> Option<&Report> {
        match self {
            Error::HttpResponse { report, .. } => Some(report),
            Error::HttpRequest { report, .. } => Some(report),
            _ => None,
        }
    }
}

impl Profiler {
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    pub async fn run(&self) -> Result<Report, Error> {
        let uris: Vec<Uri> = self
            .config
            .targets
            .iter()
            .map(|uri| uri.to_string().parse::<hyper::Uri>().unwrap())
            .collect();

        let http_method: hyper::Method = self
            .config
            .http_method
            .parse()
            .context("Invalid HTTP method")?;

        let payload = self.config.payload.clone().unwrap_or_default();

        let (tx, rx) = mpsc::channel(1024);
        let plan = plan::Builder::new().blocks(&self.config.blocks).build();
        let mut signaller = Signaller::start(self.config.signaller_kind, plan.clone());

        tokio::spawn(async move {
            let https = HttpsConnector::new();
            let client = Client::builder().build::<_, hyper::Body>(https);
            let mut uri_idx = 0;

            let start = Instant::now();
            let stop_at = plan.calculate_duration().map(|d| start + d);

            while let Some(sig) = signaller.recv().await {
                // Quit if we've hit the time limit.
                if let Some(stop_at) = stop_at && Instant::now() >= stop_at {
                    break;
                }

                // Quit if the channel has closed (which implies that the receiver encountered
                // an error condition and hung up).
                if tx.is_closed() {
                    break;
                }

                // Round-robin through the target URIs.
                let target_uri = uris[uri_idx].clone();
                uri_idx = (uri_idx + 1) % uris.len();

                // Clone other items that need to be moved into the spawned task below.
                let client = client.clone();
                let tx = tx.clone();
                let http_method = http_method.clone();
                let payload = payload.clone();

                // Send Result<Sample> down the channel
                tokio::spawn(async move {
                    let req = hyper::Request::builder()
                        .method(http_method)
                        .uri(target_uri)
                        .body(hyper::Body::from(payload))?;

                    let sent = Instant::now();
                    let resp = client.request(req).await;
                    let done = Instant::now();

                    let sample = resp.map(|resp| {
                        let status = StatusCode(resp.status().as_u16());
                        Sample::new(sig.due, sent, done, status)
                    });

                    tx.send(sample).await?;

                    Result::<(), anyhow::Error>::Ok(())
                });
            }
        });

        self.build_report(rx).await
    }

    async fn drain_receiver(mut rx: mpsc::Receiver<Result<Sample, hyper::Error>>) {
        rx.close();
        while (rx.recv().await).is_some() {}
    }

    async fn build_report(
        &self,
        mut rx: mpsc::Receiver<Result<Sample, hyper::Error>>,
    ) -> Result<Report, Error> {
        let mut report_builder = report::Builder::new();

        while let Some(sample) = rx.recv().await {
            report_builder = report_builder.record(&sample);
            match sample {
                Ok(sample) if !sample.status.is_2xx() && self.config.stop_on_non_2xx => {
                    Self::drain_receiver(rx).await;
                    return Err(Error::HttpResponse {
                        status: sample.status,
                        report: report_builder.build(),
                    });
                }
                Err(err) if self.config.stop_on_error => {
                    Self::drain_receiver(rx).await;
                    return Err(Error::HttpRequest {
                        source: err.into(),
                        report: report_builder.build(),
                    });
                }
                _ => (),
            }
        }

        Ok(report_builder.build())
    }
}

#[derive(Debug)]
pub struct StatusCode(u16);

impl Display for StatusCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl StatusCode {
    pub fn is_2xx(&self) -> bool {
        (200..300).contains(&self.0)
    }
}

#[derive(Debug)]
pub struct Sample {
    pub due: Instant,
    pub sent: Instant,
    pub done: Instant,
    pub status: StatusCode,
}

impl Sample {
    pub fn new(due: Instant, sent: Instant, done: Instant, status: StatusCode) -> Self {
        Self {
            due,
            sent,
            done,
            status,
        }
    }

    pub fn actual_latency(&self) -> Duration {
        self.done - self.sent
    }

    pub fn corrected_latency(&self) -> Duration {
        self.done - self.due
    }
}
