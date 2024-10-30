use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use url::Url;

pub type Rate = f32;
pub type Headers = HashMap<String, String>;
pub type Environment = HashMap<String, String>;

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

/// Load testing plan.
///
/// A [Plan] describes how a load test should be run.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Plan {
    pub segments: Vec<RateSegment>,
    pub actions: Vec<Action>,
}

impl Plan {
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
    fn find_segment(&self, progress: Duration) -> Option<RateSegment> {
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

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum Action {
    Http {
        method: HttpMethod,
        headers: Headers,
        payload: String,
        target: Url,
    },
    Udp {
        payload: String,
        target: Url,
    },
    // TODO: Optionally compile in support for certain things.
    // E.g. A https://github.com/RustPython/RustPython might be nice
    // but don't want all builds to pull in that dependency.
    Exec {
        command: String,
        args: Vec<String>,
        env: Environment,
    },
    // See: https://docs.datadoghq.com/synthetics/api_tests/grpc_tests/?tab=behaviorcheck
    // Grpc {
    //     // What is a stronger type to use here?
    //     // proto_file: String,
    //     // rpc: String,
    //     // payload: String,
    // },
    Wasm {
        // TODO: For running a WASM module.
    },
}

/// How request rate should be treated over a given duration.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum RateSegment {
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

impl RateSegment {
    fn duration(&self) -> Option<Duration> {
        match self {
            RateSegment::Fixed { duration, .. } => *duration,
            RateSegment::Linear { duration, .. } => Some(*duration),
        }
    }
}

pub struct Ticks<'a> {
    /// The plan.
    plan: &'a Plan,
    /// Cached plan duration.
    duration: Option<Duration>,
    /// When the plan was started.
    start: Instant,
    /// Previously returned instant (none if not started).
    prev: Option<Instant>,
}

impl<'a> Ticks<'a> {
    pub fn new(plan: &'a Plan, start: Instant) -> Self {
        Self {
            plan,
            duration: plan.calculate_duration(),
            start,
            prev: None,
        }
    }

    fn rate_period(rate: Rate) -> Duration {
        Duration::from_secs_f32(1.0 / rate)
    }
}

impl Iterator for Ticks<'_> {
    type Item = Instant;

    fn next(&mut self) -> Option<Self::Item> {
        // How far into the plan are we?
        let progress = self.prev.unwrap_or(self.start) - self.start;

        if let Some(block) = self.plan.find_segment(progress) {
            // Calculate the next value in the series.
            let next = match block {
                RateSegment::Fixed { rate, .. } => self
                    .prev
                    .map(|t| t + Self::rate_period(rate))
                    .unwrap_or(self.start),

                RateSegment::Linear {
                    rate_start,
                    rate_end,
                    duration,
                } => {
                    let ramp_start = Self::rate_period(rate_start).as_secs_f32();
                    let ramp_end = Self::rate_period(rate_end).as_secs_f32();
                    let duration = duration.as_secs_f32();
                    let progress = progress.as_secs_f32();

                    let ramp_progress_factor =
                        (ramp_start - ramp_end) * (progress / duration).min(1.0);
                    let delta = Duration::from_secs_f32(ramp_start - ramp_progress_factor);

                    self.prev.map(|t| t + delta).unwrap_or(self.start)
                }
            };

            self.prev = Some(next);

            if let Some(d) = self.duration
                && next - self.start >= d
            {
                None
            } else {
                self.prev
            }
        } else {
            None
        }
    }
}
