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
    /// Number of connections to use
    #[clap(short, long, value_name = "COUNT", default_value_t = 1)]
    pub connections: usize,

    /// Number of threads
    #[clap(short, long, value_name = "COUNT", default_value_t = 4)]
    pub threads: usize,

    /// Requests per second
    #[clap(short, long)]
    pub rate: Option<u32>,

    /// Execution duration
    #[clap(short, long)]
    pub duration: Option<humantime::Duration>,

    /// Initial requests per second for ramped throughput
    #[clap(short, long)]
    pub init_rate: Option<u32>,

    /// Duration over which to ramp up throughput
    #[clap(short, long)]
    pub ramp_duration: Option<humantime::Duration>,

    /// Header to pass in request (may be repeated)
    #[clap(short = 'H', long, value_name = "NAME:VALUE")]
    pub header: Vec<Header>,

    // Target URL
    pub target: Uri,
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

impl From<TestCli> for TestConfig {
    fn from(cli: TestCli) -> Self {
        Self {
            connections: cli.connections,
            worker_threads: cli.threads,
            rate: cli.rate,
            duration: cli.duration.map(|d| d.into()),
            init_rate: cli.init_rate,
            ramp_duration: cli.ramp_duration.map(|d| d.into()),
            headers: cli.header,
            target: cli.target,
        }
    }
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
