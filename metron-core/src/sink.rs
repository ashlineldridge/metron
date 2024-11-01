use std::time::Duration;

use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "lowercase", content = "spec")]
pub enum Sink {
    Otel(OtelSink),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct OtelSink {
    pub address: Url,
    pub period: Duration,
    pub timeout: Duration,
}
