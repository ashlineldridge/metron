mod cli;
mod config;
mod profile;
mod server;
mod wait;

use anyhow::Result;
use config::Config;
use metron::LogLevel;

fn main() -> Result<()> {
    let config = crate::cli::parse()?;

    init_logging(config.log_level());

    match config {
        Config::Load(config) => {
            let report = crate::profile::run(&config)?;
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
