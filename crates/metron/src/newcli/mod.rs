use std::ffi::OsString;

use thiserror::Error;

use crate::config;

#[derive(Error, Debug)]
#[non_exhaustive]
pub enum Error {
    #[error(transparent)]
    InvalidCli(#[from] clap::Error),

    #[error(transparent)]
    Unexpected(#[from] anyhow::Error),
}

const ABOUT: &str = "
Metron is a modern L7 performance profiler.

Use --help for more details.

Project home: https://github.com/ashlineldridge/metron
";

/// Parses the CLI arguments into a [`Config`][config::Config] struct.
pub fn parse<I, T>(it: I) -> Result<config::Config, Error>
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    let matches = command().try_get_matches_from(it)?;

    // Construct the config based on the provided subcommand. We use `unwrap` and
    // `panic!` as if we were to encounter these it'd mean we've misconfigured clap.
    let subcommand = matches.subcommand().unwrap();
    let config = match subcommand {
        ("echo", matches) => config::Config::Echo(parse_echo_config(matches)?),
        ("node", matches) => config::Config::Node(parse_node_config(matches)?),
        ("profile", matches) => config::Config::Profile(parse_profile_config(matches)?),
        ("control", matches) => config::Config::Control(parse_control_config(matches)?),
        _ => panic!("Unknown subcommand"),
    };

    Ok(config)
}

pub fn command() -> clap::Command {
    use clap::*;

    Command::new("metron")
        .author(crate_authors!())
        .version(crate_version!())
        .about(ABOUT)
        .arg_required_else_help(true)
        .subcommand_required(true)
        .subcommand(
        )
}

// TODO: Start bringing args over here and adding them.
