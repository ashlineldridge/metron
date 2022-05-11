use std::time::Duration;

#[derive(Debug)]
pub struct Report {
    pub total_200s: usize,
    pub total_non200s: usize,
    pub total_requests: usize,
    pub total_errors: usize,
    pub total_duration: Duration,
    pub avg_actual_latency: Duration,
    pub avg_corrected_latency: Duration,
}
