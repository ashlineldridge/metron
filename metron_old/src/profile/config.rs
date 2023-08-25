use metron_old::{Header, HttpMethod, LogLevel};
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
    pub http_method: HttpMethod,
    pub targets: Vec<Url>,
    pub headers: Vec<Header>,
    pub payload: Option<String>,
    pub runtime: runtime::Config,
    pub signaller_kind: SignallerKind,
    pub no_latency_correction: bool,
    pub stop_on_client_error: bool,
    pub stop_on_non_2xx: bool,
    pub log_level: LogLevel,
}
