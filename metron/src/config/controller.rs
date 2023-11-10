use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use url::Url;

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
