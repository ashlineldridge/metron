use crate::cli::{profile, server};

const ABOUT: &str = "
Metron is a modern L7 performance profiler.

Use --help for more details.

Project home: https://github.com/ashlineldridge/metron
";

/// Returns the root [`clap::Command`] for the application.
pub(crate) fn command() -> clap::Command<'static> {
    use clap::*;

    Command::new("metron")
        .author(crate_authors!())
        .version(crate_version!())
        .about(ABOUT)
        .subcommands(all_subcommands())
        .subcommand_required(true)
}

fn all_subcommands() -> Vec<clap::Command<'static>> {
    vec![profile::command(), server::command()]
        .into_iter()
        .map(|c| c.args(common_args()).groups(common_arg_groups()))
        .collect()
}

/// Returns all [`clap::Arg`]s for the root command.
fn common_args() -> Vec<clap::Arg<'static>> {
    vec![]
}

/// Returns the [`clap::ArgGroup`]s for the root command.
fn common_arg_groups() -> Vec<clap::ArgGroup<'static>> {
    vec![]
}
