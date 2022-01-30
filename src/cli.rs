use crate::{
    serve::ServeConfig,
    test::{Header, TestConfig},
};
use anyhow::Result;
use clap::{Parser, Subcommand};
use hyper::Uri;
use std::str::FromStr;

#[derive(Parser)]
#[clap(version, about, long_about = None)]
pub struct Cli {
    /// Subcommand
    #[clap(subcommand)]
    pub command: Command,
}

impl Cli {
    pub fn parse_args() -> Self {
        Cli::parse()
    }
}

#[derive(Subcommand)]
pub enum Command {
    /// Run a performance test
    Test(TestCli),

    /// Start an echo server
    Serve(ServeCli),
}

#[derive(Debug, Parser)]
pub struct TestCli {
    /// Number of connections to keep open
    #[clap(short, long, value_name = "COUNT", default_value_t = 10)]
    pub connections: usize,

    /// Number of threads
    #[clap(short, long, value_name = "COUNT", default_value_t = 4)]
    pub threads: usize,

    /// Requests per second (0 for max)
    #[clap(short, long, default_value_t = 0)]
    pub rate: usize,

    /// Execution duration (0s for forever)
    #[clap(short, long, default_value = "0s")]
    pub duration: humantime::Duration,

    /// Header to pass in request (may be repeated)
    #[clap(short = 'H', long, value_name = "NAME:VALUE")]
    pub header: Vec<Header>,

    // Target URL
    pub target: Uri,
}

impl From<TestCli> for TestConfig {
    fn from(cli: TestCli) -> Self {
        Self {
            connections: cli.connections,
            threads: cli.threads,
            rate: cli.rate,
            duration: cli.duration,
            headers: cli.header,
            target: cli.target,
        }
    }
}

#[derive(Debug, Parser)]
pub struct ServeCli {
    /// Port to listen on
    #[clap(short, long, default_value_t = 8080)]
    pub port: u16,

    /// Number of threads
    #[clap(short, long, value_name = "COUNT", default_value_t = 4)]
    pub threads: usize,
}

impl From<ServeCli> for ServeConfig {
    fn from(cli: ServeCli) -> Self {
        Self {
            port: cli.port,
            threads: cli.threads,
        }
    }
}

impl FromStr for Header {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (name, value) = s.split_once(':').unwrap_or((s, ""));

        Ok(Self {
            name: name.trim().into(),
            value: value.trim().into(),
        })
    }
}
