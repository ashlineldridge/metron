mod load;
mod server;

const ABOUT: &str = "
wrkr is a modern performance characterization tool.

Use --help for more details.

Project home: https://github.com/ashlineldridge/wrkr
";

/// Parses the command-line arguments into a [clap::ArgMatches].
///
/// This function will exit and print an appropriate help message if the
/// supplied command-line arguments are invalid. The returned [clap::ArgMatches]
/// is guaranteed to be valid (anything less should be considered a bug).
pub fn parse_clap() -> clap::ArgMatches {
    root_command().get_matches()
}

/// Returns the root [`clap::Command`] for the application.
fn root_command() -> clap::Command<'static> {
    use clap::*;

    Command::new("wrkr")
        .author(crate_authors!())
        .version(crate_version!())
        .about(ABOUT)
        .subcommand(load::command())
        .subcommand(server::command())
        .subcommand_required(true)
        .arg(Arg::new(""))
}
