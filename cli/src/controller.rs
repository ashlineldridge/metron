use clap::{value_parser, ArgAction};
use metron::core::MetronControllerConfig;

use crate::parser;

/// Creates the [`clap::Command`] for the `control` subcommand.
///
/// # Examples
/// ```bash
/// # Run Metron as a gRPC controller listening on port 9191 and controlling
/// # an agent running at localhost:9090.
/// metron control --port 9191 --agent localhost:9090
/// ```
pub(crate) fn command() -> clap::Command {
    const SHORT: &str = "Run Metron as an agent controller.";
    const LONG: &str = "\
Run Metron as a gRPC server that controls a pool of agent instances. The gRPC
controller implements the same protobuf contract as the agent which allows
agents and controllers to be composed freely.
";

    clap::Command::new("controller")
        .about(SHORT)
        .long_about(LONG)
        .args(all_args())
        .groups(all_arg_groups())
        .disable_version_flag(true)
}

pub(crate) fn parse_args(
    matches: &clap::ArgMatches,
) -> Result<MetronControllerConfig, clap::Error> {
    let config = matches
        .get_one::<MetronControllerConfig>("config-file")
        .cloned()
        .unwrap_or_default();

    Ok(config)
}

/// Returns all [`clap::Arg`]s for the `control` subcommand.
fn all_args() -> Vec<clap::Arg> {
    vec![arg_config_file(), arg_print_config(), arg_port()]
}

/// Returns the [`clap::ArgGroup`]s for the `control` subcommand.
fn all_arg_groups() -> Vec<clap::ArgGroup> {
    vec![]
}

/// Returns the [`clap::Arg`] for `--config-file`.
fn arg_config_file() -> clap::Arg {
    const SHORT: &str = "Controller configuration file.";
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
        .value_parser(parser::config_file::<MetronControllerConfig>)
        .help(SHORT)
        .long_help(LONG)
}

/// Returns the [`clap::Arg`] for `--print-config`.
fn arg_print_config() -> clap::Arg {
    const SHORT: &str = "Print the controller configuration.";
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

/// Returns the [`clap::Arg`] for `--port`.
fn arg_port() -> clap::Arg {
    const SHORT: &str = "Agent gRPC port to listen on.";
    const LONG: &str = "\
Sets the agent's gRPC port to PORT. Defaults to 9090.
";

    clap::Arg::new("port")
        .long("port")
        .value_name("PORT")
        .default_value("9090")
        .value_parser(value_parser!(u16))
        .help(SHORT)
        .long_help(LONG)
}
