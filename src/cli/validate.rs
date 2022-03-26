use std::{fs::File, str::FromStr};

fn parse<T>(s: &str) -> Result<T, String>
where
    T: FromStr,
    T::Err: ToString,
{
    s.parse::<T>().map_err(|e| e.to_string())
}

pub fn validate<T>(s: &str) -> Result<(), String>
where
    T: FromStr,
    T::Err: ToString,
{
    parse::<T>(s).map(|_| ())
}

pub fn file(s: &str) -> Result<(), String> {
    File::open(s).map(|_| ()).map_err(|err| err.to_string())
}

pub fn url(s: &str) -> Result<(), String> {
    let url = parse::<url::Url>(s)?;

    if url.cannot_be_a_base() {
        return Err("Invalid target URL".into());
    }

    let scheme = url.scheme();
    if scheme != "http" && scheme != "https" {
        return Err("Only HTTP and HTTPS URL schemes are currently supported".into());
    }

    Ok(())
}

pub fn key_value(s: &str) -> Result<(), String> {
    if s.split_once(':').is_some() {
        Ok(())
    } else {
        Err("Invalid K:V value".into())
    }
}
