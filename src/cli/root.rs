use crate::cli::{load, server};

const ABOUT: &str = "
wrkr is a modern performance characterization tool.

Use --help for more details.

Project home: https://github.com/ashlineldridge/wrkr
";

/// Returns the root [`clap::Command`] for the application.
pub(crate) fn command() -> clap::Command<'static> {
    use clap::*;

    Command::new("wrkr")
        .author(crate_authors!())
        .version(crate_version!())
        .about(ABOUT)
        .subcommand(load::command())
        .subcommand(server::command())
        .subcommand_required(true)
}
