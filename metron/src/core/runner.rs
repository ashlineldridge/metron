use serde::{Deserialize, Serialize};
use thiserror::Error;
use url::Url;

use crate::core::{Header, HttpMethod, Plan, PlanSegment};

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Unexpected(#[from] anyhow::Error),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Config {
    // TODO: Should this be wrapped up in a plan?
    pub segments: Vec<PlanSegment>,
    pub connections: usize,
    pub http_method: HttpMethod,
    pub targets: Vec<Url>,
    pub headers: Vec<Header>,
    pub payload: Option<String>,
    pub worker_threads: usize,
    pub latency_correction: bool,
}

pub struct Runner {
    config: Config,
}

impl Runner {
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    pub async fn run(plan: Plan) -> Result<(), Error> {
        todo!()
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            segments: vec![],
            connections: 1,
            http_method: HttpMethod::Get,
            targets: vec![],
            headers: vec![],
            payload: None,
            worker_threads: 0,
            latency_correction: true,
        }
    }
}
