use std::{collections::HashMap, time::Duration};

use clap::ValueEnum;
use quanta::Instant;
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

impl From<&HttpMethod> for reqwest::Method {
    fn from(method: &HttpMethod) -> Self {
        match method {
            HttpMethod::Get => reqwest::Method::GET,
            HttpMethod::Post => reqwest::Method::POST,
            HttpMethod::Put => reqwest::Method::PUT,
            HttpMethod::Patch => reqwest::Method::PATCH,
            HttpMethod::Delete => reqwest::Method::DELETE,
            HttpMethod::Head => reqwest::Method::HEAD,
            HttpMethod::Options => reqwest::Method::OPTIONS,
            HttpMethod::Trace => reqwest::Method::TRACE,
            HttpMethod::Connect => reqwest::Method::CONNECT,
        }
    }
}

/// Load testing plan.
///
/// A [Plan] describes how a load test should be run.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Plan {
    pub name: String,
    pub segments: Vec<Segment>,
    pub actions: Vec<Action>,
}

impl Plan {
    pub fn empty() -> Self {
        Self {
            name: String::new(),
            segments: vec![],
            actions: vec![],
        }
    }

    pub fn ticks(&self, start: Instant) -> Ticks {
        Ticks::new(self, start)
    }

    /// Calculates the total duration of the plan.
    pub fn calculate_duration(&self) -> Duration {
        self.segments
            .iter()
            .fold(Duration::from_secs(0), |total, seg| total + seg.duration)
    }

    /// Finds the `PlanSegment` that `progress` falls into.
    ///
    /// If the returned value is `None` then we have completed the plan.
    fn find_segment(&self, progress: Duration) -> Option<Segment> {
        let mut total = Duration::from_secs(0);
        for seg in &self.segments {
            total += seg.duration;
            if progress < total {
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
        name: String,
        target: Url,
        method: HttpMethod,
        headers: Headers,
        payload: Vec<u8>,
    },
    Udp {
        name: String,
        target: Url,
        payload: Vec<u8>,
    },
    // TODO: Optionally compile in support for certain things.
    // E.g. A https://github.com/RustPython/RustPython might be nice
    // but don't want all builds to pull in that dependency.
    Exec {
        name: String,
        command: String,
        args: Vec<String>,
        env: Environment,
    },
    // See: https://docs.datadoghq.com/synthetics/api_tests/grpc_tests/?tab=behaviorcheck
    // Grpc {
    //     // name: String,
    //     // What is a stronger type to use here?
    //     // proto_file: String,
    //     // rpc: String,
    //     // payload: String,
    // },
    Wasm {
        // TODO: For running a WASM module.
        // name: String,
    },
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Segment {
    pub name: String,
    pub rate_start: Rate,
    pub rate_end: Rate,
    #[serde(with = "humantime_serde")]
    pub duration: Duration,
}

pub struct Ticks<'a> {
    /// The plan.
    plan: &'a Plan,
    /// Cached plan duration.
    duration: Duration,
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

        // Calculate the next value in the series.
        if let Some(seg) = self.plan.find_segment(progress) {
            let ramp_start = Self::rate_period(seg.rate_start).as_secs_f32();
            let ramp_end = Self::rate_period(seg.rate_end).as_secs_f32();
            let duration = seg.duration.as_secs_f32();
            let progress = progress.as_secs_f32();
            let ramp_progress_factor = (ramp_start - ramp_end) * (progress / duration).min(1.0);
            let delta = Duration::from_secs_f32(ramp_start - ramp_progress_factor);

            let next = self.prev.map(|t| t + delta).unwrap_or(self.start);
            self.prev = Some(next);

            if next - self.start >= self.duration {
                None
            } else {
                self.prev
            }
        } else {
            None
        }
    }
}
