use std::time::{Duration, Instant};

#[derive(Clone)]
pub struct RequestSchedule {
    pub start: Instant,
}

pub struct FixedIntervalSchedule {
    index: u32,
    start: Instant,
    interval: Duration,
    duration: Duration,
}

impl FixedIntervalSchedule {
    pub fn new(start: Instant, interval: Duration, duration: Duration) -> Self {
        Self {
            index: 0,
            start,
            interval,
            duration,
        }
    }
}

impl Iterator for FixedIntervalSchedule {
    type Item = RequestSchedule;

    fn next(&mut self) -> Option<Self::Item> {
        let offset = self.index * self.interval;
        if offset < self.duration {
            Some(RequestSchedule {
                start: self.start + offset,
            })
        } else {
            None
        }
    }
}

pub struct RampedFixedIntervalSchedule {
    start: Instant,
    previous: Option<Instant>,
    init_interval: Duration,
    ramp_duration: Duration,
    main_interval: Duration,
    main_duration: Duration,
}

impl RampedFixedIntervalSchedule {
    pub fn new(
        start: Instant,
        init_interval: Duration,
        main_interval: Duration,
        ramp_duration: Duration,
        main_duration: Duration,
    ) -> Self {
        Self {
            start,
            previous: None,
            init_interval,
            main_interval,
            ramp_duration,
            main_duration,
        }
    }
}

impl Iterator for RampedFixedIntervalSchedule {
    type Item = RequestSchedule;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(previous) = self.previous {
            let prog = (previous - self.start).as_secs_f32();
            let mult = (prog / self.ramp_duration.as_secs_f32()).min(1.0);
            let diff = mult * (self.init_interval - self.main_interval).as_secs_f32();
            let next = previous + self.init_interval - Duration::from_secs_f32(diff);

            let end = self.start + self.ramp_duration + self.main_duration;
            if next < end {
                self.previous = Some(next);
                Some(RequestSchedule { start: next })
            } else {
                None
            }
        } else {
            self.previous = Some(self.start);
            Some(RequestSchedule { start: self.start })
        }
    }
}
