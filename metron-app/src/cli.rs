// TODO(VERY_NEXT): Build a simple derive API that just supports config files for now.
// https://www.reddit.com/r/rust/comments/vf3gjf/comment/icu0wzc/?utm_source=share&utm_medium=web3x&utm_name=web3xcss&utm_term=1&utm_content=share_button

// TODOs:
// Use https://crates.io/crates/clio

use clap::{Parser, Subcommand};
use metron_config::{AgentConfig, PollConfig, TestConfig};
use serde::de::DeserializeOwned;

use crate::experimental;

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
    /// Experimental features
    #[command(subcommand)]
    Experimental(experimental::Command),
}

// TODO:
// Try layer below ontop of clio value_parser so that the above
// can be the parsed type?

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
