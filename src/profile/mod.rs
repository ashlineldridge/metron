mod client;
mod config;
mod plan;
mod report;
mod signaller;

use std::time::{Duration, Instant};

use anyhow::{anyhow, Result};
use hyper::Client;
use hyper::Uri;
use hyper_tls::HttpsConnector;

pub use self::client::ClientResult;
pub use self::config::Config;
pub use self::plan::Plan;
pub use self::plan::RateBlock;
use self::report::Report;
pub use self::signaller::Kind as SignallerKind;
pub use self::signaller::Signal;
use self::signaller::Signaller;
use crate::runtime;

pub fn run(config: &Config) -> Result<Report> {
    let runtime = runtime::build(&config.runtime)?;
    let _guard = runtime.enter();

    let uris: Vec<Uri> = config
        .targets
        .iter()
        .map(|uri| uri.to_string().parse::<hyper::Uri>().unwrap())
        .collect();

    let stop_on_error = config.stop_on_error;
    let stop_on_non_2xx = config.stop_on_non_2xx;

    let (tx, mut rx) = tokio::sync::mpsc::channel(1024);
    let plan = plan::Builder::new().blocks(&config.blocks).build()?;
    let mut signaller = Signaller::new(config.signaller_kind, plan.clone());
    signaller.start();

    let handle = tokio::spawn(async move {
        let https = HttpsConnector::new();
        let client = Client::builder().build::<_, hyper::Body>(https);
        let mut uri_idx = 0;

        let start = Instant::now();
        let stop = plan.calculate_duration().map(|d| start + d);

        while let Some(sig) = signaller.recv().await {
            if let Some(stop) = stop && Instant::now() >= stop {
                break;
            }

            let client = client.clone();
            let tx = tx.clone();

            // Round-robin through the target URIs.
            let target_uri = uris[uri_idx].clone();
            uri_idx = (uri_idx + 1) % uris.len();

            tokio::spawn(async move {
                let sent = Instant::now();

                // TODO: Use a real request
                // let req = hyper::Request::builder()
                //     .method(hyper::Method::POST)
                //     .uri("http://httpbin.org/post")
                //     .body(hyper::Body::from("Hallo!"))
                //     .expect("request builder");
                // let resp = client.request(req).await;

                let resp = client.get(target_uri.clone()).await;
                let done = Instant::now();

                let status = resp
                    .as_ref()
                    .map(|r| r.status().as_u16())
                    .map_err(|e| e.to_string());

                let result = ClientResult {
                    due: sig.due,
                    sent,
                    done,
                    status,
                };

                tx.send(result).await?;

                Result::<(), anyhow::Error>::Ok(())
            });
        }

        Result::<Duration, anyhow::Error>::Ok(Instant::now() - start)
    });

    let (
        total_requests,
        total_200s,
        total_non200s,
        total_errors,
        total_duration,
        avg_actual_latency,
        avg_corrected_latency,
    ) = runtime.block_on(async move {
        let mut total_requests = 0;
        let mut total_200s = 0;
        let mut total_non200s = 0;
        let mut total_errors = 0;
        let mut total_actual_latency = Duration::from_secs(0);
        let mut total_corrected_latency = Duration::from_secs(0);

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
                    if stop_on_non_2xx {
                        return Err(anyhow!("Received status: {}", status));
                    }
                }
                Err(err) => {
                    total_errors += 1;
                    if stop_on_error {
                        return Err(anyhow!("Received error: {}", err));
                    }
                }
            }
        }

        let total_duration = handle.await??;

        Result::<_, anyhow::Error>::Ok((
            total_requests as usize,
            total_200s,
            total_non200s,
            total_errors,
            total_duration,
            total_actual_latency / total_requests,
            total_corrected_latency / total_requests,
        ))
    })?;

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
