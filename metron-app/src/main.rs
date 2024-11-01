#![feature(let_chains)]

//! Entry point for the main `metron` binary.

use anyhow::{Context, Result};
use clap::Parser;
use metron_app::cli::{AgentCommand, Cli, Command};
use metron_config::*;
use metron_core::{Agent, Runner};
use metron_grpc::AgentClient;
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
    info!("running local test");

    let mut runner = Runner {
        name: "local".to_owned(),
        signaller: config.signaller.unwrap_or_default().into(),
        worker_threads: config.worker_threads.unwrap_or_default(),
        sinks: config.sinks,
    };

    runner.test(&config.plan).await?;

    Ok(())
}

async fn run_remote_test(config: RemoteTestConfig) -> Result<()> {
    info!("running remote test");

    let address = hack_single_agent_address(&config.agents)?;
    let mut agent = AgentClient::connect(address).await?;
    agent.test(&config.plan).await?;

    Ok(())
}

async fn run_agent_server(_config: AgentConfig) -> Result<()> {
    info!("running agent server");

    // TODO...

    Ok(())
}

async fn run_proxy_server(_config: ProxyConfig) -> Result<()> {
    info!("running proxy server");

    // TODO...

    Ok(())
}

async fn cancel_remote_test(config: CancelConfig) -> Result<()> {
    info!("cancelling any running test");

    let address = hack_single_agent_address(&config.agents)?;
    let mut agent = AgentClient::connect(address).await?;
    agent.cancel().await?;

    Ok(())
}

async fn report_remote_test(config: ReportConfig) -> Result<()> {
    info!("requesting load test report");

    let address = hack_single_agent_address(&config.agents)?;
    let mut agent = AgentClient::connect(address).await?;
    let report = agent.report().await?;

    println!("got report: {:?}", report);

    Ok(())
}

// HACK: Just use the first endpoint for now.
fn hack_single_agent_address(agents: &[RemoteAgentDiscovery]) -> Result<String> {
    let address = agents
        .iter()
        .find_map(|d| {
            if let RemoteAgentDiscovery::Static(d) = d
                && let Some(endpoint) = d.endpoints.first()
            {
                return Some(endpoint.to_owned());
            }
            None
        })
        .context("no static agent endpoint found")?;

    Ok(address)
}
