mod client;
mod config;
mod plan;
mod report;
mod signaller;

use anyhow::Result;
use std::time::{Duration, Instant};

use self::report::Report;
use self::signaller::Signaller;

pub use self::client::ClientResult;
pub use self::config::Config;
pub use self::plan::Plan;
pub use self::plan::RateBlock;
pub use self::signaller::Kind as SignallerKind;
pub use self::signaller::Signal;

pub fn run(config: &Config) -> Result<Report> {
    let runtime = if let Some(worker_threads) = config.worker_threads {
        tokio::runtime::Builder::new_multi_thread()
            .worker_threads(worker_threads)
            .enable_all()
            .build()?
    } else {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()?
    };

    let _guard = runtime.enter();

    let target_uri = config.targets[0].to_string().parse::<hyper::Uri>()?;
    let (tx, mut rx) = tokio::sync::mpsc::channel(1024);
    let mut signaller = create_signaller(config)?;
    signaller.start();

    tokio::spawn(async move {
        let client = hyper::Client::new();

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

    let (total_responses, total_200s, total_non200s, total_errors) = runtime.block_on(async move {
        let mut total_responses = 0;
        let mut total_200s = 0;
        let mut total_non200s = 0;
        let mut total_errors = 0;

        while let Some(r) = rx.recv().await {
            println!("Read response: {:?}", r.corrected_latency());

            total_responses += 1;
            match r.status {
                Ok(status) if status != 200 => total_non200s += 1,
                Ok(_) => total_200s += 1,
                Err(_) => total_errors += 1,
            }
        }

        (total_responses, total_200s, total_non200s, total_errors)
    });

    Ok(Report {
        total_requests: 0,
        total_errors: 0,
        total_duration: Duration::from_secs(0),
        avg_latency: Duration::from_secs(0),
    })
}

fn create_signaller(config: &Config) -> Result<Signaller> {
    // let plan = crate::plan::Builder::new()
    //     .ramp(from, to, over)
    //     .duration(duration)
    //     .rate(rate)
    //     .build()?;

    // let schedule = if let Some(ramp_duration) = config.ramp_duration {
    //     let from = config.init_rate.context("Ramp requires an initial rate")?;
    //     let to = config.rate.context("Ramp requires main rate")?;
    //     let mut schedule = crate::schedule::ramped_rate(from, to, ramp_duration).boxed();
    //     if let Some(duration) = config.duration {
    //         let duration = ramp_duration + duration;
    //         schedule = crate::schedule::finite(duration, schedule).boxed();
    //     };

    //     Some(schedule)
    // } else if let Some(rate) = config.rate {
    //     let mut schedule = crate::schedule::fixed_rate(rate).boxed();
    //     if let Some(duration) = config.duration {
    //         schedule = crate::schedule::finite(duration, schedule).boxed();
    //     };

    //     Some(schedule)
    // } else {
    //     None
    // };

    // let signaller = if let Some(schedule) = schedule {
    //     if config.async_signaller {
    //         crate::signaller::async_signaller(schedule).boxed()
    //     } else {
    //         crate::signaller::blocking_signaller(schedule).boxed()
    //     }
    // } else {
    //     crate::signaller::asap_signaller(config.duration).boxed()
    // };

    // Ok(signaller)
    todo!()
}
