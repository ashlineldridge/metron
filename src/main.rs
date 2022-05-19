#![feature(let_chains)]

mod cli;
mod config;
mod profile;
mod runtime;
mod server;
mod wait;

use anyhow::{Context, Result};
use config::Config;
use metron::LogLevel;

use crate::profile::Profiler;

fn main() -> Result<()> {
    // For now, just let the anyhow hook print the errors.
    // The error printing commented out below doesn't print the error chain.
    // if let Err(err) = try_main() {
    //     eprintln!("{}", err);
    //     process::exit(2);
    // }
    try_main()
}

fn try_main() -> Result<()> {
    let config = crate::cli::parse()?;

    init_logging(config.log_level());

    let runtime = runtime::build(config.runtime())?;
    let _guard = runtime.enter();

    let handle = tokio::spawn(async move {
        match config {
            Config::Profile(config) => run_profile(config).await,
            Config::Server(config) => run_server(config).await,
        }
    });

    runtime.block_on(handle)??;

    Ok(())
}

async fn run_profile(config: profile::Config) -> Result<()> {
    let profiler = Profiler::new(config);
    let report = profiler.run().await;
    match report {
        Ok(ref report) => print_report(report),
        Err(ref err) => {
            if let Some(report) = err.partial_report() {
                print_report(report);
            }
        }
    }

    report
        .map(|_| ())
        .context("Profiling operation was aborted due to error")
}

async fn run_server(config: server::Config) -> Result<()> {
    server::run(&config)
}

fn print_report(report: &profile::Report) {
    println!("{}\n", report);
}

fn init_logging(level: LogLevel) {
    env_logger::builder().filter_level(level).init();
}
