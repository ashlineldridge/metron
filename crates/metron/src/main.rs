#![feature(let_chains)]

mod cli;
mod config;
mod control;
mod echo;
mod newcli;
mod node;
mod profile;
mod runtime;
mod wait;

use std::env;

use anyhow::{Context, Result};
use config::Config;

use crate::profile::Profiler;

/// Application entry point.
fn main() -> Result<()> {
    let config = match cli::parse(env::args_os()) {
        Err(cli::Error::InvalidCli(err)) => err.exit(),
        v => v,
    }?;

    env_logger::builder()
        .filter_level(config.log_level().into())
        .init();

    let runtime = runtime::build(config.runtime())?;
    let _guard = runtime.enter();

    let handle = tokio::spawn(async move {
        match config {
            Config::Echo(config) => run_echo_server(&config).await,
            Config::Node(config) => run_node(&config).await,
            Config::Profile(config) => run_profile_test(&config).await,
            Config::Control(config) => run_control_command(&config).await,
        }
    });

    runtime.block_on(handle)??;

    Ok(())
}

async fn run_echo_server(config: &echo::Config) -> Result<()> {
    echo::serve(config).await
}

async fn run_node(_config: &node::Config) -> Result<()> {
    println!("Running Metron node");
    Ok(())
}

async fn run_profile_test(config: &profile::Config) -> Result<()> {
    let profiler = Profiler::new(config.clone());
    let report = profiler.run().await;
    match report {
        Ok(ref report) => print_report(report)?,
        Err(ref err) => {
            if let Some(report) = err.partial_report() {
                print_report(report)?;
            }
        }
    }

    report
        .map(|_| ())
        .context("Profiling operation was aborted due to error")
}

async fn run_control_command(_config: &control::Config) -> Result<()> {
    println!("Running Metron control command");
    Ok(())
}

fn print_report(report: &profile::Report) -> Result<()> {
    println!("{}", serde_yaml::to_string(report)?);
    Ok(())
}
