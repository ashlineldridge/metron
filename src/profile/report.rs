use std::time::{Duration, Instant};

use super::profiler::Sample;

#[derive(Debug, Default)]
pub struct Report {
    pub total_200s: usize,
    pub total_non200s: usize,
    pub total_completed_requests: usize,
    pub total_errors: usize,
    pub total_duration: Duration,
    pub total_actual_latency: Duration,
    pub total_corrected_latency: Duration,
}

impl Report {
    pub fn avg_actual_latency(&self) -> Duration {
        if self.total_completed_requests > 0 {
            self.total_actual_latency / self.total_completed_requests as u32
        } else {
            Duration::default()
        }
    }

    pub fn avg_corrected_latency(&self) -> Duration {
        if self.total_completed_requests > 0 {
            self.total_corrected_latency / self.total_completed_requests as u32
        } else {
            Duration::default()
        }
    }
}

/// Builder used to construct a [Report].
pub struct Builder {
    /// The report under construction.
    report: Report,
    /// When we started building the report.
    start: Instant,
}

impl Builder {
    pub fn new() -> Self {
        Self {
            report: Report::default(),
            start: Instant::now(),
        }
    }

    pub fn record(mut self, sample: &Result<Sample, hyper::Error>) -> Builder {
        match sample {
            Ok(sample) => {
                self.report.total_completed_requests += 1;
                if sample.status.is_2xx() {
                    self.report.total_200s += 1;
                } else {
                    self.report.total_non200s += 1;
                }

                self.report.total_actual_latency += sample.actual_latency();
                self.report.total_corrected_latency += sample.corrected_latency();
            }
            Err(_) => {
                self.report.total_errors += 1;
            }
        }

        self
    }

    pub fn build(mut self) -> Report {
        self.report.total_duration = Instant::now() - self.start;
        self.report
    }
}
