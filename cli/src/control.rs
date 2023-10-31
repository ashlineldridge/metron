use clap::value_parser;

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

    clap::Command::new("control")
        .about(SHORT)
        .long_about(LONG)
        .args(all_args())
        .groups(all_arg_groups())
        .disable_version_flag(true)
}

/// Returns all [`clap::Arg`]s for the `control` subcommand.
fn all_args() -> Vec<clap::Arg> {
    vec![arg_port()]
}

/// Returns the [`clap::ArgGroup`]s for the `control` subcommand.
fn all_arg_groups() -> Vec<clap::ArgGroup> {
    vec![]
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
