mod profile;
mod root;
mod server;
mod validate;

use std::{
    fs::{self, File},
    io,
};

use anyhow::{bail, Context, Result};
use serde::de::DeserializeOwned;

use crate::{
    config,
    profile::{PlanSegment, SignallerKind},
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

    let rates = matches.values_of_t::<String>("rate")?;
    let durations = matches.values_of_t::<String>("duration")?;

    if rates.len() != durations.len() {
        return Err(profile::command()
            .error(
                clap::ErrorKind::WrongNumberOfValues,
                "The number of --rate and --duration arguments must match",
            )
            .into());

        // let mut cmd = clap::Command::new("foobar");
        // return Err(clap::Error::raw(
        //     clap::ErrorKind::Format,
        //     "The number of --rate and --duration arguments must match",
        // )
        // .format(&mut cmd)
        // .into());
        // bail!("The number of --rate and --duration arguments must match");
    }

    for (rate, duration) in rates.into_iter().zip(durations) {
        let duration = if duration == "forever" {
            None
        } else {
            let duration = duration.parse::<humantime::Duration>()?;
            Some(duration.into())
        };

        let segment = if let Some((rate_start, rate_end)) = rate.split_once(':') {
            if let Some(duration) = duration {
                let rate_start = rate_start.parse()?;
                let rate_end = rate_end.parse()?;
                PlanSegment::Linear {
                    rate_start,
                    rate_end,
                    duration,
                }
            } else {
                bail!("A finite duration must be used when the rate varies over time");
            }
        } else {
            let rate = rate.parse()?;
            PlanSegment::Fixed { rate, duration }
        };

        config.segments.push(segment);
    }

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
