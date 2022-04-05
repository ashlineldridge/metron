use anyhow::Result;
use std::process;
use std::{fmt::Display, str::FromStr};
use wrkr::Rate;

use crate::load::RateBlock;

use super::root;

/// Parses the CLI arguments into a [`Config`][crate::config::Config] struct.
///
/// This function will exit and print an appropriate help message if the
/// supplied command-line arguments are invalid. The returned [clap::ArgMatches]
/// is guaranteed to be valid (anything less should be considered a bug).
pub fn root_config() -> Result<crate::config::Config> {
    let matches = root::command().try_get_matches()?;

    // Construct the config based on the provided subcommand. We use `unwrap` and
    // `panic!` as if we were to encounter these it'd mean we've misconfigured clap.
    let subcommand = matches.subcommand().unwrap();
    let config = match subcommand {
        ("load", matches) => load_config(matches)?,
        ("server", matches) => server_config(matches)?,
        _ => panic!("Unknown subcommand"),
    };

    Ok(config)
}

fn load_config(matches: &clap::ArgMatches) -> Result<crate::config::Config> {
    let mut blocks = vec![];

    // Add a linear ramp block if requested.
    if matches.is_present("group-ramp") {
        blocks.push(RateBlock::Linear(
            matches.value_of_t::<Rate>("ramp-rate-start")?,
            matches.value_of_t::<Rate>("rate-rate-end")?,
            matches
                .value_of_t::<humantime::Duration>("ramp-duration")
                .map(|d| d.into())?,
        ));
    }

    let duration = if matches.is_present("duration") {
        Some(
            matches
                .value_of_t::<humantime::Duration>("duration")
                .map(|d| d.into())?,
        )
    } else {
        None
    };

    let rate = matches.value_of("rate");
    let duration = matches.value_of("duration");

    if rate.is_some() {}

    // let rate = matches.value_of("rate");
    // let duration = matches.value_of("duration");
    // blocks.push(RateBlock::Fixed())

    let config = crate::config::Config::Load(crate::load::Config {
        blocks,
        connections: todo!(),
        http_method: todo!(),
        targets: todo!(),
        headers: todo!(),
        payload: todo!(),
        worker_threads: todo!(),
        signaller_kind: todo!(),
        log_level: todo!(),
    });

    Ok(config)
}

fn server_config(matches: &clap::ArgMatches) -> Result<crate::config::Config> {
    let config = crate::config::Config::Server(crate::server::Config {
        port: todo!(),
        worker_threads: todo!(),
        log_level: todo!(),
    });

    Ok(config)
}

fn parse_or_exit<T>(s: &str) -> T
where
    T: FromStr,
    T::Err: Display,
{
    T::from_str(s).unwrap_or_else(|e| {
        eprintln!("Error: {}", e);
        process::exit(1);
    })
}
