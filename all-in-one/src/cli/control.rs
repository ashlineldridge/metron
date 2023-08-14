use clap::value_parser;
use metron::LogLevel;

/// Creates the [`clap::Command`] for the `control` subcommand.
///
/// # Examples
/// ```bash
/// metron control --blah --blah
/// ```
pub(crate) fn command() -> clap::Command {
    const SHORT: &str = "Runs a Metron control command against a node.";
    const LONG: &str = "\
TBD

Hmmm...
";

    clap::Command::new("control")
        .about(SHORT)
        .long_about(LONG)
        .args(all_args())
        .groups(all_arg_groups())
        .disable_version_flag(true)
}

/// Returns all [`clap::Arg`]s for the `server` subcommand.
fn all_args() -> Vec<clap::Arg> {
    vec![arg_log_level()]
}

/// Returns the [`clap::ArgGroup`]s for the `server` subcommand.
fn all_arg_groups() -> Vec<clap::ArgGroup> {
    vec![]
}

/// Returns the [`clap::Arg`] for `--log-level`.
fn arg_log_level() -> clap::Arg {
    const SHORT: &str = "Minimum logging level.";
    const LONG: &str = "\
Sets the minimum logging level. Log messages at or above the specified
severity level will be printed.
";

    clap::Arg::new("log-level")
        .long("log-level")
        .value_name("LEVEL")
        .default_value("info")
        .value_parser(value_parser!(LogLevel))
        .help(SHORT)
        .long_help(LONG)
}
