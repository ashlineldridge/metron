use std::str::FromStr;

use anyhow::bail;
use url::Url;
use wrkr::LogLevel;

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
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some((k, v)) = s.split_once(':') {
            Ok(Self {
                name: k.into(),
                value: v.into(),
            })
        } else {
            bail!("Invalid K:V value: {}", s);
        }
    }
}
