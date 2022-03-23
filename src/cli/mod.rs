mod load;
mod root;
mod server;

/// Parses the command-line arguments into a
/// [`config::Config`][crate::config::Config] struct.
///
/// This function will exit and print an appropriate help message if the
/// supplied command-line arguments are invalid. The returned [clap::ArgMatches]
/// is guaranteed to be valid (anything less should be considered a bug).
pub fn parse() -> crate::config::Config {
    let matches = root::command().get_matches();
    // let subcommand = matches.subcommand().unwrap();
    match matches.subcommand() {
        Some(_) => todo!(),
        None => todo!(),
    }
}

mod validator {
    use std::{fs::File, str::FromStr};

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

    pub(crate) fn readable_file(s: &str) -> Result<(), String> {
        File::open(s).map(|_| ()).map_err(|err| err.to_string())
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

    pub(crate) fn kv(s: &str) -> Result<(), String> {
        let split: Vec<_> = s.split(':').collect();
        if split.len() != 2 {
            return Err("Invalid K:V value".into());
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
