use std::str::FromStr;

use anyhow::bail;
use metron::LogLevel;
use serde::{Deserialize, Serialize};
use url::Url;

use crate::{
    profile::{PlanSegment, SignallerKind},
    runtime,
};

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Config {
    pub segments: Vec<PlanSegment>,
    pub connections: usize,
    pub http_method: String,
    pub targets: Vec<Url>,
    pub headers: Vec<Header>,
    pub payload: Option<String>,
    pub runtime: runtime::Config,
    pub signaller_kind: SignallerKind,
    pub stop_on_error: bool,
    pub stop_on_non_2xx: bool,
    pub log_level: LogLevel,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
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
