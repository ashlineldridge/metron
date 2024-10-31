//! Entry point for the main `metron` binary.

use anyhow::Result;
use clap::Parser;
use metron_app::cli::{AgentCommand, Cli, Command};
use metron_config::*;
use tracing::info;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

// TODO: This is exiting early when CLI parse fails... maybe fine?
#[tokio::main]
async fn main() -> Result<()> {
    init_tracing()?;

    let cli = Cli::parse();
    match cli.command {
        Command::Test { config } => run_local_test(config).await?,
        Command::Agent { command } => match command {
            AgentCommand::Run { config } => run_agent_server(config).await?,
            AgentCommand::Test { config } => run_remote_test(config).await?,
            AgentCommand::Cancel { config } => cancel_remote_test(config).await?,
            AgentCommand::Report { config } => report_remote_test(config).await?,
            AgentCommand::Proxy { config } => run_proxy_server(config).await?,
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

async fn run_local_test(config: LocalTestConfig) -> Result<()> {
    info!("running local test with config: {:?}", config);
    Ok(())
}

async fn run_remote_test(config: RemoteTestConfig) -> Result<()> {
    info!("running remote test with config: {:?}", config);
    Ok(())
}

async fn run_agent_server(config: AgentConfig) -> Result<()> {
    info!("running agent with config: {:?}", config);
    Ok(())
}

async fn run_proxy_server(config: ProxyConfig) -> Result<()> {
    info!("running proxy with config: {:?}", config);
    Ok(())
}

async fn cancel_remote_test(config: CancelConfig) -> Result<()> {
    info!("running stop with config: {:?}", config);
    Ok(())
}

async fn report_remote_test(config: ReportConfig) -> Result<()> {
    info!("running report with config: {:?}", config);
    Ok(())
}
