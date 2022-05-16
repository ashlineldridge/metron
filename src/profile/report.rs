use std::{
    collections::HashMap,
    fmt::Display,
    time::{Duration, Instant},
};

use anyhow::Result;
use hdrhistogram::Histogram;
use url::Url;

use super::profiler::Sample;

//
const ERROR_STATUS: u16 = 0;

type LatencyHistograms = HashMap<Url, HashMap<u16, (Histogram<u64>, Histogram<u64>)>>;

pub struct Report {
    latencies: LatencyHistograms,
    total_duration: Duration,
}

impl std::fmt::Debug for Report {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl Display for Report {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (target, m) in &self.latencies {
            for (status, (actual, corrected)) in m {
                let status = if *status == 0 {
                    "ERROR".to_string()
                } else {
                    status.to_string()
                };

                f.write_fmt(format_args!(
                    "
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

        f.write_fmt(format_args!("\nTotal duration: {:?}", self.total_duration))?;

        Ok(())
    }
}

/// Builder used to construct a [Report].
pub struct Builder {
    /// Latency histograms.
    latencies: LatencyHistograms,
    /// When we started building the report.
    start: Instant,
}

impl Builder {
    pub fn new() -> Self {
        Self {
            latencies: HashMap::new(),
            start: Instant::now(),
        }
    }

    pub fn record(&mut self, sample: &Sample) -> Result<()> {
        let status = match sample.status {
            Ok(status) => status,
            Err(_) => ERROR_STATUS,
        };

        let (actual, corrected) = self
            .latencies
            .entry(sample.target.clone())
            .or_default()
            .entry(status)
            .or_insert_with(|| (Self::new_histogram(), Self::new_histogram()));

        let value = sample.actual_latency().as_micros().try_into()?;
        actual.record(value)?;

        let value = sample.corrected_latency().as_micros().try_into()?;
        corrected.record(value)?;

        Ok(())
    }

    pub fn build(self) -> Report {
        Report {
            latencies: self.latencies,
            total_duration: Instant::now() - self.start,
        }
    }

    fn new_histogram() -> Histogram<u64> {
        Histogram::<u64>::new_with_bounds(1, 30 * 1_000_000, 3).unwrap()
    }
}
