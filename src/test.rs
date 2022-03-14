use anyhow::Result;
use hyper::Uri;
use std::time::{Duration, Instant};
use tokio::runtime::Builder;

use crate::schedule::{FixedIntervalSchedule, RampedFixedIntervalSchedule};

#[derive(Debug)]
pub struct TestConfig {
    pub connections: usize,
    pub worker_threads: usize,
    pub rate: Option<u32>,
    pub duration: Option<Duration>,
    pub init_rate: Option<u32>,
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

    let start = Instant::now();
    let interval = Duration::from_secs(1) / config.rate.unwrap();
    let use_ramp = config.ramp_duration.is_some() && !config.ramp_duration.unwrap().is_zero();

    let schedule = if use_ramp {
        FixedIntervalSchedule::new(start, interval, config.duration);
    } else {
        let init_interval = Duration::from_secs(1) / config.init_rate;
        RampedFixedIntervalSchedule::new(
            start,
            init_interval,
            interval,
            config.ramp_duration,
            config.duration,
        )
    };

    let results = runtime.block_on(run_test(config))?;

    Ok(results)
}

async fn run_test(_config: &TestConfig) -> Result<TestResults> {
    todo!()
}
