use url::Url;

use crate::load::RateBlock;
use crate::load::SignallerKind;

#[derive(Clone, Debug)]
pub struct Config {
    pub blocks: Vec<RateBlock>,
    pub connections: usize,
    pub http_method: hyper::Method,
    pub targets: Vec<Url>,
    pub headers: Vec<Header>,
    pub payload: String,
    pub worker_threads: Option<usize>,
    pub signaller_kind: SignallerKind,
}

#[derive(Clone, Debug)]
pub struct Header {
    pub name: String,
    pub value: String,
}
