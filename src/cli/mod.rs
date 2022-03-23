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

mod validator {
    use std::str::FromStr;

    fn try_parse<T>(s: &str) -> Result<(), String>
    where
        T: FromStr,
        T::Err: ToString,
    {
        try_parse_t::<T>(s).map(|_| ())
    }

    fn try_parse_t<T>(s: &str) -> Result<T, String>
    where
        T: FromStr,
        T::Err: ToString,
    {
        s.parse::<T>().map_err(|e| e.to_string())
    }

    pub(crate) fn duration(s: &str) -> Result<(), String> {
        try_parse::<humantime::Duration>(s)
    }

    pub(crate) fn url(s: &str) -> Result<(), String> {
        let url = try_parse_t::<url::Url>(s)?;

        if url.cannot_be_a_base() {
            return Err("Invalid target URL".into());
        }

        let scheme = url.scheme();
        if scheme != "http" && scheme != "https" {
            return Err("Only HTTP and HTTPS URL schemes are currently supported".into());
        }

        Ok(())
    }

    pub(crate) fn u16(s: &str) -> Result<(), String> {
        try_parse::<u16>(s)
    }

    pub(crate) fn usize(s: &str) -> Result<(), String> {
        try_parse::<usize>(s)
    }
}
