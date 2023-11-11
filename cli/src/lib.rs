//! CLI resources used by the 'metron` binary.

mod controller;
mod parser;
mod root;
mod runner;
mod test;

use std::ffi::OsString;

use clap::error::ErrorKind;
use metron::{ControllerConfig, LoadTestConfig, RunnerConfig};
use thiserror::Error;

pub(crate) const CLAP_EXPECT: &str = "clap has been misconfigured";

pub fn parse<I, T>(it: I) -> Result<ParsedCli, CliError>
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    let spec = root::command()
        .try_get_matches_from(it)
        .and_then(|matches| match matches.subcommand() {
            Some(("test", matches)) => test::parse_args(matches).map(ParsedCli::LoadTest),
            Some(("runner", matches)) => runner::parse_args(matches).map(ParsedCli::Runner),
            Some(("controller", matches)) => {
                controller::parse_args(matches).map(ParsedCli::Controller)
            }
            // TODO: Sort this out...
            _ => panic!("couldn't match clap subcommand"),
        })
        .or_else(|clap_err| {
            let msg = clap_err.render().to_string();
            match clap_err.kind() {
                ErrorKind::DisplayHelp | ErrorKind::DisplayVersion => Ok(ParsedCli::Help(msg)),
                _ => Err(CliError::Invalid(msg)),
            }
        })?;

    Ok(spec)
}

#[derive(Clone, Debug)]
pub enum ParsedCli {
    LoadTest(LoadTestConfig),
    Runner(RunnerConfig),
    Controller(ControllerConfig),
    Help(String),
}

#[derive(Error, Debug)]
pub enum CliError {
    #[error("{0}")]
    Invalid(String),

    #[error(transparent)]
    Unexpected(#[from] anyhow::Error),
}
