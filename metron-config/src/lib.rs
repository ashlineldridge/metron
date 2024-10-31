use std::{collections::HashMap, time::Duration};

use clap::ValueEnum;
use metron_core::Plan;
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct LocalTestConfig {
    pub plan: Plan,
    pub signaller: Option<SignallerKind>,
    pub worker_threads: Option<usize>,
    pub logging: Option<LoggingConfig>,
    pub prometheus: Option<PrometheusConfig>,
    pub open_telemetry: Option<OpenTelemetryConfig>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RemoteTestConfig {
    pub plan: Plan,
    pub agents: Vec<RemoteAgentDiscovery>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AgentConfig {
    pub name: Option<String>,
    pub signaller: Option<SignallerKind>,
    pub worker_threads: Option<usize>,
    pub logging: Option<LoggingConfig>,
    pub prometheus: Option<PrometheusConfig>,
    pub open_telemetry: Option<OpenTelemetryConfig>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CancelConfig {
    pub agents: Vec<RemoteAgentDiscovery>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ReportConfig {
    pub agents: Vec<RemoteAgentDiscovery>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ProxyConfig {
    pub agents: Vec<RemoteAgentDiscovery>,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum SignallerKind {
    Dedicated,
    Cooperative,
}

impl Default for SignallerKind {
    fn default() -> Self {
        Self::Dedicated
    }
}

impl From<SignallerKind> for metron_core::SignallerKind {
    fn from(kind: SignallerKind) -> Self {
        match kind {
            SignallerKind::Dedicated => metron_core::SignallerKind::Dedicated,
            SignallerKind::Cooperative => metron_core::SignallerKind::Cooperative,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "lowercase", content = "spec")]
pub enum RemoteAgentDiscovery {
    Static(StaticAgentDiscovery),
    KubePod(KubePodAgentDiscovery),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct StaticAgentDiscovery {
    pub endpoints: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct KubePodAgentDiscovery {
    pub match_labels: HashMap<String, String>,
    pub port: u16,
    pub refresh: Duration,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PrometheusConfig {
    pub port: u16,
    pub path: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct OpenTelemetryConfig {
    pub address: Url,
    pub period: Duration,
    pub timeout: Duration,
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
