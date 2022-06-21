use std::time::{Duration, Instant};

use anyhow::Context;
use hyper::{Client, Uri};
use hyper_tls::HttpsConnector;
use thiserror::Error;
use tokio::sync::mpsc;
use url::Url;

use super::{metrics, plan, report, Config, Report, Signaller};

pub struct Profiler {
    config: Config,
}

#[derive(Error, Debug)]
#[non_exhaustive]
pub enum Error {
    #[error("Received unexpected HTTP status {status}")]
    HttpResponse { status: u16, report: Report },

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
        let target_urls = self.config.targets.clone();
        let target_uris: Vec<Uri> = target_urls
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
        let plan = plan::Builder::new().segments(&self.config.segments).build();
        let mut signaller = Signaller::start(self.config.signaller_kind, plan.clone());

        tokio::spawn(async move {
            let https = HttpsConnector::new();
            let client = Client::builder().build::<_, hyper::Body>(https);
            let mut target_idx = 0;

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
                let target_uri = target_uris[target_idx].clone();
                let target_url = target_urls[target_idx].clone();
                target_idx = (target_idx + 1) % target_uris.len();

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

                    let status = resp
                        .map(|r| r.status().as_u16())
                        .map_err(|e| Error::Unexpected(e.into()));

                    let sample = Sample {
                        target: target_url,
                        due: sig.due,
                        sent,
                        done,
                        status,
                    };

                    tx.send(sample).await?;

                    Result::<(), anyhow::Error>::Ok(())
                });
            }
        });

        self.build_report(rx).await
    }

    async fn drain_receiver(mut rx: mpsc::Receiver<Sample>) {
        rx.close();
        while (rx.recv().await).is_some() {}
    }

    async fn build_report(&self, mut rx: mpsc::Receiver<Sample>) -> Result<Report, Error> {
        let mut report_builder = report::Builder::new();

        let mut backend = metrics::Backend {};
        while let Some(sample) = rx.recv().await {
            backend.record(&sample).await?;
            report_builder.record(&sample)?;

            if self.config.stop_on_client_error {
                if let Err(err) = sample.status {
                    Self::drain_receiver(rx).await;
                    return Err(Error::HttpRequest {
                        source: err.into(),
                        report: report_builder.build(),
                    });
                }
            }

            if self.config.stop_on_non_2xx {
                if let Ok(status) = sample.status && !(200..300).contains(&status) {
                    Self::drain_receiver(rx).await;
                    return Err(Error::HttpResponse {
                        status,
                        report: report_builder.build(),
                    });
                }
            }
        }

        Ok(report_builder.build())
    }
}

#[derive(Debug)]
pub struct Sample {
    pub target: Url,
    pub due: Instant,
    pub sent: Instant,
    pub done: Instant,
    pub status: Result<u16, Error>,
}

impl Sample {
    pub fn actual_latency(&self) -> Duration {
        self.done - self.sent
    }

    pub fn corrected_latency(&self) -> Duration {
        self.done - self.due
    }

    pub fn client_latency(&self) -> Duration {
        self.sent - self.due
    }
}
