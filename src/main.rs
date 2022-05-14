#![feature(let_chains)]

mod cli;
mod config;
mod profile;
mod runtime;
mod server;
mod wait;

use std::process;

use anyhow::Result;
use config::Config;
use metron::LogLevel;

use crate::profile::Profiler;

fn main() {
    if let Err(err) = try_main() {
        eprintln!("{}", err);
        process::exit(2);
    }
}

fn try_main() -> Result<()> {
    let config = crate::cli::parse()?;

    init_logging(config.log_level());

    let runtime = runtime::build(config.runtime())?;
    let _guard = runtime.enter();

    let handle = tokio::spawn(async move {
        match config {
            Config::Load(config) => {
                let profiler = Profiler::new(config);
                let report = profiler.run().await?;
                println!("{:#?}", report);
            }
            Config::Server(config) => {
                server::run(&config)?;
            }
        }

        Result::<(), anyhow::Error>::Ok(())
    });

    runtime.block_on(handle)??;

    Ok(())
}

fn init_logging(level: LogLevel) {
    env_logger::builder().filter_level(level).init();
}
