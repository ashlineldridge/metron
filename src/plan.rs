use anyhow::Result;
use std::time::{Duration, Instant};

use wrkr::Rate;

/// Timing plan for outbound requests.
///
/// The plan dictates when requests should be sent should be sent to the
/// test target.
#[derive(Clone)]
pub struct Plan {
    /// Optional ramp used to increase rate over an initial duration.
    ramp: Option<Ramp>,
    /// Optional duration (none implies run forever).
    duration: Option<Duration>,
    /// Optional rate (none implies maximal rate).
    rate: Option<Rate>,
    /// When the plan was started (none if not started).
    start: Option<Instant>,
    /// Previously returned instant (none if not started).
    prev: Option<Instant>,
}

/// Ramp specs used to vary the initial rate of the `Plan`.
///
/// Ramping can be used to increase the request rate over a fixed time period
/// to allow the target infrastructure to scale up.
#[derive(Clone)]
struct Ramp {
    /// Rate that the ramp starts at.
    from: Rate,
    /// Rate that the ramp finishes at.
    to: Rate,
    /// Time period over which to vary the rate.
    duration: Duration,
}

impl Plan {
    /// Resets the plan back to an unstarted state.
    pub fn reset(&mut self) {
        self.start = None;
        self.prev = None;
    }
}

impl Iterator for Plan {
    type Item = Instant;

    fn next(&mut self) -> Option<Self::Item> {
        // When did the plan start?
        let start = *self.start.get_or_insert(Instant::now());

        // How far into the plan are we?
        let progress = self.prev.unwrap_or(start) - start;

        // Calculate the next instant that a request should be sent.
        let next = match (&self.ramp, &self.rate) {
            // Are we ramping the rate up?
            (Some(ramp), _) if progress < ramp.duration => {
                let ramp_from = ramp.from.as_interval().as_secs_f32();
                let ramp_to = ramp.to.as_interval().as_secs_f32();
                let progress = progress.as_secs_f32();
                let ramp_duration = ramp.duration.as_secs_f32();

                let ramp_progress_factor =
                    (ramp_from - ramp_to) * (progress / ramp_duration).min(1.0);
                let delta = Duration::from_secs_f32(ramp_from - ramp_progress_factor);

                self.prev.map(|t| t + delta).unwrap_or(start)
            }

            // We must be doing a fixed-rate test.
            (_, Some(rate)) => self.prev.map(|t| t + rate.as_interval()).unwrap_or(start),

            // Are we going full tilt?
            (_, None) => Instant::now(),
        };

        self.prev = Some(next);

        match self.duration {
            Some(d) if next - start >= d => None,
            _ => Some(next),
        }
    }
}

/// Builder used to construct a [Plan].
///
/// # Examples
/// ```
/// use crate::plan::Builder;
/// use wrkr::Rate;
///
/// // Construct a maximal throughput plan that runs forever.
/// let plan = Builder::new()
///   .build()
///   .unwrap();
///
/// // Construct a fixed throughput plan (500 RPS) that runs for 5 minutes.
/// let plan = Builder::new()
///   .rate(Rate(500))
///   .duration(Duration::from_secs(5 * 60))
///   .build()
///   .unwrap();
///
/// // Construct a plan that ramps up throughput from 10 RPS to 500 RPS over
/// // the first 60 seconds and then maintains 500 RPS for a further 5 minutes.
/// let plan = Builder::new()
///   .ramp(Rate(10), Rate(500), Duration::from_secs(60))
///   .rate(Rate(500))
///   .duration(Duration::from_secs(5 * 60))
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
                ramp: None,
                duration: None,
                rate: None,
                start: None,
                prev: None,
            },
        }
    }

    pub fn ramp(mut self, from: Rate, to: Rate, over: Duration) -> Builder {
        self.plan.ramp = Some(Ramp {
            from,
            to,
            duration: over,
        });
        self
    }

    pub fn duration(mut self, d: Duration) -> Builder {
        self.plan.duration = Some(d);
        self
    }

    pub fn rate(mut self, r: Rate) -> Builder {
        self.plan.rate = Some(r);
        self
    }

    pub fn build(&self) -> Result<Plan> {
        Ok(self.plan.clone())
    }
}
