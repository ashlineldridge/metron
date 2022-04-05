mod cli;
mod config;
mod error;
mod load;
mod server;
mod wait;

use std::process;

use anyhow::Result;
use config::Config;
use wrkr::LogLevel;

fn main() {
    if let Err(e) = try_main() {
        // TODO: Need proper error handling
        // The underlying clap error
        println!("{}", e);
        process::exit(1);
    }
}

fn try_main() -> Result<()> {
    let config = crate::cli::parse()?;

    init_logging(config.log_level());

    match config {
        Config::Load(config) => {
            let report = crate::load::run(&config)?;
            println!("{:?}", report);
        }
        Config::Server(config) => {
            crate::server::run(&config)?;
        }
    }

    Ok(())
}

fn init_logging(level: LogLevel) {
    env_logger::builder().filter_level(level).init();
}
