mod client;
mod config;
mod plan;
mod report;
mod signaller;

use std::time::{Duration, Instant};

use anyhow::Result;
use hyper::Client;
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

    let target_uri = config.targets[0].to_string().parse::<hyper::Uri>()?;
    let (tx, mut rx) = tokio::sync::mpsc::channel(1024);
    let plan = plan::Builder::new().blocks(&config.blocks).build()?;
    let mut signaller = Signaller::new(config.signaller_kind, plan);
    signaller.start();

    let sent = Instant::now();

    tokio::spawn(async move {
        let https = HttpsConnector::new();
        let client = Client::builder().build::<_, hyper::Body>(https);

        while let Some(sig) = signaller.recv().await {
            let client = client.clone();
            let tx = tx.clone();
            let target_uri = target_uri.clone();

            tokio::spawn(async move {
                let sent = Instant::now();

                // TODO: Use a real request
                // let req = hyper::Request::builder()
                //     .method(hyper::Method::POST)
                //     .uri("http://httpbin.org/post")
                //     .body(hyper::Body::from("Hallo!"))
                //     .expect("request builder");
                // let resp = client.request(req).await;

                let resp = client.get(target_uri).await;
                let done = Instant::now();
                let status = resp
                    .map(|resp| resp.status().as_u16())
                    .map_err(|e| e.into());

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

        Result::<(), anyhow::Error>::Ok(())
    });

    let (
        total_requests,
        total_200s,
        total_non200s,
        total_errors,
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
                Ok(status) if status != 200 => total_non200s += 1,
                Ok(_) => total_200s += 1,
                Err(e) => {
                    eprintln!("{}", e);
                    total_errors += 1;
                }
            }
        }

        (
            total_requests as usize,
            total_200s,
            total_non200s,
            total_errors,
            total_actual_latency / total_requests,
            total_corrected_latency / total_requests,
        )
    });

    let done = Instant::now();
    let total_duration = done - sent;

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
