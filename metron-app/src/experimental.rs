use std::time::Duration;

use anyhow::Result;
use clap::Subcommand;
use metron_config::*;
use metron_core::Plan;

#[derive(Clone, Debug, Subcommand)]
pub enum Command {
    /// Generate example configuration files
    Generate {
        // Can put args in here to print specific types, write to a directory, etc.
    },
}

pub fn print_example_configs_as_yaml() -> Result<()> {
    let config = TestConfig {
        agent: AgentConfig {
            name: "local-agent".to_owned(),
            signaller: SignallerKind::Dedicated,
            worker_threads: 8,
            logging: LoggingConfig {
                level: LogLevel::Debug,
                format: LogFormat::Bunyan,
            },
            prometheus: Some(PrometheusConfig {
                port: 8081,
                path: "/metrics".to_owned(),
            }),
            open_telemetry: Some(OpenTelemetryConfig {
                address: url::Url::parse("http://localhost:8989")?,
                period: Duration::from_secs(60),
                timeout: Duration::from_secs(60),
            }),
            proxy: vec![AgentDiscovery::Static {
                endpoints: vec!["127.0.0.1:8585".to_owned()],
            }],
        },
        plan: Plan {
            segments: vec![],
            actions: vec![],
        },
    };

    let config = serde_yaml::to_string(&config)?;
    println!("{}", config);

    Ok(())
}
