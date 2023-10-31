use clap::value_parser;

/// Creates the [`clap::Command`] for the `agent` subcommand.
///
/// # Examples
/// ```bash
/// # Run Metron as a gRPC agent listening on port 9090.
/// metron agent --port 9090
/// ```
pub(crate) fn command() -> clap::Command {
    const SHORT: &str = "Run Metron as a distributed agent instance.";
    const LONG: &str = "\
Run Metron as a gRPC agent that listens for instructions from a controller.
Typically, agents are deployed in a pool and managed by a central controller.
The controller can be Metron running as a CLI tool (e.g. on a laptop) or
running as a distributed controller instance (e.g. as a Kubernetes pod).
";

    clap::Command::new("agent")
        .about(SHORT)
        .long_about(LONG)
        .args(all_args())
        .groups(all_arg_groups())
        .disable_version_flag(true)
}

/// Returns all [`clap::Arg`]s for the `agent` subcommand.
fn all_args() -> Vec<clap::Arg> {
    vec![arg_port()]
}

/// Returns the [`clap::ArgGroup`]s for the `agent` subcommand.
fn all_arg_groups() -> Vec<clap::ArgGroup> {
    vec![]
}

/// Returns the [`clap::Arg`] for `--port`.
fn arg_port() -> clap::Arg {
    const SHORT: &str = "Agent gRPC port to listen on.";
    const LONG: &str = "\
Set the agent's gRPC port to PORT. Defaults to 9090.
";

    clap::Arg::new("port")
        .long("port")
        .value_name("PORT")
        .default_value("9090")
        .value_parser(value_parser!(u16))
        .help(SHORT)
        .long_help(LONG)
}
