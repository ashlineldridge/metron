use clap::{Parser, Subcommand};
use metron_config::*;
use serde::de::DeserializeOwned;

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Clone, Debug, Subcommand)]
pub enum Command {
    /// Run a local load test
    Test {
        /// Test config file (use '-' for stdin)
        #[arg(short = 'f', long = "file", value_parser = config_file::<LocalTestConfig>, value_name = "FILE")]
        config: LocalTestConfig,
    },
    /// Run agent commands
    Agent {
        #[command(subcommand)]
        command: AgentCommand,
    },
}

#[derive(Clone, Debug, Subcommand)]
pub enum AgentCommand {
    /// Run an agent server
    Run {
        /// Agent config file (use '-' for stdin)
        #[arg(short = 'f', long = "file", value_parser = config_file::<AgentConfig>, value_name = "FILE")]
        config: AgentConfig,
    },
    /// Run a remote load test
    Test {
        /// Test config file (use '-' for stdin)
        #[arg(short = 'f', long = "file", value_parser = config_file::<RemoteTestConfig>, value_name = "FILE")]
        config: RemoteTestConfig,
    },
    /// Cancel a remote load test
    Cancel {
        /// Stop config file (use '-' for stdin)
        #[arg(short = 'f', long = "file", value_parser = config_file::<CancelConfig>, value_name = "FILE")]
        config: CancelConfig,
    },
    /// Print a test report
    Report {
        /// Report config file (use '-' for stdin)
        #[arg(short = 'f', long = "file", value_parser = config_file::<ReportConfig>, value_name = "FILE")]
        config: ReportConfig,
    },
    /// Run an agent proxy server
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
