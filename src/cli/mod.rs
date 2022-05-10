mod profile;
mod root;
mod server;
mod validate;

use anyhow::Result;
use std::fs;
use metron::Rate;

use crate::profile::RateBlock;

/// Parses the CLI arguments into a [`Config`][crate::config::Config] struct.
///
/// This function will exit and print an appropriate help message if the
/// supplied command-line arguments are invalid. The returned [clap::ArgMatches]
/// is guaranteed to be valid (anything less should be considered a bug).
pub fn parse() -> Result<crate::config::Config> {
    let matches = root::command().try_get_matches()?;

    // Construct the config based on the provided subcommand. We use `unwrap` and
    // `panic!` as if we were to encounter these it'd mean we've misconfigured clap.
    let subcommand = matches.subcommand().unwrap();
    let config = match subcommand {
        ("profile", matches) => parse_profile_config(matches),
        ("server", matches) => parse_server_config(matches),
        _ => panic!("Unknown subcommand"),
    };

    Ok(config)
}

fn parse_profile_config(matches: &clap::ArgMatches) -> crate::config::Config {
    let mut blocks = vec![];

    // Add a linear ramp block if requested.
    if matches.is_present("group-ramp") {
        blocks.push(RateBlock::Linear(
            matches.value_of_t_or_exit::<Rate>("ramp-rate-start"),
            matches.value_of_t_or_exit::<Rate>("rate-rate-end"),
            matches
                .value_of_t_or_exit::<humantime::Duration>("ramp-duration")
                .into(),
        ));
    }

    // Add the fixed rate block.
    let rate = matches.value_of_t_or_exit::<Rate>("rate");
    let duration = if matches.is_present("duration") {
        Some(
            matches
                .value_of_t_or_exit::<humantime::Duration>("duration")
                .into(),
        )
    } else {
        None
    };

    blocks.push(RateBlock::Fixed(rate, duration));

    let connections = matches.value_of_t_or_exit("connections");
    let http_method = matches.value_of_t_or_exit("http-method");

    let targets = if matches.is_present("target") {
        vec![matches.value_of_t_or_exit("target")]
    } else {
        matches.values_of_t_or_exit("multi-target")
    };

    let headers = if matches.is_present("header") {
        matches.values_of_t_or_exit("header")
    } else {
        vec![]
    };

    let payload = if matches.is_present("payload") {
        Some(matches.value_of_t_or_exit("payload"))
    } else if matches.is_present("payload-file") {
        let file = matches.value_of_t_or_exit::<String>("payload-file");
        let payload = fs::read_to_string(file).unwrap();
        Some(payload)
    } else {
        None
    };

    let worker_threads = if matches.is_present("worker-threads") {
        Some(matches.value_of_t_or_exit("worker-threads"))
    } else {
        None
    };

    let signaller_kind = matches.value_of_t_or_exit("signaller");

    let log_level = matches.value_of_t_or_exit("log-level");

    crate::config::Config::Load(crate::profile::Config {
        blocks,
        connections,
        http_method,
        targets,
        headers,
        payload,
        worker_threads,
        signaller_kind,
        log_level,
    })
}

fn parse_server_config(matches: &clap::ArgMatches) -> crate::config::Config {
    let port = matches.value_of_t_or_exit("port");

    let worker_threads = if matches.is_present("worker-threads") {
        Some(matches.value_of_t_or_exit("worker-threads"))
    } else {
        None
    };

    let log_level = matches.value_of_t_or_exit("log-level");

    crate::config::Config::Server(crate::server::Config {
        port,
        worker_threads,
        log_level,
    })
}
