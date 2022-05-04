mod cli;
mod config;
mod load;
mod server;
mod wait;

use config::Config;
use wrkr::LogLevel;

fn main() {
    if let Err(e) = try_main() {
        // TODO: Need proper error handling
        // The underlying clap error
        println!("{}", e);
        std::process::exit(1);
    }

    // See https://docs.rs/snafu/0.7.0/snafu/guide/examples/backtrace/enum.Error.html
    // for example of printing error and backtrace.
}

fn try_main() -> Result<(), anyhow::Error> {
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
