use std::time::{Duration, Instant};

use metron::Rate;
use serde::{Deserialize, Serialize, ser::SerializeStruct, de::Visitor};
use serde::de;

/// Timing plan for outbound requests.
///
/// The plan dictates when requests should be sent should be sent to the
/// test target.
#[derive(Clone, Debug, Serialize)]
pub struct Plan {
    /// Plan building blocks.
    blocks: Vec<RateBlock>,
}

impl Plan {
    pub fn new(blocks: Vec<RateBlock>) -> Self {
        Self { blocks }
    }
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

impl Serialize for RateBlock {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            RateBlock::Fixed(r, d) => {
                let mut state = serializer.serialize_struct("RateBlock", 2)?;
                state.serialize_field("rate", &r.0)?;
                state.serialize_field("duration", d)?;
                state.end()
            }
            RateBlock::Linear(r1, r2, d) => {
                let mut state = serializer.serialize_struct("RateBlock", 3)?;
                state.serialize_field("rate_from", &r1.0)?;
                state.serialize_field("rate_to", &r2.0)?;
                state.serialize_field("duration", d)?;
                state.end()
            }
        }
    }
}

impl<'de> Deserialize<'de> for RateBlock {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &["kind", "rate", "rate_from", "rate_to", "duration"];

        #[derive(Deserialize)]
        #[serde(field_identifier, rename_all = "lowercase")]
        enum Kind { Fixed, Linear }

        struct RateBlockVisitor;
        impl<'de> Visitor<'de> for RateBlockVisitor {
            type Value = RateBlock;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("enum RateBlock")
            }

            fn visit_map<A>(self, map: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::MapAccess<'de>, {
                let mut kind = None;
                let mut rate1 = None;
                let mut rate2 = None;
                let mut duration = None;
                while let Some(key) = map.next_key()? {
                    match key {
                        "kind" => {
                            if kind.is_some() {
                                return Err(de::Error::duplicate_field("kind"));
                            } else {
                                kind = Some(map.next_value()?);
                            }
                        }
                        _ => return Err(de::Error::unknown_field(key, FIELDS)),
                    }
                }

                match kind {
                    Some(Kind::Fixed) => todo!(),
                    Some(Kind::Linear) => todo!(),
                    None => Err(de::Error::missing_field("kind"))
                }
            }
        }

        deserializer.deserialize_struct("RateBlock", FIELDS, RateBlockVisitor)
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
        self.blocks
            .iter()
            .fold(Some(Duration::from_secs(0)), |total, b| {
                match (total, b.duration()) {
                    (Some(total), Some(d)) => Some(total + d),
                    _ => None,
                }
            })
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

        if let Some(block) = self.plan.get_block(progress) {
            // Calculate the next value in the series.
            let next = match block {
                RateBlock::Fixed(r, _) => {
                    self.prev.map(|t| t + r.as_interval()).unwrap_or(self.start)
                }
                RateBlock::Linear(from, to, d) => {
                    let ramp_from = from.as_interval().as_secs_f32();
                    let ramp_to = to.as_interval().as_secs_f32();
                    let ramp_duration = d.as_secs_f32();
                    let progress = progress.as_secs_f32();

                    let ramp_progress_factor =
                        (ramp_from - ramp_to) * (progress / ramp_duration).min(1.0);
                    let delta = Duration::from_secs_f32(ramp_from - ramp_progress_factor);

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
            plan: Plan { blocks: vec![] },
        }
    }

    #[allow(dead_code)]
    pub fn fixed_rate_block(mut self, r: Rate, d: Option<Duration>) -> Builder {
        self.plan.blocks.push(RateBlock::Fixed(r, d));
        self
    }

    #[allow(dead_code)]
    pub fn linear_rate_block(mut self, start: Rate, end: Rate, d: Duration) -> Builder {
        self.plan.blocks.push(RateBlock::Linear(start, end, d));
        self
    }

    pub fn blocks(mut self, blocks: &[RateBlock]) -> Builder {
        for block in blocks {
            self.plan.blocks.push(block.clone());
        }

        self
    }

    pub fn build(self) -> Plan {
        self.plan
    }
}
