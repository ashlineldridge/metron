mod parser;
mod profile;
mod root;
mod server;

use std::{
    ffi::OsString,
    fs::{self, File},
    io,
    time::Duration,
};

use anyhow::Context;
use either::Either::{Left, Right};
use serde::de::DeserializeOwned;
use thiserror::Error;
use url::Url;

use self::parser::RateArgValue;
use crate::{config, profile::PlanSegment, runtime};

#[derive(Error, Debug)]
#[non_exhaustive]
pub enum Error {
    #[error(transparent)]
    InvalidCli(#[from] clap::Error),

    #[error(transparent)]
    Unexpected(#[from] anyhow::Error),
}

/// Parses the CLI arguments into a [`Config`][config::Config] struct.
pub fn parse<I, T>(it: I) -> Result<config::Config, Error>
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    let matches = root::command().try_get_matches_from(it)?;

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

fn parse_profile_config(matches: &clap::ArgMatches) -> Result<crate::profile::Config, Error> {
    // Deserialize the config file if one was specified. Additional command line
    // options are then applied on top.
    let mut config = if let Some(config) = parse_config_file(matches)? {
        config
    } else {
        crate::profile::Config::default()
    };

    let rates = matches.get_many::<RateArgValue>("rate").unwrap();
    let durations = matches.get_many::<Option<Duration>>("duration").unwrap();

    if rates.len() != durations.len() {
        return Err(profile::command()
            .error(
                clap::ErrorKind::WrongNumberOfValues,
                "The number of --rate and --duration arguments must match",
            )
            .into());
    }

    let mut it = rates.zip(durations).peekable();
    while let Some((&rate, &duration)) = it.next() {
        // Check that only the last duration value is infinite.
        if duration.is_none() && it.peek().is_some() {
            return Err(profile::command()
                .error(
                    clap::ErrorKind::ValueValidation,
                    "Only the last --duration value can be \"forever\"",
                )
                .into());
        }

        let segment = match rate {
            Left(rate) => PlanSegment::Fixed { rate, duration },
            Right((rate_start, rate_end)) => {
                if let Some(duration) = duration {
                    PlanSegment::Linear {
                        rate_start,
                        rate_end,
                        duration,
                    }
                } else {
                    return Err(profile::command()
                        .error(
                            clap::ErrorKind::ValueValidation,
                            "Only fixed-rate segments may have a --duration value can be \"forever\"",
                        )
                        .into());
                }
            }
        };

        config.segments.push(segment);
    }

    config.connections = *matches.get_one::<u64>("connections").unwrap() as usize;
    config.http_method = *matches.get_one("http-method").unwrap();
    config.targets = matches
        .get_many::<Url>("target")
        .unwrap()
        .cloned()
        .collect::<Vec<_>>();

    config.headers = matches
        .get_many("header")
        .unwrap_or_default()
        .cloned()
        .collect();

    config.payload = if let Some(payload) = matches.get_one::<String>("payload") {
        Some(payload.to_owned())
    } else if let Some(file) = matches.get_one::<String>("payload-file") {
        let payload = fs::read_to_string(file).context("Error reading payload file")?;
        Some(payload)
    } else {
        None
    };

    config.runtime = parse_runtime_config(matches)?;

    config.signaller_kind = *matches.get_one("signaller").unwrap();
    config.no_latency_correction = *matches.get_one("no-latency-correction").unwrap();
    config.stop_on_client_error = *matches.get_one("stop-on-client-error").unwrap();
    config.stop_on_non_2xx = *matches.get_one("stop-on-non-2xx").unwrap();
    config.log_level = *matches.get_one("log-level").unwrap();

    // Ensure that we haven't been requested to create a single-threaded runtime with a
    // blocking signaller. This combination is not possible as the blocking signaller uses
    // a separate blocking thread to generate signal timing.
    if config.runtime.is_single_threaded() && config.signaller_kind.is_blocking() {
        return Err(profile::command()
            .error(
                clap::ErrorKind::ArgumentConflict,
                "Use of a single-threaded runtime is not compatible with a blocking signaller",
            )
            .into());
    }

    Ok(config)
}

fn parse_server_config(matches: &clap::ArgMatches) -> Result<crate::server::Config, Error> {
    // Deserialize the config file if one was specified. Additional command line
    // options are then applied on top.
    let mut config = if let Some(config) = parse_config_file(matches)? {
        config
    } else {
        crate::server::Config::default()
    };

    config.runtime = parse_runtime_config(matches)?;

    config.port = *matches.get_one("port").unwrap();
    config.log_level = *matches.get_one("log-level").unwrap();

    Ok(config)
}

fn parse_runtime_config(matches: &clap::ArgMatches) -> Result<runtime::Config, Error> {
    let config = if *matches.get_one("single-threaded").unwrap() {
        runtime::Config::SingleThreaded
    } else if let Some(worker_threads) = matches.get_one::<u64>("worker-threads") {
        runtime::Config::MultiThreaded {
            worker_threads: *worker_threads as usize,
        }
    } else {
        runtime::Config::default()
    };

    Ok(config)
}

/// Parses the config file if one has been specified.
fn parse_config_file<T>(matches: &clap::ArgMatches) -> Result<Option<T>, Error>
where
    T: DeserializeOwned,
{
    let config = match matches.get_one::<String>("config-file") {
        Some(f) if f == "-" => {
            let d = serde_yaml::from_reader(io::stdin())
                .context("Error parsing YAML configuration from stdin")?;
            Some(d)
        }
        Some(f) => {
            let file =
                File::open(f).context(format!("Error opening YAML configuration file: {}", f))?;
            let d = serde_yaml::from_reader(file)
                .context(format!("Error parsing YAML configuration file: {}", f))?;
            Some(d)
        }
        None => None,
    };

    Ok(config)
}

#[cfg(test)]
mod profile_tests {
    use clap::{
        error::{ContextKind, ContextValue},
        ErrorKind,
    };

    use super::*;

    #[test]
    fn minimal_required_args() {
        // Specify the minimal required arguments.
        let args = [
            "metron",
            "profile",
            "--rate=500",
            "--duration=30s",
            "--target=https://example.com",
        ];

        parse(args).unwrap();
    }

    #[test]
    fn missing_required_args() {
        // Invocation is missing all arguments so we expect an error to be returned. The
        // error should contain a report of the missing arguments.
        let args = ["metron", "profile"];

        let err = parse(args).unwrap_err();
        if let Error::InvalidCli(inner) = err {
            let (ctx_kind, ctx_value) = inner.context().next().unwrap();

            assert_eq!(inner.kind, ErrorKind::MissingRequiredArgument);
            assert_eq!(ctx_kind, ContextKind::InvalidArg);
            assert_eq!(
                ctx_value,
                &ContextValue::Strings(
                    vec![
                        "--rate <RATE>...",
                        "--duration <DURATION>...",
                        "--target <URL>...",
                    ]
                    .into_iter()
                    .map(ToString::to_string)
                    .collect()
                )
            );
        } else {
            panic!("Expected Error::InvalidCli error but got: {:?}", err);
        }
    }

    #[test]
    fn multi_segment_plan_single_arg() {
        // Specify a multi-segment test plan by passing a list of rates and a list of
        // duration values using a comma separator.
        let args = [
            "metron",
            "profile",
            "--rate=10:1000,1000",
            "--duration=5m,forever",
            "--target=https://example.com",
        ];

        parse(args).unwrap();
    }

    #[test]
    fn multi_segment_plan_multi_arg() {
        // Specify a multi-segment test plan by passing multiple rate and duration arguments.
        let args = [
            "metron",
            "profile",
            "--rate=10:1000",
            "--rate=1000",
            "--duration=5m",
            "--duration=forever",
            "--target=https://example.com",
        ];

        parse(args).unwrap();
    }

    #[test]
    fn multi_segment_plan_missing_duration_value() {
        // Specify a multi-segment test plan with a mismatching number of rates and durations.
        let args = [
            "metron",
            "profile",
            "--rate=10:1000,1000",
            "--duration=5m",
            "--target=https://example.com",
        ];

        let err = parse(args).unwrap_err();

        dbg!(&err);
    }
}
