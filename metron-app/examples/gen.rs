use std::{
    fs::File,
    io::{BufWriter, Write},
    path::{Path, PathBuf},
    time::Duration,
};

use anyhow::Result;
use metron_config::*;
use metron_core::Plan;
use serde::ser;

/// Generate example config files and save them under the top-level examples directory.
fn main() -> Result<()> {
    let local_test_config = LocalTestConfig {
        plan: Plan {
            segments: vec![],
            actions: vec![],
        },
        signaller: Some(SignallerKind::Dedicated),
        worker_threads: Some(num_cpus::get()),
        logging: Some(LoggingConfig {
            level: LogLevel::Debug,
            format: LogFormat::Bunyan,
        }),
        prometheus: Some(PrometheusConfig {
            port: 8081,
            path: "/metrics".to_owned(),
        }),
        open_telemetry: Some(OpenTelemetryConfig {
            address: url::Url::parse("http://localhost:8989")?,
            period: Duration::from_secs(60),
            timeout: Duration::from_secs(60),
        }),
    };
    let remote_agents = vec![RemoteAgentDiscovery::Static(StaticAgentDiscovery {
        endpoints: vec![
            "198.120.113.0:8080".to_owned(),
            "foo.bar.com:8080".to_owned(),
        ],
    })];
    let remote_test_config = RemoteTestConfig {
        plan: Plan {
            segments: vec![],
            actions: vec![],
        },
        agents: remote_agents.clone(),
    };
    let agent_config = AgentConfig {
        name: Some("remote-0".to_owned()),
        signaller: Some(SignallerKind::Dedicated),
        worker_threads: Some(num_cpus::get()),
        logging: Some(LoggingConfig {
            level: LogLevel::Debug,
            format: LogFormat::Bunyan,
        }),
        prometheus: Some(PrometheusConfig {
            port: 8081,
            path: "/metrics".to_owned(),
        }),
        open_telemetry: Some(OpenTelemetryConfig {
            address: url::Url::parse("http://localhost:8989")?,
            period: Duration::from_secs(60),
            timeout: Duration::from_secs(60),
        }),
    };
    let cancel_config = CancelConfig {
        agents: remote_agents.clone(),
    };
    let report_config = CancelConfig {
        agents: remote_agents.clone(),
    };
    let proxy_config = ProxyConfig {
        agents: remote_agents.clone(),
    };

    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.pop();
    path.push("examples");

    save_config(&local_test_config, path.join("local-test-config.yaml"))?;
    save_config(&remote_test_config, path.join("remote-test-config.yaml"))?;
    save_config(&agent_config, path.join("agent-config.yaml"))?;
    save_config(&cancel_config, path.join("cancel-config.yaml"))?;
    save_config(&report_config, path.join("report-config.yaml"))?;
    save_config(&proxy_config, path.join("proxy-config.yaml"))?;

    Ok(())
}

fn save_config<T: ?Sized + ser::Serialize, P: AsRef<Path>>(config: &T, path: P) -> Result<()> {
    println!("Saving {}", path.as_ref().to_str().unwrap());
    let mut writer = BufWriter::new(File::create(path)?);
    let data = serde_yaml::to_string(config)?;
    writer.write_all(data.trim().as_bytes())?;

    Ok(())
}
