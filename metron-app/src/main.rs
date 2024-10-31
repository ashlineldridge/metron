//! Entry point for the main `metron` binary.

use anyhow::Result;
use clap::Parser;
use metron_app::cli::{Cli, Command};
use metron_config::{AgentConfig, PollConfig, TestConfig};
use tracing::info;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

// TODO: This is exiting early when CLI parse fails... maybe fine?
#[tokio::main]
async fn main() -> Result<()> {
    init_tracing()?;

    let cli = Cli::parse();
    match cli.command {
        Command::Test { config } => run_test(config).await?,
        Command::Agent { config } => run_agent(config).await?,
        Command::Poll { config } => run_poll(config).await?,
    }

    Ok(())
}

fn init_tracing() -> Result<()> {
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env())
        .try_init()?;

    Ok(())
}

async fn run_test(config: TestConfig) -> Result<()> {
    info!("running test with config: {:?}", config);
    Ok(())
}

async fn run_agent(config: AgentConfig) -> Result<()> {
    info!("running agent with config: {:?}", config);
    Ok(())
}

async fn run_poll(config: PollConfig) -> Result<()> {
    info!("running poll with config: {:?}", config);
    Ok(())
}
