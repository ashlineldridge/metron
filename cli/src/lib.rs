//! CLI resources used by the 'metron` binary.

mod controller;
mod parser;
mod root;
mod runner;
mod test;

use std::ffi::OsString;

use anyhow::{anyhow, Context};
use clap::error::ErrorKind;
use metron::{ControllerConfig, LoadTestConfig, RunnerConfig};
use thiserror::Error;

pub(crate) const BAD_CLAP: &str = "clap has been misconfigured";
pub(crate) const BAD_SERDE: &str = "serde has been misconfigured";

pub fn parse<I, T>(it: I) -> Result<ParsedCli, CliError>
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    let matches = match root::command().try_get_matches_from(it) {
        Ok(m) => m,
        Err(e) => {
            let msg = e.render().to_string();
            if e.kind() == ErrorKind::DisplayHelp || e.kind() == ErrorKind::DisplayVersion {
                return Ok(ParsedCli::Help(msg));
            } else {
                return Err(CliError::Invalid(msg));
            }
        }
    };

    let result = if let Some((command, matches)) = matches.subcommand() {
        let print_config = *matches.get_one("print-config").unwrap_or(&false);
        match command {
            "test" => {
                let config = test::parse_args(matches).context(BAD_CLAP)?;
                if print_config {
                    let text = serde_yaml::to_string(&config).context(BAD_SERDE)?;
                    Ok(ParsedCli::PrintConfig(text))
                } else {
                    Ok(ParsedCli::LoadTest(config))
                }
            }
            "runner" => {
                let config = runner::parse_args(matches).context(BAD_CLAP)?;
                if print_config {
                    let text = serde_yaml::to_string(&config).context(BAD_SERDE)?;
                    Ok(ParsedCli::PrintConfig(text))
                } else {
                    Ok(ParsedCli::Runner(config))
                }
            }
            "controller" => {
                let config = controller::parse_args(matches).context(BAD_CLAP)?;
                if print_config {
                    let text = serde_yaml::to_string(&config).context(BAD_SERDE)?;
                    Ok(ParsedCli::PrintConfig(text))
                } else {
                    Ok(ParsedCli::Controller(config))
                }
            }
            _ => Err(CliError::Unexpected(anyhow!(BAD_CLAP))),
        }
    } else {
        Err(CliError::Unexpected(anyhow!(BAD_CLAP)))
    };

    result
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
pub enum CliError {
    #[error("{0}")]
    Invalid(String),

    #[error(transparent)]
    Unexpected(#[from] anyhow::Error),
}

impl CliError {
    pub(crate) fn invalid(
        command: &mut clap::Command,
        kind: clap::error::ErrorKind,
        message: &str,
    ) -> CliError {
        CliError::Invalid(command.error(kind, message).render().to_string())
    }
}
