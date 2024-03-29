use std::time::Duration;

use anyhow::{bail, Result};
use either::Either;
use metron::{Header, Rate};
use url::Url;
use Either::{Left, Right};

pub type RateArgValue = Either<Rate, (Rate, Rate)>;

/// Request rate clap [`Arg::value_parser`][clap::Arg::value_parser].
pub fn rate(value: &str) -> Result<RateArgValue> {
    if let Some((rate_start, rate_end)) = value.split_once(':') {
        let rate_start = rate_start.parse()?;
        let rate_end = rate_end.parse()?;
        Ok(Right((rate_start, rate_end)))
    } else {
        let rate = value.parse()?;
        Ok(Left(rate))
    }
}

/// Duration clap [`Arg::value_parser`][clap::Arg::value_parser].
pub fn duration(value: &str) -> Result<Option<Duration>> {
    if value == "forever" {
        Ok(None)
    } else {
        let duration = value.parse::<humantime::Duration>()?;
        Ok(Some(duration.into()))
    }
}

/// Target URL clap [`Arg::value_parser`][clap::Arg::value_parser].
pub fn target(value: &str) -> Result<Url> {
    let url = value.parse::<url::Url>()?;

    if url.cannot_be_a_base() {
        bail!("Supplied URL cannot be a base URL");
    }

    let scheme = url.scheme();
    if scheme != "http" && scheme != "https" {
        bail!("Only HTTP and HTTPS URL schemes are currently supported");
    }

    Ok(url)
}

/// Header clap [`Arg::value_parser`][clap::Arg::value_parser].
pub fn header(value: &str) -> Result<Header> {
    if let Some((k, v)) = value.split_once(':') {
        Ok(Header {
            name: k.to_owned(),
            value: v.to_owned(),
        })
    } else {
        bail!("Headers must be specified in 'K:V' format");
    }
}
