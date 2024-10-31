use clap::{Parser, Subcommand};
use metron_config::{AgentConfig, ProxyConfig, ReportConfig, StopConfig, TestConfig};
use serde::de::DeserializeOwned;

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Clone, Debug, Subcommand)]
pub enum Command {
    // TODO: Can make these tuple types?
    /// Run a load test
    Test {
        /// Test config file (use '-' for stdin)
        #[arg(short = 'f', long = "file", value_parser = config_file::<TestConfig>, value_name = "FILE")]
        config: TestConfig,
    },
    /// Run an agent server
    Agent {
        /// Agent config file (use '-' for stdin)
        #[arg(short = 'f', long = "file", value_parser = config_file::<AgentConfig>, value_name = "FILE")]
        config: AgentConfig,
    },
    /// Stop a load test (running on agents)
    Stop {
        /// Stop config file (use '-' for stdin)
        #[arg(short = 'f', long = "file", value_parser = config_file::<StopConfig>, value_name = "FILE")]
        config: StopConfig,
    },
    /// Print a report of test results (retrieved from agents)
    Report {
        /// Report config file (use '-' for stdin)
        #[arg(short = 'f', long = "file", value_parser = config_file::<ReportConfig>, value_name = "FILE")]
        config: ReportConfig,
    },
    /// Run a proxy server (to balance load between agents)
    Proxy {
        /// Proxy config file (use '-' for stdin)
        #[arg(short = 'f', long = "file", value_parser = config_file::<ProxyConfig>, value_name = "FILE")]
        config: ProxyConfig,
    },
}

/// Config file clap [`Arg::value_parser`][clap::Arg::value_parser].
fn config_file<T>(value: &str) -> anyhow::Result<T>
where
    T: DeserializeOwned,
{
    let config = if value == "-" {
        serde_yaml::from_reader(std::io::stdin())?
    } else {
        let file = std::fs::File::open(value)?;
        serde_yaml::from_reader(file)?
    };

    Ok(config)
}
