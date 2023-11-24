use clap::{value_parser, ArgAction};
use metron::ControllerConfig;

use crate::{parser, InvalidArgsError, BAD_CLAP};

/// Creates the [`clap::Command`] for the `controller` subcommand.
pub(crate) fn command() -> clap::Command {
    const SHORT: &str = "Start a Metron controller.";
    const LONG: &str = "\
Run Metron as a gRPC service that controls a pool of runners. Metron
controllers also implement the runner protobuf contract allowing them
to be composed.
";

    clap::Command::new("controller")
        .about(SHORT)
        .long_about(LONG)
        .args(all_args())
        .groups(all_arg_groups())
        .disable_version_flag(true)
}

pub(crate) fn parse(matches: &clap::ArgMatches) -> Result<ControllerConfig, InvalidArgsError> {
    let config = matches
        .get_one::<ControllerConfig>("file")
        .cloned()
        .expect(BAD_CLAP);
    // .cloned()
    // .unwrap_or_default();

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

/// Returns the [`clap::Arg`] for `--file`.
fn arg_config_file() -> clap::Arg {
    const SHORT: &str = "Controller configuration file.";
    const LONG: &str = "\
A configuration file to be used as an alternative to individual command line
arguments. Stdin can also be used by specifying hyphen as the file name (i.e.
`--file -`).

When both a configuration file and individual command line arguments are used,
the arguments will override their counterpart properties in the configuration
file.

See --print-config for bootstrapping a configuration file.
";

    clap::Arg::new("file")
        .long("file")
        .value_name("FILE")
        .value_parser(parser::config_file::<ControllerConfig>)
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
    const SHORT: &str = "gRPC port to listen on.";
    const LONG: &str = "\
Set the controller's gRPC port to PORT. Defaults to 9090.
";

    clap::Arg::new("port")
        .long("port")
        .value_name("PORT")
        .default_value("9090")
        .value_parser(value_parser!(u16))
        .help(SHORT)
        .long_help(LONG)
}
