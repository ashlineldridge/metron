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
pub struct TelemetryBackendConfig {
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
