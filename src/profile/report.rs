use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

use anyhow::Result;
use url::Url;

use super::profiler::Sample;

#[derive(Clone, Debug, Default)]
pub struct Report {
    pub response_latency: Vec<ReportSection>,
    pub response_latency_corrected: Vec<ReportSection>,
    pub response_latency_combined: Option<ReportSection>,
    pub response_latency_combined_corrected: Option<ReportSection>,
    pub error_latency: Option<ReportSection>,
    pub error_latency_corrected: Option<ReportSection>,
    pub request_delay: Option<ReportSection>,
    pub total_requests: usize,
    pub total_duration: Duration,
}

#[derive(Clone, Debug)]
pub struct ReportSection {
    pub target: Option<Url>,
    pub status_code: Option<u16>,
    pub percentiles: Vec<ReportPercentile>,
    pub total_requests: usize,
}

#[derive(Clone, Debug)]
pub struct ReportPercentile {
    pub percentile: f64,
    pub duration: Duration,
}

type Histogram = hdrhistogram::Histogram<u64>;

/// Builder used to construct a [Report].
pub struct Builder {
    /// When we started building the report.
    _start: Instant,

    /// Response latency histograms keyed by target URL, then by HTTP status. The values are
    /// tuples of `(actual, corrected)` where "actual" measures the duration between when a
    /// request was sent and when the response was received, and "corrected" measures the
    /// duration between when a request _should_ have been sent and when it was received.
    response_histograms: HashMap<Url, HashMap<u16, (Histogram, Histogram)>>,

    /// Error latency histograms keyed by target URL. Values are tuples of `(actual, corrected)`.
    error_histograms: HashMap<Url, (Histogram, Histogram)>,

    /// Request delay histograms keyed by target URL. These histograms track the delay period
    /// between when a request should have been sent and when it was sent (i.e., when the delay
    /// increases it means that we cannot keep up with the desired request rate).
    delay_histograms: HashMap<Url, Histogram>,
}

impl Builder {
    pub fn new() -> Self {
        Self {
            _start: Instant::now(),
            response_histograms: HashMap::new(),
            error_histograms: HashMap::new(),
            delay_histograms: HashMap::new(),
        }
    }

    pub fn record(&mut self, sample: &Sample) -> Result<()> {
        let (actual_histogram, corrected_histogram) = if let Ok(status) = sample.status {
            self.response_histograms
                .entry(sample.target.clone())
                .or_default()
                .entry(status)
                .or_insert_with(|| (Self::new_histogram(), Self::new_histogram()))
        } else {
            self.error_histograms
                .entry(sample.target.clone())
                .or_insert_with(|| (Self::new_histogram(), Self::new_histogram()))
        };

        let actual_latency = sample.actual_latency().as_micros().try_into()?;
        let corrected_latency = sample.corrected_latency().as_micros().try_into()?;
        actual_histogram.record(actual_latency)?;
        corrected_histogram.record(corrected_latency)?;

        let delay_histogram = self
            .delay_histograms
            .entry(sample.target.clone())
            .or_insert_with(Self::new_histogram);

        let delay = sample.client_latency().as_micros().try_into()?;
        delay_histogram.record(delay)?;

        Ok(())
    }

    pub fn build(self) -> Report {
        // let total_duration = Instant::now() - self.start;
        // let report = Report {
        //     response_latency: self.response_histograms,
        //     response_latency_corrected: todo!(),
        //     error_latency: todo!(),
        //     error_latency_corrected: todo!(),
        //     request_delay: todo!(),
        //     total_requests: todo!(),
        //     total_duration,
        // };
        // self.report
        Report::default()
    }

    fn new_histogram() -> Histogram {
        Histogram::new_with_bounds(1, 30 * 1_000_000, 3).unwrap()
    }
}
