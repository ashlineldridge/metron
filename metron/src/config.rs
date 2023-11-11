use std::fmt::Debug;

use anyhow::bail;
use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use url::Url;

const DEFAULT_GRPC_PORT: u16 = 9090;

// --- Load Test Configuration ---

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct LoadTestConfig {
    // Always need a plan
    pub plan: LoadTestPlan,
    // Always need to know how we produce results
    pub telemetry: TelemetryConfig,
    // Sometimes have external runners
    pub external_runners: Option<RunnerDiscovery>,
    // If external runners are NOT present then a
    // runtime needs to be present since the runner will
    // be hosted locally
    pub runtime: Option<RuntimeConfig>,
}

impl Default for LoadTestConfig {
    fn default() -> Self {
        Self {
            plan: LoadTestPlan::default(),
            telemetry: TelemetryConfig::default(),
            external_runners: None,
            runtime: Some(RuntimeConfig::default()),
        }
    }
}

// --- Runner Configuration ---

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RunnerConfig {
    pub server_port: u16,
    pub telemetry: TelemetryConfig,
    pub runtime: RuntimeConfig,
}

impl Default for RunnerConfig {
    fn default() -> Self {
        Self {
            server_port: DEFAULT_GRPC_PORT,
            telemetry: Default::default(),
            runtime: Default::default(),
        }
    }
}

// --- Controller Configuration ---

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ControllerConfig {
    pub server_port: u16,
    pub external_runners: RunnerDiscovery,
}

impl Default for ControllerConfig {
    fn default() -> Self {
        Self {
            server_port: DEFAULT_GRPC_PORT,
            external_runners: RunnerDiscovery::default(),
        }
    }
}

// --- Test Plan ---

/// Load testing plan.
///
/// A [Plan] describes how a load test should be run.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct LoadTestPlan {
    pub segments: Vec<PlanSegment>,
    pub connections: usize,
    pub http_method: HttpMethod,
    pub targets: Vec<Url>,
    pub headers: Vec<Header>,
    pub payload: Option<String>,
    pub latency_correction: bool,
}

impl LoadTestPlan {
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

impl Default for LoadTestPlan {
    fn default() -> Self {
        Self {
            segments: vec![],
            connections: 10,
            http_method: HttpMethod::Get,
            targets: vec![],
            headers: vec![],
            payload: None,
            latency_correction: true,
        }
    }
}

/// How request rate should be treated over a given duration.
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

pub struct Ticks<'a> {
    /// The plan.
    plan: &'a LoadTestPlan,
    /// Cached plan duration.
    duration: Option<Duration>,
    /// When the plan was started.
    start: Instant,
    /// Previously returned instant (none if not started).
    prev: Option<Instant>,
}

impl<'a> Ticks<'a> {
    pub fn new(plan: &'a LoadTestPlan, start: Instant) -> Self {
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
pub struct RunnerDiscovery {
    // TODO: Rename serde to "static".
    pub static_runners: Vec<Url>,
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

use std::{
    fmt::Display,
    ops::Deref,
    str::FromStr,
    time::{Duration, Instant},
};

#[derive(Clone, Copy, Deserialize, Serialize)]
#[serde(try_from = "String")]
pub struct Rate(u32);

impl Rate {
    pub fn per_second(value: u32) -> Self {
        Self(value)
    }

    pub fn as_interval(&self) -> Duration {
        Duration::from_secs(1) / self.0
    }
}

impl Debug for Rate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self.0, f)
    }
}

impl Deref for Rate {
    type Target = u32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FromStr for Rate {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> core::result::Result<Self, Self::Err> {
        Rate::try_from(s.to_owned())
    }
}

/// This `TryFrom` implementation provides a simple means of validating rate values without
/// needing to provide a custom `Deserialize` implementation. See also other examples here:
/// https://github.com/serde-rs/serde/issues/939.
impl TryFrom<String> for Rate {
    type Error = anyhow::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let rate = value.parse()?;
        if rate == 0 {
            // TODO: Sort out error handling once and for all...
            bail!("request rate cannot be zero");
        }

        Ok(Rate(rate))
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Header {
    pub name: String,
    pub value: String,
}

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
