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

type Histogram = hdrhistogram::Histogram<u64>;

#[derive(Clone, Default)]
pub struct Report {
    /// Response latency histograms keyed by target URL, then by HTTP status. The values are
    /// tuples of `(actual, corrected)` where "actual" measures the duration between when a
    /// request was sent and when the response was received, and "corrected" measures the
    /// duration between when a request _should_ have been sent and when it was received.
    response_histograms: HashMap<Url, HashMap<u16, (Histogram, Histogram)>>,

    /// Client latency histograms keyed by target URL. These histograms track the latency as
    /// measured between when a request was sent and when it should have been sent (i.e., when the
    /// request latency increases it means that we cannot keep up with the desired request rate).
    client_histograms: HashMap<Url, Histogram>,

    /// Error latency histograms keyed by target URL. Values are tuples of `(actual, corrected)`.
    error_histograms: HashMap<Url, (Histogram, Histogram)>,

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
                    Duration::from_micros(corrected.value_at_quantile(0.9999)),
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
                Duration::from_micros(corrected.value_at_quantile(0.9999)),
                actual.len(),
            ))?;
        }

        for (target, hist) in &self.client_histograms {
            f.write_fmt(format_args!(
                "
Request Latency
---------------
Target URL:                  {}
Latency (95%):               {:?}
Latency (99%):               {:?}
Latency (99.9%):             {:?}
Latency (99.99%):            {:?}
Latency (99.999%):           {:?}
Total Requests:              {}\n",
                target,
                Duration::from_micros(hist.value_at_quantile(0.95)),
                Duration::from_micros(hist.value_at_quantile(0.99)),
                Duration::from_micros(hist.value_at_quantile(0.999)),
                Duration::from_micros(hist.value_at_quantile(0.9999)),
                Duration::from_micros(hist.value_at_quantile(0.9999)),
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
        let actual_latency = sample.actual_latency().as_micros().try_into()?;
        let corrected_latency = sample.corrected_latency().as_micros().try_into()?;
        let client_latency = sample.client_latency().as_micros().try_into()?;

        if let Ok(status) = sample.status {
            let (actual_histogram, corrected_histogram) = self
                .report
                .response_histograms
                .entry(sample.target.clone())
                .or_default()
                .entry(status)
                .or_insert_with(|| (Self::new_histogram(), Self::new_histogram()));

            actual_histogram.record(actual_latency)?;
            corrected_histogram.record(corrected_latency)?;
        } else {
            let (actual_histogram, corrected_histogram) = self
                .report
                .error_histograms
                .entry(sample.target.clone())
                .or_insert_with(|| (Self::new_histogram(), Self::new_histogram()));

            actual_histogram.record(actual_latency)?;
            corrected_histogram.record(corrected_latency)?;
        }

        let client_histogram = self
            .report
            .client_histograms
            .entry(sample.target.clone())
            .or_insert_with(Self::new_histogram);

        client_histogram.record(client_latency)?;

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
