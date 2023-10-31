use std::{
    ops::Deref,
    str::FromStr,
    time::{Duration, Instant},
};

use anyhow::bail;
use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use url::Url;

mod wait;

pub mod runner;
pub mod signaller;

#[derive(Clone, Copy, Deserialize, Serialize)]
#[serde(try_from = "String")]
pub struct Rate(u32);

impl Rate {
    pub fn as_interval(&self) -> Duration {
        Duration::from_secs(1) / self.0
    }
}

impl std::fmt::Debug for Rate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl Deref for Rate {
    type Target = u32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FromStr for Rate {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> core::result::Result<Self, Self::Err> {
        Rate::try_from(s.to_owned())
    }
}

/// This `TryFrom` implementation provides a simple means of validating rate values without
/// needing to provide a custom `Deserialize` implementation. See also other examples here:
/// https://github.com/serde-rs/serde/issues/939.
impl TryFrom<String> for Rate {
    type Error = anyhow::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let rate = value.parse()?;
        if rate == 0 {
            // TODO: Sort out error handling once and for all...
            bail!("request rate cannot be zero");
        }

        Ok(Rate(rate))
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Off,
    Info,
    Debug,
    Warn,
    Error,
}

impl Default for LogLevel {
    fn default() -> Self {
        Self::Off
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Header {
    pub name: String,
    pub value: String,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Patch,
    Delete,
    Head,
    Options,
    Trace,
    Connect,
}

impl Default for HttpMethod {
    fn default() -> Self {
        Self::Get
    }
}

/// Load test timing plan for outbound requests.
///
/// The plan dictates when requests should be sent should be sent to the
/// test target.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct TestPlan {
    /// Segments that define how the request rate varies over the plan.
    pub segments: Vec<TestPlanSegment>,
    pub connections: usize,
    pub http_method: HttpMethod,
    pub targets: Vec<Url>,
    pub headers: Vec<Header>,
    pub payload: Option<String>,
    pub worker_threads: usize,
    pub signaller_kind: signaller::Kind,
    pub no_latency_correction: bool,
    pub stop_on_client_error: bool,
    pub stop_on_non_2xx: bool,
    pub log_level: LogLevel,
}

/// Describes how request rate should be treated over a given duration.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum TestPlanSegment {
    /// Rate should be fixed over the given duration (or forever).
    Fixed {
        rate: Rate,
        #[serde(default)]
        #[serde(with = "humantime_serde")]
        duration: Option<Duration>,
    },

    /// Rate should vary linearly over the given duration.
    Linear {
        rate_start: Rate,
        rate_end: Rate,
        #[serde(with = "humantime_serde")]
        duration: Duration,
    },
}

impl TestPlanSegment {
    fn duration(&self) -> Option<Duration> {
        match self {
            TestPlanSegment::Fixed { duration, .. } => *duration,
            TestPlanSegment::Linear { duration, .. } => Some(*duration),
        }
    }
}

impl TestPlan {
    pub fn ticks(&self, start: Instant) -> Ticks {
        Ticks::new(self, start)
    }

    /// Calculates the total duration of the plan.
    ///
    /// If the returned value is `None` the plan runs forever.
    pub fn calculate_duration(&self) -> Option<Duration> {
        self.segments
            .iter()
            .try_fold(Duration::from_secs(0), |total, seg| {
                seg.duration().map(|d| total + d)
            })
    }

    /// Finds the `PlanSegment` that `progress` falls into.
    ///
    /// If the returned value is `None` then we have completed the plan.
    fn find_segment(&self, progress: Duration) -> Option<TestPlanSegment> {
        let mut total = Duration::from_secs(0);
        for seg in &self.segments {
            if let Some(d) = seg.duration() {
                total += d;
                if progress < total {
                    return Some(seg.clone());
                }
            } else {
                // The plan runs forever.
                return Some(seg.clone());
            }
        }

        None
    }
}

pub struct Ticks<'a> {
    /// The plan.
    plan: &'a TestPlan,
    /// Cached plan duration.
    duration: Option<Duration>,
    /// When the plan was started.
    start: Instant,
    /// Previously returned instant (none if not started).
    prev: Option<Instant>,
}

impl<'a> Ticks<'a> {
    pub fn new(plan: &'a TestPlan, start: Instant) -> Self {
        Self {
            plan,
            duration: plan.calculate_duration(),
            start,
            prev: None,
        }
    }
}

impl<'a> Iterator for Ticks<'a> {
    type Item = Instant;

    fn next(&mut self) -> Option<Self::Item> {
        // How far into the plan are we?
        let progress = self.prev.unwrap_or(self.start) - self.start;

        if let Some(block) = self.plan.find_segment(progress) {
            // Calculate the next value in the series.
            let next = match block {
                TestPlanSegment::Fixed { rate, .. } => self
                    .prev
                    .map(|t| t + rate.as_interval())
                    .unwrap_or(self.start),

                TestPlanSegment::Linear {
                    rate_start,
                    rate_end,
                    duration,
                } => {
                    let ramp_start = rate_start.as_interval().as_secs_f32();
                    let ramp_end = rate_end.as_interval().as_secs_f32();
                    let duration = duration.as_secs_f32();
                    let progress = progress.as_secs_f32();

                    let ramp_progress_factor =
                        (ramp_start - ramp_end) * (progress / duration).min(1.0);
                    let delta = Duration::from_secs_f32(ramp_start - ramp_progress_factor);

                    self.prev.map(|t| t + delta).unwrap_or(self.start)
                }
            };

            self.prev = Some(next);

            if let Some(d) = self.duration && next - self.start >= d {
                None
            } else {
                self.prev
            }
        } else {
            None
        }
    }
}

/// Builder used to construct a [Plan].
///
/// # Examples
/// ```
/// use crate::plan::Builder;
/// use metron_old::Rate;
///
/// // Construct a plan that ramps up throughput from 10 RPS to 500 RPS over
/// // the first 60 seconds and then maintains 500 RPS for a further 5 minutes.
/// let plan = Builder::new()
///   .segments(vec![])
///   .fixed_rate_block(Rate(500), Duration::from_secs(5 * 60))
///   .build()
///   .unwrap();
/// ```
pub struct Builder {
    /// The plan under construction.
    plan: TestPlan,
}

impl Builder {
    pub fn segments(mut self, segments: &[TestPlanSegment]) -> Builder {
        for seg in segments {
            self.plan.segments.push(seg.clone());
        }

        self
    }

    pub fn build(self) -> TestPlan {
        self.plan
    }
}

impl Default for Builder {
    fn default() -> Self {
        Self {
            plan: TestPlan { segments: vec![] },
        }
    }
}
