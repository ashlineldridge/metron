use std::time::{Duration, Instant};

use anyhow::Result;
use hyper::{Client, Uri};
use tokio::runtime::Builder;

#[derive(Debug)]
pub struct TestConfig {
    pub connections: usize,
    pub threads: usize,
    pub rate: usize,
    pub duration: Duration,
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

pub fn test(config: &TestConfig) -> Result<TestResults> {
    let runtime = Builder::new_multi_thread()
        .worker_threads(config.threads)
        .enable_all()
        .build()?;

    let results = runtime.block_on(run_test(config))?;

    Ok(results)
}

async fn run_test(config: &TestConfig) -> Result<TestResults> {
    let client = Client::new();

    let mut total_requests = 0;
    let mut total_errors = 0;
    let mut total_latency = Duration::from_secs(0);

    let test_start = Instant::now();
    loop {
        let loop_start = Instant::now();

        if test_start.elapsed() >= config.duration {
            break;
        }

        let req_start = Instant::now();
        let resp = client.get(config.target.clone()).await?;
        let req_duration = req_start.elapsed();
        total_latency += req_duration;

        total_requests += 1;

        if !resp.status().is_success() {
            total_errors += 1;
        }

        let sleep_duration =
            (Duration::from_secs(1) / config.rate as u32).saturating_sub(loop_start.elapsed());
        if !sleep_duration.is_zero() {
            tokio::time::sleep(sleep_duration).await;
        }
    }

    Ok(TestResults {
        total_requests,
        total_errors,
        total_duration: test_start.elapsed(),
        avg_latency: total_latency / total_requests as u32,
    })
}
