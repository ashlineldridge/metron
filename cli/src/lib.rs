//! CLI resources used by the 'metron` binary.

mod controller;
mod parser;
mod root;
mod runner;
mod test;

use std::{ffi::OsString, fmt::Display};

use clap::error::ErrorKind;
use metron::{ControllerConfig, LoadTestConfig, RunnerConfig};
use thiserror::Error;

pub(crate) const BAD_CLAP: &str = "clap has been misconfigured";
pub(crate) const BAD_SERDE: &str = "serde has been misconfigured";

pub fn parse<I, T>(it: I) -> Result<ParsedCli, InvalidArgsError>
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    // Extract the matches, handling for the fact that clap returns an error
    // when the user passes --help / -h or --version / -V.
    let matches = match root::command().try_get_matches_from(it) {
        Ok(m) => m,
        Err(e) => {
            let msg = e.render().to_string();
            if e.kind() == ErrorKind::DisplayHelp || e.kind() == ErrorKind::DisplayVersion {
                return Ok(ParsedCli::Help(msg));
            } else {
                return Err(InvalidArgsError(msg));
            }
        }
    };

    let (command, matches) = matches.subcommand().expect(BAD_CLAP);
    let print_config = *matches.get_one("print-config").unwrap_or(&false);

    let result = match command {
        "test" => {
            let config = test::parse_args(matches).expect(BAD_CLAP);
            if print_config {
                let text = serde_yaml::to_string(&config).expect(BAD_SERDE);
                ParsedCli::PrintConfig(text)
            } else {
                ParsedCli::LoadTest(config)
            }
        }
        "runner" => {
            let config = runner::parse_args(matches).expect(BAD_CLAP);
            if print_config {
                let text = serde_yaml::to_string(&config).expect(BAD_SERDE);
                ParsedCli::PrintConfig(text)
            } else {
                ParsedCli::Runner(config)
            }
        }
        "controller" => {
            let config = controller::parse_args(matches).expect(BAD_CLAP);
            if print_config {
                let text = serde_yaml::to_string(&config).expect(BAD_SERDE);
                ParsedCli::PrintConfig(text)
            } else {
                ParsedCli::Controller(config)
            }
        }
        _ => panic!("{}", BAD_CLAP),
    };

    Ok(result)
}

#[derive(Clone, Debug)]
pub enum ParsedCli {
    LoadTest(LoadTestConfig),
    Runner(RunnerConfig),
    Controller(ControllerConfig),
    PrintConfig(String),
    Help(String),
}

#[derive(Error, Debug)]
pub struct InvalidArgsError(String);

impl Display for InvalidArgsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
