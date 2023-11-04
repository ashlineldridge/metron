//! CLI resources used by the 'metron` binary.

mod agent;
mod controller;
mod parser;
mod root;
mod run;

use std::ffi::OsString;

use clap::error::ErrorKind;
use metron::core::{AgentConfig, ControllerConfig, RunnerConfig};
pub use root::command;
use thiserror::Error;

pub(crate) const CLAP_EXPECT: &str = "clap has been misconfigured";

#[derive(Error, Debug)]
pub enum Error {
    #[error("{0}")]
    InvalidArgs(String),

    #[error(transparent)]
    Unexpected(#[from] anyhow::Error),
}

#[derive(Clone, Debug)]
pub enum Spec {
    Run(RunnerConfig),
    Agent(AgentConfig),
    Controller(ControllerConfig),
    Help(String),
}

pub fn parse<I, T>(it: I) -> Result<Spec, Error>
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    let spec = root::command()
        .try_get_matches_from(it)
        .and_then(|matches| match matches.subcommand() {
            Some(("run", matches)) => run::parse_args(matches).map(Spec::Run),
            Some(("agent", matches)) => agent::parse_args(matches).map(Spec::Agent),
            Some(("controller", matches)) => controller::parse_args(matches).map(Spec::Controller),
            // TODO: Sort this out...
            _ => panic!("couldn't match clap subcommand"),
        })
        .or_else(|clap_err| {
            let msg = clap_err.render().to_string();
            match clap_err.kind() {
                ErrorKind::DisplayHelp | ErrorKind::DisplayVersion => Ok(Spec::Help(msg)),
                _ => Err(Error::InvalidArgs(msg)),
            }
        })?;

    Ok(spec)
}
