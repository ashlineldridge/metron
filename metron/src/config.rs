use std::{
    collections::HashMap,
    fmt::Display,
    time::{Duration, Instant},
};

use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use url::Url;

// --- Load Test Configuration ---

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TestConfig {
    pub plan: Plan,
    pub runners: Option<RunnerDiscoveryConfig>,
    pub runtime: Option<RuntimeConfig>,
    pub telemetry: TelemetryConfig,
}

// --- Runner Configuration ---

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RunnerConfig {
    pub port: u16,
    pub runtime: RuntimeConfig,
    pub telemetry: TelemetryConfig,
}

// --- Controller Configuration ---

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ControllerConfig {
    pub port: u16,
    pub runners: RunnerDiscoveryConfig,
    pub telemetry: TelemetryConfig,
}

// --- Run Config ---

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RunConfig {
    pub port: u16,
    pub runners: RunnerDiscoveryConfig,
    pub telemetry: TelemetryConfig,
}

// --- Test Plan ---

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

impl<'a> Iterator for Ticks<'a> {
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

// --- Supporting Types ---

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct RunnerDiscoveryConfig {
    #[serde(rename = "static")]
    pub static_runners: Vec<String>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct TelemetryConfig {
    pub open_telemetry: Option<OpenTelemetryBackend>,
    pub logging: LoggingConfig,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct OpenTelemetryBackend {
    pub endpoint: Url,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct LoggingConfig {
    pub level: LogLevel,
    pub format: LogFormat,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum LogFormat {
    Bunyan,
    Json,
}

impl Default for LogFormat {
    fn default() -> Self {
        Self::Bunyan
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
        Self::Error
    }
}

impl From<LogLevel> for tracing_core::LevelFilter {
    fn from(level: LogLevel) -> Self {
        match level {
            LogLevel::Off => tracing_core::LevelFilter::OFF,
            LogLevel::Error => tracing_core::LevelFilter::ERROR,
            LogLevel::Warn => tracing_core::LevelFilter::WARN,
            LogLevel::Info => tracing_core::LevelFilter::INFO,
            LogLevel::Debug => tracing_core::LevelFilter::DEBUG,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RuntimeConfig {
    pub signaller: SignallerKind,
    pub worker_threads: usize,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            signaller: SignallerKind::Dedicated,
            worker_threads: num_cpus::get(),
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum SignallerKind {
    Dedicated,
    Cooperative,
}

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

impl Default for HttpMethod {
    fn default() -> Self {
        Self::Get
    }
}

// This impl is used to provide the default value used by clap. When Metron
// becomes more protocol agnostic this can go away.
impl Display for HttpMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value = serde_yaml::to_string(self).unwrap();
        write!(f, "{}", value.trim())?;

        Ok(())
    }
}
