use clap::{value_parser, ArgAction};
use metron::RunnerConfig;

use crate::{parser, InvalidArgsError, BAD_CLAP};

/// Create the [`clap::Command`] for the `runner` subcommand.
pub(crate) fn command() -> clap::Command {
    const SHORT: &str = "Start a Metron runner.";
    const LONG: &str = "\
Run Metron as a gRPC service that listens for instructions from a controller.
Typically, runners are deployed in a pool and managed by a central controller.
The controller can be Metron running as a CLI tool (e.g. on a laptop) or
running as a distributed controller instance (e.g. as a Kubernetes pod).
";

    clap::Command::new("runner")
        .about(SHORT)
        .long_about(LONG)
        .args(all_args())
        .groups(all_arg_groups())
        .disable_version_flag(true)
}

pub(crate) fn parse_args(matches: &clap::ArgMatches) -> Result<RunnerConfig, InvalidArgsError> {
    let mut config = matches
        .get_one::<RunnerConfig>("config-file")
        .cloned()
        .unwrap_or_default();

    config.server_port = *matches.get_one("port").expect(BAD_CLAP);

    Ok(config)
}

/// Return all [`clap::Arg`]s for the `runner` subcommand.
fn all_args() -> Vec<clap::Arg> {
    vec![arg_config_file(), arg_print_config(), arg_port()]
}

/// Return the [`clap::ArgGroup`]s for the `runner` subcommand.
fn all_arg_groups() -> Vec<clap::ArgGroup> {
    vec![]
}

/// Returns the [`clap::Arg`] for `--config-file`.
fn arg_config_file() -> clap::Arg {
    const SHORT: &str = "Runner configuration file.";
    const LONG: &str = "\
A configuration file to be used as an alternative to individual command line
arguments. Stdin can also be used by specifying hyphen as the file name (i.e.
`--config-file -`).

When both a configuration file and individual command line arguments are used,
the arguments will override their counterpart properties in the configuration
file.

See --print-config for bootstrapping a configuration file.
";

    clap::Arg::new("config-file")
        .long("config-file")
        .value_name("FILE")
        .value_parser(parser::config_file::<RunnerConfig>)
        .help(SHORT)
        .long_help(LONG)
}

/// Returns the [`clap::Arg`] for `--print-config`.
fn arg_print_config() -> clap::Arg {
    const SHORT: &str = "Print the runner configuration.";
    const LONG: &str = "\
Generates the configuration for this command and prints it to stdout. This may
be used to bootstrap a configuration file based on command line arguments so
that a configuration file can be used rather than individual command line
arguments.
";

    clap::Arg::new("print-config")
        .long("print-config")
        .action(ArgAction::SetTrue)
        .help(SHORT)
        .long_help(LONG)
}

/// Return the [`clap::Arg`] for `--port`.
fn arg_port() -> clap::Arg {
    const SHORT: &str = "gRPC port to listen on.";
    const LONG: &str = "\
Set the runner's gRPC port to PORT. Defaults to 9090.
";

    clap::Arg::new("port")
        .long("port")
        .value_name("PORT")
        .default_value(default::PORT.as_str())
        .value_parser(value_parser!(u16))
        .help(SHORT)
        .long_help(LONG)
}

mod default {
    use super::*;
    lazy_static::lazy_static! {
        static ref CONFIG: RunnerConfig = RunnerConfig::default();
        pub(super) static ref PORT: String = CONFIG.server_port.to_string();
    }
}
