use std::time::{Duration, Instant};

use wrkr::Rate;

#[allow(dead_code)]
pub fn asap() -> AsapSchedule {
    AsapSchedule::new()
}

pub fn fixed_rate(rate: Rate) -> FixedRateSchedule {
    FixedRateSchedule::new(Duration::from_secs(1) / rate)
}

pub fn ramped_rate(from: Rate, to: Rate, over: Duration) -> RampedRateSchedule {
    let from = Duration::from_secs(1) / from;
    let to = Duration::from_secs(1) / to;
    RampedRateSchedule::new(from, to, over)
}

pub fn finite<S: Schedule>(duration: Duration, inner: S) -> FiniteDurationSchedule<S> {
    FiniteDurationSchedule::new(duration, inner)
}

pub trait Schedule: Send {
    fn next(&mut self) -> Option<Duration>;

    fn iter_from(self, start: Instant) -> ScheduleIter<Self>
    where
        Self: Sized,
    {
        ScheduleIter {
            schedule: self,
            start,
        }
    }

    fn boxed<'a>(self) -> Box<dyn Schedule + 'a>
    where
        Self: Sized + 'a,
    {
        Box::new(self) as Box<dyn Schedule>
    }
}

pub struct ScheduleIter<S: Schedule> {
    schedule: S,
    start: Instant,
}

impl<S: Schedule> Iterator for ScheduleIter<S> {
    type Item = Instant;

    fn next(&mut self) -> Option<Self::Item> {
        self.schedule.next().map(|n| self.start + n)
    }
}

impl Schedule for Box<dyn Schedule> {
    fn next(&mut self) -> Option<Duration> {
        self.as_mut().next()
    }
}

pub struct AsapSchedule {
    start: Option<Instant>,
}

impl AsapSchedule {
    #[allow(dead_code)]
    fn new() -> Self {
        Self { start: None }
    }
}

impl Schedule for AsapSchedule {
    fn next(&mut self) -> Option<Duration> {
        let start = self.start.expect("AsapSchedule expected start value");
        Some(Instant::now() - start)
    }

    fn iter_from(mut self, start: Instant) -> ScheduleIter<Self> {
        self.start = Some(start);
        ScheduleIter {
            schedule: self,
            start,
        }
    }
}

pub struct FixedRateSchedule {
    prev: Option<Duration>,
    interval: Duration,
}

impl FixedRateSchedule {
    fn new(interval: Duration) -> Self {
        Self {
            prev: None,
            interval,
        }
    }
}

impl Schedule for FixedRateSchedule {
    fn next(&mut self) -> Option<Duration> {
        let next = if let Some(prev) = self.prev {
            Some(prev + self.interval)
        } else {
            Some(Duration::from_secs(0))
        };

        self.prev = next;
        next
    }
}

pub struct FiniteDurationSchedule<S: Schedule> {
    duration: Duration,
    inner: S,
}

impl<S: Schedule> FiniteDurationSchedule<S> {
    fn new(duration: Duration, inner: S) -> Self {
        Self { duration, inner }
    }
}

impl<S: Schedule> Schedule for FiniteDurationSchedule<S> {
    fn next(&mut self) -> Option<Duration> {
        self.inner
            .next()
            .and_then(|n| if n < self.duration { Some(n) } else { None })
    }
}

pub struct RampedRateSchedule {
    progress: Duration,
    ramp_from_interval: Duration,
    ramp_to_interval: Duration,
    ramp_duration: Duration,
}

impl RampedRateSchedule {
    pub fn new(
        ramp_from_interval: Duration,
        ramp_to_interval: Duration,
        ramp_duration: Duration,
    ) -> Self {
        Self {
            progress: Duration::from_secs(0),
            ramp_from_interval,
            ramp_to_interval,
            ramp_duration,
        }
    }
}

impl Schedule for RampedRateSchedule {
    fn next(&mut self) -> Option<Duration> {
        let next = self.ramp_from_interval
            - Duration::from_secs_f32(
                (self.ramp_from_interval - self.ramp_to_interval).as_secs_f32()
                    * (self.progress.as_secs_f32() / self.ramp_duration.as_secs_f32()).min(1.0),
            );

        self.progress += next;
        Some(next)
    }
}
