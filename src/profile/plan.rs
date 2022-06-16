use std::time::{Duration, Instant};

use metron::Rate;
use serde::{Deserialize, Serialize};

/// Timing plan for outbound requests.
///
/// The plan dictates when requests should be sent should be sent to the
/// test target.
#[derive(Clone, Debug, Deserialize)]
pub struct Plan {
    /// Segments that define how the request rate varies over the plan.
    segments: Vec<PlanSegment>,
}

/// Describes how request rate should be treated over a given duration.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum PlanSegment {
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

impl PlanSegment {
    fn duration(&self) -> Option<Duration> {
        match self {
            PlanSegment::Fixed { duration, .. } => *duration,
            PlanSegment::Linear { duration, .. } => Some(*duration),
        }
    }
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
            .fold(Some(Duration::from_secs(0)), |total, seg| {
                match (total, seg.duration()) {
                    (Some(total), Some(d)) => Some(total + d),
                    _ => None,
                }
            })
    }

    /// Finds the `PlanSegment` that `progress` falls into.
    ///
    /// If the returned value is `None` then we have completed the plan.
    fn find_segment(&self, progress: Duration) -> Option<PlanSegment> {
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
}

impl<'a> Iterator for Ticks<'a> {
    type Item = Instant;

    fn next(&mut self) -> Option<Self::Item> {
        // How far into the plan are we?
        let progress = self.prev.unwrap_or(self.start) - self.start;

        if let Some(block) = self.plan.find_segment(progress) {
            // Calculate the next value in the series.
            let next = match block {
                PlanSegment::Fixed { rate, .. } => self
                    .prev
                    .map(|t| t + rate.as_interval())
                    .unwrap_or(self.start),

                PlanSegment::Linear {
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
/// use metron::Rate;
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
    plan: Plan,
}

impl Builder {
    pub fn new() -> Self {
        Self {
            plan: Plan { segments: vec![] },
        }
    }

    pub fn segments(mut self, segments: &[PlanSegment]) -> Builder {
        for seg in segments {
            self.plan.segments.push(seg.clone());
        }

        self
    }

    pub fn build(self) -> Plan {
        self.plan
    }
}
