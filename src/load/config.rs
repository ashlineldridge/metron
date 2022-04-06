use std::str::FromStr;

use url::Url;
use wrkr::LogLevel;

use crate::error::Error;
use crate::load::RateBlock;
use crate::load::SignallerKind;

#[derive(Clone, Debug)]
pub struct Config {
    pub blocks: Vec<RateBlock>,
    pub connections: usize,
    pub http_method: hyper::Method,
    pub targets: Vec<Url>,
    pub headers: Vec<Header>,
    pub payload: Option<String>,
    pub worker_threads: Option<usize>,
    pub signaller_kind: SignallerKind,
    pub log_level: LogLevel,
}

#[derive(Clone, Debug)]
pub struct Header {
    pub name: String,
    pub value: String,
}

impl FromStr for Header {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some((k, v)) = s.split_once(':') {
            Ok(Self {
                name: k.into(),
                value: v.into(),
            })
        } else {
            Err(Error::GenericError(format!("Invalid K:V value: {}", s)))
        }
    }
}
