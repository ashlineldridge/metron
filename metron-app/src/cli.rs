use clap::{Parser, Subcommand};
use metron_config::{AgentConfig, PollConfig, TestConfig};
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
    /// Run an agent
    Agent {
        /// Agent config file (use '-' for stdin)
        #[arg(short = 'f', long = "file", value_parser = config_file::<AgentConfig>, value_name = "FILE")]
        config: AgentConfig,
    },
    /// Poll the results of a load test
    Poll {
        /// Poll config file (use '-' for stdin)
        #[arg(short = 'f', long = "file", value_parser = config_file::<PollConfig>, value_name = "FILE")]
        config: PollConfig,
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
