use std::time::Duration;

#[derive(Debug)]
pub struct Report {
    pub total_requests: usize,
    pub total_errors: usize,
    pub total_duration: Duration,
    pub avg_latency: Duration,
}
