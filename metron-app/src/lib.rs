//! CLI resources used by the 'metron` binary.

mod parser;
mod root;
mod test;

use std::{ffi::OsString, fmt::Display};

use clap::error::ErrorKind;
use metron_config::{AgentConfig, TestConfig};
use thiserror::Error;

pub(crate) const BAD_CLAP: &str = "clap has been misconfigured";

#[allow(unused)]
pub(crate) const BAD_SERDE: &str = "serde has been misconfigured";

pub use parser::HttpHeader;

#[derive(Clone, Debug)]
pub enum ParsedCli {
    Test(TestConfig),
    Agent(AgentConfig),
    Help(String),
}

#[derive(Error, Debug)]
pub struct InvalidArgsError(String);

impl Display for InvalidArgsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

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

    let result = match command {
        "run" => ParsedCli::Test(test::parse(matches).expect(BAD_CLAP)),
        _ => panic!("{}", BAD_CLAP),
    };

    Ok(result)
}
