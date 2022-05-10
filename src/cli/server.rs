use crate::cli::validate::validate;

/// Creates the [`clap::Command`] for the `server` subcommand.
///
/// # Examples
/// ```bash
/// metron server --port 8080
/// ```
pub(crate) fn command() -> clap::Command<'static> {
    clap::Command::new("server")
        .arg(arg_port())
        .arg(arg_worker_threads())
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
