use std::{
    collections::HashMap,
    fmt::Display,
    time::{Duration, Instant},
};

use anyhow::Result;
use url::Url;

use super::profiler::Sample;

// TODO: Probably the report should be a simple datastructure
// Then we can have differerent printers/serializers
// And the builder/recorder can hold the histograms

pub struct Report2 {
    pub response_latency: Option<ReportSection>,
    pub response_latency_corrected: Option<ReportSection>,
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

#[derive(Clone, Default)]
pub struct Report {
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

    /// Total duration of profiling operation.
    total_duration: Duration,
}

impl std::fmt::Debug for Report {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl Display for Report {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (target, m) in &self.response_histograms {
            for (status, (actual, corrected)) in m {
                f.write_fmt(format_args!(
                    "
Response Latency
----------------
Target URL:                  {}
Status Code:                 {}
Actual Latency (95%):        {:?}
Actual Latency (99%):        {:?}
Actual Latency (99.9%):      {:?}
Actual Latency (99.99%):     {:?}
Actual Latency (99.999%):    {:?}
Corrected Latency (95%):     {:?}
Corrected Latency (99%):     {:?}
Corrected Latency (99.9%):   {:?}
Corrected Latency (99.99%):  {:?}
Corrected Latency (99.999%): {:?}
Total Requests:              {}\n",
                    target,
                    status,
                    Duration::from_micros(actual.value_at_quantile(0.95)),
                    Duration::from_micros(actual.value_at_quantile(0.99)),
                    Duration::from_micros(actual.value_at_quantile(0.999)),
                    Duration::from_micros(actual.value_at_quantile(0.9999)),
                    Duration::from_micros(actual.value_at_quantile(0.9999)),
                    Duration::from_micros(corrected.value_at_quantile(0.95)),
                    Duration::from_micros(corrected.value_at_quantile(0.99)),
                    Duration::from_micros(corrected.value_at_quantile(0.999)),
                    Duration::from_micros(corrected.value_at_quantile(0.9999)),
                    Duration::from_micros(corrected.value_at_quantile(0.99999)),
                    actual.len(),
                ))?;
            }
        }

        for (target, (actual, corrected)) in &self.error_histograms {
            f.write_fmt(format_args!(
                "
Error Latency
-------------
Target URL:                  {}
Actual Latency (95%):        {:?}
Actual Latency (99%):        {:?}
Actual Latency (99.9%):      {:?}
Actual Latency (99.99%):     {:?}
Actual Latency (99.999%):    {:?}
Corrected Latency (95%):     {:?}
Corrected Latency (99%):     {:?}
Corrected Latency (99.9%):   {:?}
Corrected Latency (99.99%):  {:?}
Corrected Latency (99.999%): {:?}
Total Requests:              {}\n",
                target,
                Duration::from_micros(actual.value_at_quantile(0.95)),
                Duration::from_micros(actual.value_at_quantile(0.99)),
                Duration::from_micros(actual.value_at_quantile(0.999)),
                Duration::from_micros(actual.value_at_quantile(0.9999)),
                Duration::from_micros(actual.value_at_quantile(0.9999)),
                Duration::from_micros(corrected.value_at_quantile(0.95)),
                Duration::from_micros(corrected.value_at_quantile(0.99)),
                Duration::from_micros(corrected.value_at_quantile(0.999)),
                Duration::from_micros(corrected.value_at_quantile(0.9999)),
                Duration::from_micros(corrected.value_at_quantile(0.99999)),
                actual.len(),
            ))?;
        }

        for (target, hist) in &self.delay_histograms {
            f.write_fmt(format_args!(
                "
Request Delay
-------------
Target URL:                  {}
Delay (95%):                 {:?}
Delay (99%):                 {:?}
Delay (99.9%):               {:?}
Delay (99.99%):              {:?}
Delay (99.999%):             {:?}
Total Requests:              {}\n",
                target,
                Duration::from_micros(hist.value_at_quantile(0.95)),
                Duration::from_micros(hist.value_at_quantile(0.99)),
                Duration::from_micros(hist.value_at_quantile(0.999)),
                Duration::from_micros(hist.value_at_quantile(0.9999)),
                Duration::from_micros(hist.value_at_quantile(0.99999)),
                hist.len(),
            ))?;
        }

        f.write_fmt(format_args!("\nTotal duration: {:?}", self.total_duration))?;

        Ok(())
    }
}

/// Builder used to construct a [Report].
pub struct Builder {
    /// Report under construction.
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

    pub fn record(&mut self, sample: &Sample) -> Result<()> {
        let (actual_histogram, corrected_histogram) = if let Ok(status) = sample.status {
            self.report
                .response_histograms
                .entry(sample.target.clone())
                .or_default()
                .entry(status)
                .or_insert_with(|| (Self::new_histogram(), Self::new_histogram()))
        } else {
            self.report
                .error_histograms
                .entry(sample.target.clone())
                .or_insert_with(|| (Self::new_histogram(), Self::new_histogram()))
        };

        let actual_latency = sample.actual_latency().as_micros().try_into()?;
        let corrected_latency = sample.corrected_latency().as_micros().try_into()?;
        actual_histogram.record(actual_latency)?;
        corrected_histogram.record(corrected_latency)?;

        let delay_histogram = self
            .report
            .delay_histograms
            .entry(sample.target.clone())
            .or_insert_with(Self::new_histogram);

        let delay = sample.client_latency().as_micros().try_into()?;
        delay_histogram.record(delay)?;

        Ok(())
    }

    pub fn build(mut self) -> Report {
        self.report.total_duration = Instant::now() - self.start;
        self.report
    }

    fn new_histogram() -> Histogram {
        Histogram::new_with_bounds(1, 30 * 1_000_000, 3).unwrap()
    }
}
