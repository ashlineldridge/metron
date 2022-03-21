use anyhow::{Context, Error, Result};
use hyper::Uri;
use std::time::{Duration, Instant};
use tokio::runtime::Builder;

use wrkr::Rate;

use crate::plan::Plan;
use crate::{client::ClientResult, signaller::Signaller};

#[derive(Debug)]
pub struct TestConfig {
    pub connections: usize,
    pub worker_threads: usize,
    pub async_signaller: bool,
    pub rate: Option<Rate>,
    pub duration: Option<Duration>,
    pub init_rate: Option<Rate>,
    pub ramp_duration: Option<Duration>,
    pub headers: Vec<Header>,
    pub target: Uri,
}

#[derive(Debug)]
pub struct TestResults {
    pub total_requests: usize,
    pub total_errors: usize,
    pub total_duration: Duration,
    pub avg_latency: Duration,
}

#[derive(Debug)]
pub struct Header {
    pub name: String,
    pub value: String,
}

pub fn run(config: &TestConfig) -> Result<TestResults> {
    let runtime = Builder::new_multi_thread()
        .worker_threads(config.worker_threads)
        .enable_all()
        .build()?;

    let _guard = runtime.enter();

    let target_uri = config.target.clone();
    let (tx, mut rx) = tokio::sync::mpsc::channel(1024);
    let mut signaller = create_signaller(config)?;
    signaller.start();

    tokio::spawn(async move {
        let client = hyper::Client::new();

        println!("Created client");

        while let Some(sig) = signaller.recv().await {
            println!("Sig: {:?}", sig.due);

            let client = client.clone();
            let tx = tx.clone();
            let target_uri = target_uri.clone();

            tokio::spawn(async move {
                let sent = Instant::now();
                let resp = client.get(target_uri).await;
                let done = Instant::now();
                let status = resp
                    .map(|resp| resp.status().as_u16())
                    .map_err(|err| err.into());

                let result = ClientResult {
                    due: sig.due,
                    sent,
                    done,
                    status,
                };

                tx.send(result).await?;

                Result::<(), Error>::Ok(())
            });
        }

        Result::<(), Error>::Ok(())
    });

    let (total_responses, total_200s, total_non200s, total_errors) = runtime.block_on(async move {
        let mut total_responses = 0;
        let mut total_200s = 0;
        let mut total_non200s = 0;
        let mut total_errors = 0;

        println!("Reading responses");

        while let Some(r) = rx.recv().await {
            println!("Read response: {:?}", r.corrected_latency());

            total_responses += 1;
            match r.status {
                Ok(status) if status != 200 => total_non200s += 1,
                Ok(_) => total_200s += 1,
                Err(_) => total_errors += 1,
            }
        }

        println!("Done reading");

        (total_responses, total_200s, total_non200s, total_errors)
    });

    dbg!((total_responses, total_200s, total_non200s, total_errors));

    Ok(TestResults {
        total_requests: 0,
        total_errors: 0,
        total_duration: Duration::from_secs(0),
        avg_latency: Duration::from_secs(0),
    })
}

fn create_signaller(config: &TestConfig) -> Result<Box<dyn Signaller>> {
    let schedule = if let Some(ramp_duration) = config.ramp_duration {
        let from = config.init_rate.context("Ramp requires an initial rate")?;
        let to = config.rate.context("Ramp requires main rate")?;
        let mut schedule = crate::schedule::ramped_rate(from, to, ramp_duration).boxed();
        if let Some(duration) = config.duration {
            let duration = ramp_duration + duration;
            schedule = crate::schedule::finite(duration, schedule).boxed();
        };

        Some(schedule)
    } else if let Some(rate) = config.rate {
        let mut schedule = crate::schedule::fixed_rate(rate).boxed();
        if let Some(duration) = config.duration {
            schedule = crate::schedule::finite(duration, schedule).boxed();
        };

        Some(schedule)
    } else {
        None
    };

    let signaller = if let Some(schedule) = schedule {
        if config.async_signaller {
            crate::signaller::async_signaller(schedule).boxed()
        } else {
            crate::signaller::blocking_signaller(schedule).boxed()
        }
    } else {
        crate::signaller::asap_signaller(config.duration).boxed()
    };

    Ok(signaller)
}
