use std::time::Duration;

use clap::ValueEnum;
use metron_core::Plan;
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TestConfig {
    pub agent: AgentConfig,
    pub plan: Plan,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AgentConfig {
    pub name: String,
    pub signaller: SignallerKind,
    pub worker_threads: usize,
    pub logging: LoggingConfig,
    pub prometheus: Option<PrometheusConfig>,
    pub open_telemetry: Option<OpenTelemetryConfig>,
    pub proxy: Vec<AgentDiscovery>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PollConfig {
    pub agents: Vec<AgentDiscovery>,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum SignallerKind {
    Dedicated,
    Cooperative,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum AgentDiscovery {
    Static { endpoints: Vec<String> },
    DnsRecord { refresh: Duration, dns_name: String },
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
