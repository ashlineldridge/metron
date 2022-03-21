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
    over: Duration,
}

impl Plan {
    /// Returns when the next request should be sent.
    ///
    /// # Arguments
    ///
    /// * `start` - When the test was started
    /// * `now` - The current time
    pub fn next(&self, start: Instant, now: Instant) -> Option<Instant> {
        assert!(start <= now, "Start value must be less than now");

        let progress = now - start;
        match (self.ramp, self.duration, self.rate) {
            // Are we finished?
            (_, Some(duration), _) if progress >= duration => None,

            // Are we ramping the rate up?
            (Some(ramp), _, _) if progress < ramp.over => {
                let delta = Duration::from_secs_f32(
                    (ramp.from.as_interval() - ramp.to.as_interval())
                        .as_secs_f32()
                        .abs()
                        * (progress.as_secs_f32() / ramp.duration.as_secs_f32()).min(1.0),
                );

                if ramp.from < ramp.to {
                    now + ramp.from.as_interval() - delta;
                } else {
                    now + ramp.from.as_interval() + delta
                }
            }

            // Are we going full tilt?
            (_, _, None) => now,

            // We must be doing a fixed-rate test.
            (_, _, Some(rate)) => now + rate.as_interval(),
        };
    }

    pub fn iter(&self, start: Instant) -> PlanIter {
        PlanIter {
            plan: self.clone(),
            start,
        }
    }
}

pub struct PlanIter {
    plan: Plan,
    start: Instant,
}

impl Iterator for PlanIter {
    type Item = Instant;

    fn next(&mut self) -> Option<Self::Item> {
        self.plan.next(self.start, Instant::now())
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
            },
        }
    }

    pub fn ramp(self, from: Rate, to: Rate, over: Duration) -> Builder {
        self.plan.ramp = Some(Ramp { from, to, over });
        self
    }

    pub fn duration(self, duration: Duration) -> Builder {
        self.plan.duration = duration;
        self
    }

    pub fn rate(self, rate: Rate) -> Builder {
        self.plan.rate = rate;
        self
    }

    pub fn build(&self) -> Result<Plan> {
        Ok(self.plan)
    }
}
