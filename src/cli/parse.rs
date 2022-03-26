use std::process;
use std::{fmt::Display, str::FromStr};

use super::root;

/// Parses the CLI arguments into a [`Config`][crate::config::Config] struct.
///
/// This function will exit and print an appropriate help message if the
/// supplied command-line arguments are invalid. The returned [clap::ArgMatches]
/// is guaranteed to be valid (anything less should be considered a bug).
pub fn root_config() -> crate::config::Config {
    let matches = root::command().get_matches();

    // Safe to unwrap...
    let subcommand = matches.subcommand().unwrap();

    match subcommand {
        ("load", matches) => load_config(matches),
        ("server", matches) => server_config(matches),
        _ => panic!("Invalid subcommand"),
    }
}

fn load_config(matches: &clap::ArgMatches) -> crate::config::Config {
    crate::config::Config::Load(crate::load::Config {
        blocks: todo!(),
        connections: todo!(),
        http_method: todo!(),
        targets: todo!(),
        headers: todo!(),
        payload: todo!(),
        worker_threads: todo!(),
        signaller_kind: todo!(),
        log_level: todo!(),
    })
}

fn server_config(matches: &clap::ArgMatches) -> crate::config::Config {
    crate::config::Config::Server(crate::server::Config {
        port: todo!(),
        worker_threads: todo!(),
        log_level: todo!(),
    })
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
