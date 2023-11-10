use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use url::Url;

// --- DriverConfig ---

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DriverConfig {
    // Always need a plan
    pub plan: Plan,
    // Always need to know how we produce results
    pub telemetry: TelemetryConfig,
    // Sometimes have external runners
    pub external_runners: Option<RunnerDiscovery>,
    // If external runners are NOT present then a
    // runtime needs to be present since the runner will
    // be hosted locally
    pub runtime: Option<RuntimeConfig>,
}

#[allow(clippy::derivable_impls)]
impl Default for DriverConfig {
    fn default() -> Self {
        Self {
            plan: Plan::default(),
            telemetry: TelemetryConfig::default(),
            external_runners: None,
            runtime: Some(RuntimeConfig::default()),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RunnerDiscovery {
    // TODO: Rename serde to "static".
    pub static_runners: Vec<Url>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TelemetryConfig {
    // By default, have the agents create their own histograms
    // and then forward the results back to the driver/cli/entry
    // so that results /can/ be forwarded to a remote OTEL endpoint
    // but they are always written to stdout by the driver as well.
    pub otel_backend: Option<OtelBackendConfig>,
    pub logging: LoggingConfig,
}

impl Default for TelemetryConfig {
    fn default() -> Self {
        todo!()
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct OtelBackendConfig {
    pub endpoint: Url,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct LoggingConfig {
    pub level: LogLevel,
    pub format: String,
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

impl Default for LogLevel {
    fn default() -> Self {
        Self::Error
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MetronRunnerConfig {
    pub server_port: u16,
    pub telemetry: TelemetryConfig,
    pub runtime: RuntimeConfig,
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

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MetronControllerConfig {
    pub server_port: u16,
    // Always have external runners
    pub external_runners: RunnerDiscovery,
}
