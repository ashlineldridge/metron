#![feature(let_chains)]

//! Entry point for the main `metron` binary.

use anyhow::{Context, Result};
use clap::Parser;
use metron_app::{
    app,
    cli::{AgentCommand, Cli, Command},
};
use tracing::info;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

// TODO: This is exiting early when CLI parse fails... maybe fine?
fn main() -> Result<()> {
    init_tracing()?;

    info!("running metron app");

    let cli = Cli::parse();
    let runtime = new_runtime(&cli)?;
    let _guard = runtime.enter();

    let handle = tokio::spawn(async move {
        match cli.command {
            Command::Test { config } => app::run_local_test(config).await,
            Command::Agent { command } => match command {
                AgentCommand::Run { config } => app::run_agent_server(config).await,
                AgentCommand::Test { config } => app::run_remote_test(config).await,
                AgentCommand::Cancel { config } => app::cancel_remote_test(config).await,
                AgentCommand::Report { config } => app::report_remote_test(config).await,
                AgentCommand::Proxy { config } => app::run_proxy_server(config).await,
            },
        }
    });

    info!("blocking on app handle");

    runtime.block_on(handle)??;

    info!("done");

    Ok(())
}

fn init_tracing() -> Result<()> {
    tracing_subscriber::registry()
        .with(fmt::layer().with_thread_ids(true).with_thread_names(true))
        .with(EnvFilter::from_default_env())
        .try_init()?;

    Ok(())
}

fn new_runtime(cli: &Cli) -> Result<tokio::runtime::Runtime> {
    let worker_threads = match &cli.command {
        Command::Test { config } => config.worker_threads,
        Command::Agent { command } => match command {
            AgentCommand::Run { config } => config.worker_threads,
            _ => None,
        },
    };
    let worker_threads = worker_threads.unwrap_or(num_cpus::get());

    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(worker_threads)
        .enable_all()
        .build()
        .context("could not build tokio runtime")?;

    Ok(runtime)
}
