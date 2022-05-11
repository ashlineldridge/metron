use std::time::{Duration, Instant};

use metron::Rate;

/// Timing plan for outbound requests.
///
/// The plan dictates when requests should be sent should be sent to the
/// test target.
#[derive(Clone, Debug)]
pub struct Plan {
    /// Plan building blocks.
    blocks: Vec<RateBlock>,
    /// When the plan was started (none if not started).
    start: Option<Instant>,
    /// Previously returned instant (none if not started).
    prev: Option<Instant>,
    /// Cached plan duration (used for iterating).
    duration: Option<Duration>,
}

/// Describes how request rate should be treated over a given duration.
#[derive(Clone, Debug)]
pub enum RateBlock {
    /// Rate should be fixed over the given duration (or forever).
    Fixed(Rate, Option<Duration>),
    /// Rate should vary linearly over the given duration.
    Linear(Rate, Rate, Duration),
}

impl RateBlock {
    fn duration(&self) -> Option<Duration> {
        match self {
            RateBlock::Fixed(_, d) => *d,
            RateBlock::Linear(_, _, d) => Some(*d),
        }
    }
}

impl Plan {
    /// Resets the plan back to an unstarted state.
    pub fn reset(&mut self) {
        self.start = None;
        self.prev = None;
    }

    /// Gets the total duration of the plan.
    ///
    /// If the returned value is `None` the plan runs forever.
    fn duration(&mut self) -> Option<Duration> {
        if self.duration.is_none() {
            self.duration =
                self.blocks
                    .iter()
                    .fold(Some(Duration::from_secs(0)), |total, b| {
                        match (total, b.duration()) {
                            (Some(total), Some(d)) => Some(total + d),
                            _ => None,
                        }
                    });
        }

        self.duration
    }

    /// Gets the `RateBlock` that `progress` falls into.
    ///
    /// If the returned value is `None` then we have completed the plan.
    fn get_block(&self, progress: Duration) -> Option<RateBlock> {
        let mut total = Duration::from_secs(0);
        for b in &self.blocks {
            if let Some(d) = b.duration() {
                total += d;
                if progress < total {
                    return Some(b.clone());
                }
            } else {
                // The plan runs forever.
                return Some(b.clone());
            }
        }

        None
    }
}

impl Iterator for Plan {
    type Item = Instant;

    fn next(&mut self) -> Option<Self::Item> {
        // When did the plan start?
        let start = *self.start.get_or_insert(Instant::now());

        // How far into the plan are we?
        let progress = self.prev.unwrap_or(start) - start;

        if let Some(block) = self.get_block(progress) {
            // Calculate the next value in the series.
            let next = match block {
                RateBlock::Fixed(r, _) => self.prev.map(|t| t + r.as_interval()).unwrap_or(start),
                RateBlock::Linear(from, to, d) => {
                    let ramp_from = from.as_interval().as_secs_f32();
                    let ramp_to = to.as_interval().as_secs_f32();
                    let ramp_duration = d.as_secs_f32();
                    let progress = progress.as_secs_f32();

                    let ramp_progress_factor =
                        (ramp_from - ramp_to) * (progress / ramp_duration).min(1.0);
                    let delta = Duration::from_secs_f32(ramp_from - ramp_progress_factor);

                    self.prev.map(|t| t + delta).unwrap_or(start)
                }
            };

            self.prev = Some(next);

            // Check whether we've exceeded the duration.
            match self.duration() {
                Some(d) if next - start >= d => None,
                _ => Some(next),
            }
        } else {
            // We've finished.
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
///   .linear_rate_block(Rate(10), Rate(500), Duration::from_secs(60))
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
            plan: Plan {
                blocks: vec![],
                start: None,
                prev: None,
                duration: None,
            },
        }
    }

    pub fn blocks(mut self, blocks: &[RateBlock]) -> Builder {
        for block in blocks {
            self.plan.blocks.push(block.clone());
        }

        self
    }

    pub fn build(&self) -> Result<Plan, anyhow::Error> {
        Ok(self.plan.clone())
    }
}
