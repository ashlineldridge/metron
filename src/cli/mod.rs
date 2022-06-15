mod profile;
mod root;
mod server;
mod validate;

use std::{
    fs::{self, File},
    io,
};

use anyhow::{Context, Result};
use metron::Rate;
use serde::de::DeserializeOwned;

use crate::{
    config,
    profile::{RateBlock, SignallerKind},
    runtime,
};

/// Parses the CLI arguments into a [`Config`][config::Config] struct.
///
/// This function will exit and print an appropriate help message if the
/// supplied command-line arguments are invalid. The returned [clap::ArgMatches]
/// is guaranteed to be valid (anything less should be considered a bug).
pub fn parse() -> Result<config::Config> {
    let matches = root::command().try_get_matches()?;

    // Construct the config based on the provided subcommand. We use `unwrap` and
    // `panic!` as if we were to encounter these it'd mean we've misconfigured clap.
    let subcommand = matches.subcommand().unwrap();
    let config = match subcommand {
        ("profile", matches) => config::Config::Profile(parse_profile_config(matches)?),
        ("server", matches) => config::Config::Server(parse_server_config(matches)?),
        _ => panic!("Unknown subcommand"),
    };

    Ok(config)
}

fn parse_profile_config(matches: &clap::ArgMatches) -> Result<crate::profile::Config> {
    // Deserialize the config file if one was specified. Additional command line
    // options are then applied on top.
    let mut config = if let Some(config) = parse_config_file(matches)? {
        config
    } else {
        crate::profile::Config::default()
    };

    // Add a linear ramp block if requested.
    if matches.is_present("group-ramp") {
        let rate_start = matches.value_of_t_or_exit::<Rate>("ramp-rate-start");
        let rate_end = matches.value_of_t_or_exit::<Rate>("rate-rate-end");
        let duration = matches
            .value_of_t_or_exit::<humantime::Duration>("ramp-duration")
            .into();

        config.blocks.push(RateBlock::Linear {
            rate_start,
            rate_end,
            duration,
        });
    }

    // Add the fixed rate block.
    let rate = matches.value_of_t_or_exit::<Rate>("rate");
    let duration = if matches.is_present("duration") {
        let duration = matches
            .value_of_t_or_exit::<humantime::Duration>("duration")
            .into();
        Some(duration)
    } else {
        None
    };

    config.blocks.push(RateBlock::Fixed { rate, duration });

    config.connections = matches.value_of_t_or_exit("connections");
    config.http_method = matches.value_of_t_or_exit("http-method");
    config.targets = if matches.is_present("target") {
        vec![matches.value_of_t_or_exit("target")]
    } else {
        matches.values_of_t_or_exit("multi-target")
    };

    config.headers = if matches.is_present("header") {
        matches.values_of_t_or_exit("header")
    } else {
        vec![]
    };

    config.payload = if matches.is_present("payload") {
        Some(matches.value_of_t_or_exit("payload"))
    } else if matches.is_present("payload-file") {
        let file = matches.value_of_t_or_exit::<String>("payload-file");
        let payload = fs::read_to_string(file).unwrap();
        Some(payload)
    } else {
        None
    };

    config.runtime = parse_runtime_config(matches)?;

    config.signaller_kind = match matches.value_of("signaller").unwrap() {
        "blocking" => SignallerKind::Blocking,
        "cooperative" => SignallerKind::Cooperative,
        s => panic!("Invalid signaller: {}", s),
    };

    config.stop_on_error = matches.value_of_t_or_exit("stop-on-error");
    config.stop_on_non_2xx = matches.value_of_t_or_exit("stop-on-non-2xx");
    config.log_level = matches.value_of_t_or_exit("log-level");

    Ok(config)
}

fn parse_server_config(matches: &clap::ArgMatches) -> Result<crate::server::Config> {
    // Deserialize the config file if one was specified. Additional command line
    // options are then applied on top.
    let mut config = if let Some(config) = parse_config_file(matches)? {
        config
    } else {
        crate::server::Config::default()
    };

    config.runtime = parse_runtime_config(matches)?;

    config.port = matches.value_of_t_or_exit("port");
    config.log_level = matches.value_of_t_or_exit("log-level");

    Ok(config)
}

fn parse_runtime_config(matches: &clap::ArgMatches) -> Result<runtime::Config> {
    let config = if matches.is_present("worker-threads") {
        let worker_threads = matches.value_of_t_or_exit("worker-threads");
        runtime::Config { worker_threads }
    } else {
        Default::default()
    };

    Ok(config)
}

/// Parses the config file if one has been specified.
fn parse_config_file<T>(matches: &clap::ArgMatches) -> Result<Option<T>>
where
    T: DeserializeOwned,
{
    let config = match matches.value_of("config-file") {
        Some(f) if f == "-" => {
            let d = serde_yaml::from_reader(io::stdin())
                .context("Error parsing YAML configuration from stdin")?;
            Some(d)
        }
        Some(f) => {
            let d = serde_yaml::from_reader(File::open(f)?)
                .context(format!("Error parsing YAML configuration file: {}", f))?;
            Some(d)
        }
        None => None,
    };

    Ok(config)
}
