use std::time::{Duration, Instant};

use anyhow::{anyhow, Result};
use hyper::Client;
use hyper::Uri;
use hyper_tls::HttpsConnector;
use tokio::sync::mpsc;

use super::plan;
use super::Config;
use super::Report;
use super::Signaller;

pub struct Profiler {
    config: Config,
}

impl Profiler {
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    pub async fn run(&self) -> Result<Report> {
        let uris: Vec<Uri> = self
            .config
            .targets
            .iter()
            .map(|uri| uri.to_string().parse::<hyper::Uri>().unwrap())
            .collect();

        let http_method: hyper::Method = self.config.http_method.parse()?;
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
                // Stop receiving timing signals if we've hit the time limit.
                if let Some(stop_at) = stop_at && Instant::now() >= stop_at {
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

                tokio::spawn(async move {
                    let req = hyper::Request::builder()
                        .method(http_method)
                        .uri(target_uri)
                        .body(hyper::Body::from(payload))?;

                    let sent = Instant::now();
                    let resp = client.request(req).await;
                    let done = Instant::now();

                    let status = resp
                        .as_ref()
                        .map(|r| r.status().as_u16())
                        .map_err(|e| e.to_string());

                    let sample = Sample::new(sig.due, sent, done, status);
                    tx.send(sample).await?;

                    Result::<(), anyhow::Error>::Ok(())
                });
            }
        });

        self.build_report(rx).await
    }

    async fn build_report(&self, mut rx: mpsc::Receiver<Sample>) -> Result<Report> {
        let mut total_requests = 0;
        let mut total_200s = 0;
        let mut total_non200s = 0;
        let mut total_errors = 0;
        let mut total_actual_latency = Duration::from_secs(0);
        let mut total_corrected_latency = Duration::from_secs(0);

        let start = Instant::now();

        while let Some(r) = rx.recv().await {
            total_actual_latency += r.actual_latency();
            total_corrected_latency += r.corrected_latency();
            total_requests += 1;
            match r.status {
                Ok(status) if (200..300).contains(&status) => {
                    total_200s += 1;
                }
                Ok(status) => {
                    total_non200s += 1;
                    if self.config.stop_on_non_2xx {
                        return Err(anyhow!("Received status: {}", status));
                    }
                }
                Err(err) => {
                    total_errors += 1;
                    if self.config.stop_on_error {
                        return Err(anyhow!("Received error: {}", err));
                    }
                }
            }
        }

        let total_duration = Instant::now() - start;
        let avg_actual_latency = total_actual_latency / total_requests as u32;
        let avg_corrected_latency = total_corrected_latency / total_requests as u32;

        Ok(Report {
            total_200s,
            total_non200s,
            total_requests,
            total_errors,
            total_duration,
            avg_actual_latency,
            avg_corrected_latency,
        })
    }
}

pub type StatusCode = u16;
pub type Status = std::result::Result<StatusCode, String>;

#[derive(Debug)]
struct Sample {
    due: Instant,
    sent: Instant,
    done: Instant,
    status: Status,
}

impl Sample {
    pub fn new(due: Instant, sent: Instant, done: Instant, status: Status) -> Self {
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
