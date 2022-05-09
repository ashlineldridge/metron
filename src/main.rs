mod cli;
mod config;
mod load;
mod server;
mod wait;

use anyhow::Result;
use config::Config;
use wrkr::LogLevel;

fn main() -> Result<()> {
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
