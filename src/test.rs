use anyhow::Result;
use hyper::Uri;
use std::time::Duration;
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

async fn run_test(_config: &TestConfig) -> Result<TestResults> {
    todo!()
}
