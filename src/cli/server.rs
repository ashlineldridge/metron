use crate::cli::validate::validate;

/// Creates the [`clap::Command`] for the `server` subcommand.
///
/// # Examples
/// ```bash
/// metron server --port 8080
/// ```
pub(crate) fn command() -> clap::Command<'static> {
    const SHORT: &str = "Runs an echo server.";
    const LONG: &str = "\
Runs an echo server that may be used within performance profile tests.

This command starts a echo server that may be configured in terms of its
responses, latency, and other properties.
";

    clap::Command::new("server")
        .about(SHORT)
        .long_about(LONG)
        .args(all_args())
        .groups(all_arg_groups())
}

/// Returns all [`clap::Arg`]s for the `server` subcommand.
fn all_args() -> Vec<clap::Arg<'static>> {
    vec![arg_log_level(), arg_port(), arg_worker_threads()]
}

/// Returns the [`clap::ArgGroup`]s for the `server` subcommand.
fn all_arg_groups() -> Vec<clap::ArgGroup<'static>> {
    vec![]
}

/// Returns the [`clap::Arg`] for `--log-level`.
fn arg_log_level() -> clap::Arg<'static> {
    const SHORT: &str = "Minimum logging level.";
    const LONG: &str = "\
Sets the minimum logging level. Log messages at or above the specified
severity level will be printed.
";

    clap::Arg::new("log-level")
        .long("log-level")
        .value_name("LEVEL")
        .default_value("info")
        .possible_values(&["off", "debug", "info", "warn", "error"])
        .help(SHORT)
        .long_help(LONG)
}

/// Returns the [`clap::Arg`] for `--port`.
fn arg_port() -> clap::Arg<'static> {
    const SHORT: &str = "Port to serve on.";
    const LONG: &str = "\
Sets the server listening port to PORT. Defaults to 8000.
";

    clap::Arg::new("port")
        .long("port")
        .value_name("PORT")
        .default_value("8000")
        .validator(validate::<u16>)
        .help(SHORT)
        .long_help(LONG)
}

/// Returns the [`clap::Arg`] for `--worker-threads`.
fn arg_worker_threads() -> clap::Arg<'static> {
    const SHORT: &str = "Number of worker threads to use.";
    const LONG: &str = "\
Sets the number of worker threads to be used by the runtime to COUNT.

If this value is not specified it will default to the number of cores on the
host machine.
";

    clap::Arg::new("worker-threads")
        .long("worker-threads")
        .value_name("COUNT")
        .validator(validate::<usize>)
        .help(SHORT)
        .long_help(LONG)
}
