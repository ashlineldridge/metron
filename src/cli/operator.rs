use clap::{value_parser, ArgAction};
use metron::LogLevel;

/// Creates the [`clap::Command`] for the `operator` subcommand.
///
/// # Examples
/// ```bash
/// metron operator --blah --blah
/// ```
pub(crate) fn command() -> clap::Command {
    const SHORT: &str = "Runs a Metron operator instance.";
    const LONG: &str = "\
TBD

Hmmm...
";

    clap::Command::new("operator")
        .about(SHORT)
        .long_about(LONG)
        .args(all_args())
        .groups(all_arg_groups())
        .disable_version_flag(true)
}

/// Returns all [`clap::Arg`]s for the `server` subcommand.
fn all_args() -> Vec<clap::Arg> {
    vec![
        arg_log_level(),
        arg_worker_threads(),
        arg_single_threaded(),
    ]
}

/// Returns the [`clap::ArgGroup`]s for the `server` subcommand.
fn all_arg_groups() -> Vec<clap::ArgGroup> {
    vec![]
}

/// Returns the [`clap::Arg`] for `--worker-threads`.
fn arg_worker_threads() -> clap::Arg {
    const SHORT: &str = "Number of worker threads to use.";
    const LONG: &str = "\
Sets the number of worker threads to be used by the runtime to COUNT.

The worker threads are the set of threads that are cooperatively scheduled to
perform the load test. This number does not include the thread allocated to the
signaller if a blocking signaller is used (see --signaller).

This argument defaults to the number of cores on the host machine.
";

    clap::Arg::new("worker-threads")
        .long("worker-threads")
        .group("group-thread-model")
        .value_name("COUNT")
        .value_parser(value_parser!(u64).range(1..1000))
        .help(SHORT)
        .long_help(LONG)
}

/// Returns the [`clap::Arg`] for `--single-threaded`.
fn arg_single_threaded() -> clap::Arg {
    const SHORT: &str = "Don't spawn threads.";
    const LONG: &str = "\
Forces all operations to run on the main thread.

The utility of this argument is unknown beyond providing interesting data on how
the number of threads affects performance of the tool itself. This argument
forces all operations to run on the main thread whereas --worker-threads=1 will
result in the main thread creating a single worker thread to perform the
requests.

This argument is incompatible with --worker-threads and --signaller=blocking.
";

    clap::Arg::new("single-threaded")
        .long("single-threaded")
        .group("group-thread-model")
        .action(ArgAction::SetTrue)
        .help(SHORT)
        .long_help(LONG)
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
