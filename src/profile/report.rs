use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

use anyhow::Result;
use serde::Serialize;
use url::Url;

use super::profiler::Sample;

const STANDARD_PERCENTILES: [f64; 6] = [99.9, 99.0, 95.0, 90.0, 75.0, 50.0];

#[derive(Clone, Debug, Serialize)]
pub struct Report {
    pub response_latency: Vec<ReportSection>,
    pub error_latency: Vec<ReportSection>,
    pub request_delay: Vec<ReportSection>,
    pub total_requests: usize,
    #[serde(with = "humantime_serde")]
    pub total_duration: Duration,
}

#[derive(Clone, Debug, Serialize)]
pub struct ReportSection {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<Url>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status_code: Option<u16>,
    pub percentiles: Vec<ReportPercentile>,
    pub total_requests: usize,
}

#[derive(Clone, Debug, Serialize)]
pub struct ReportPercentile {
    pub percentile: f64,
    #[serde(with = "humantime_serde")]
    pub duration: Duration,
}

type Histogram = hdrhistogram::Histogram<u64>;

/// Builder used to construct a [Report].
pub struct Builder {
    /// Whether latency correction is disabled.
    no_latency_correction: bool,

    /// When we started building the report.
    start: Instant,

    /// Response latency histograms keyed by target URL and HTTP status.
    response_histograms: HashMap<(Url, u16), Histogram>,

    /// Error latency histograms keyed by target URL.
    error_histograms: HashMap<Url, Histogram>,

    /// Request delay histograms keyed by target URL. These histograms track the delay period
    /// between when a request should have been sent and when it was sent (i.e., when the delay
    /// increases it means that we cannot keep up with the desired request rate).
    delay_histograms: HashMap<Url, Histogram>,
}

impl Builder {
    pub fn new(no_latency_correction: bool) -> Self {
        Self {
            no_latency_correction,
            start: Instant::now(),
            response_histograms: HashMap::new(),
            error_histograms: HashMap::new(),
            delay_histograms: HashMap::new(),
        }
    }

    pub fn record(&mut self, sample: &Sample) -> Result<()> {
        let hist = if let Ok(status) = sample.status {
            self.response_histograms
                .entry((sample.target.clone(), status))
                .or_insert_with(Self::new_histogram)
        } else {
            self.error_histograms
                .entry(sample.target.clone())
                .or_insert_with(Self::new_histogram)
        };

        let latency = if self.no_latency_correction {
            sample.actual_latency().as_micros().try_into()?
        } else {
            sample.corrected_latency().as_micros().try_into()?
        };

        hist.record(latency)?;

        let delay_histogram = self
            .delay_histograms
            .entry(sample.target.clone())
            .or_insert_with(Self::new_histogram);

        let delay = sample.client_latency().as_micros().try_into()?;
        delay_histogram.record(delay)?;

        Ok(())
    }

    pub fn build(self) -> Report {
        let mut response_latency = vec![];
        for ((url, status), hist) in self.response_histograms {
            response_latency.push(ReportSection {
                target: Some(url.clone()),
                status_code: Some(status),
                percentiles: STANDARD_PERCENTILES
                    .iter()
                    .map(|&p| ReportPercentile {
                        percentile: p,
                        duration: Duration::from_micros(hist.value_at_percentile(p)),
                    })
                    .collect(),
                total_requests: hist.len() as usize,
            });
        }

        let mut error_latency = vec![];
        for (url, hist) in self.error_histograms {
            error_latency.push(ReportSection {
                target: Some(url.clone()),
                status_code: None,
                percentiles: STANDARD_PERCENTILES
                    .iter()
                    .map(|&p| ReportPercentile {
                        percentile: p,
                        duration: Duration::from_micros(hist.value_at_percentile(p)),
                    })
                    .collect(),
                total_requests: hist.len() as usize,
            });
        }

        let mut total_requests = 0;
        let mut request_delay = vec![];
        for (url, hist) in self.delay_histograms {
            request_delay.push(ReportSection {
                target: Some(url.clone()),
                status_code: None,
                percentiles: STANDARD_PERCENTILES
                    .iter()
                    .map(|&p| ReportPercentile {
                        percentile: p,
                        duration: Duration::from_micros(hist.value_at_percentile(p)),
                    })
                    .collect(),
                total_requests: hist.len() as usize,
            });

            total_requests += hist.len() as usize;
        }

        Report {
            response_latency,
            error_latency,
            request_delay,
            total_requests,
            total_duration: self.start.elapsed(),
        }
    }

    fn new_histogram() -> Histogram {
        Histogram::new_with_bounds(1, 30 * 1_000_000, 3).unwrap()
    }
}
