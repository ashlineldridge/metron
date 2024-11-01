#![feature(let_chains)]

//! Entry point for the main `metron` binary.

use anyhow::Result;
use clap::Parser;
use metron_app::{
    app,
    cli::{AgentCommand, Cli, Command},
};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

// TODO: This is exiting early when CLI parse fails... maybe fine?
#[tokio::main]
async fn main() -> Result<()> {
    init_tracing()?;

    let cli = Cli::parse();
    match cli.command {
        Command::Test { config } => app::run_local_test(config).await?,
        Command::Agent { command } => match command {
            AgentCommand::Run { config } => app::run_agent_server(config).await?,
            AgentCommand::Test { config } => app::run_remote_test(config).await?,
            AgentCommand::Cancel { config } => app::cancel_remote_test(config).await?,
            AgentCommand::Report { config } => app::report_remote_test(config).await?,
            AgentCommand::Proxy { config } => app::run_proxy_server(config).await?,
        },
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
