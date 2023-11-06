use std::{future::Future, pin::Pin, task::Poll};

use serde::{Deserialize, Serialize};
use thiserror::Error;
use tower::Service;

use crate::core::{Plan, Runner, RunnerError};

// TODO: Should there be an error enum per module or for all of core or...?
#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Unexpected(#[from] anyhow::Error),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Config {
    pub port: u16,
}

impl Default for Config {
    fn default() -> Self {
        // TODO***: How to share defaults between cli and core? IMO, ideally, core should
        // define the defaults since it's "business logic".
        Self { port: 9090 }
    }
}

#[derive(Clone)]
pub struct Agent {
    config: Config,
    runner: Runner,
}

impl Agent {
    pub fn new(config: Config, runner: Runner) -> Self {
        Self { config, runner }
    }

    pub async fn run(&self, plan: &Plan) -> Result<(), Error> {
        self.runner.run(plan).await.map_err(|e| match e {
            RunnerError::Unexpected(e) => Error::Unexpected(e),
        })
    }
}

impl Service<Plan> for Agent {
    type Response = ();
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(
        &mut self,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::result::Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Plan) -> Self::Future {
        let agent = self.clone();
        Box::pin(async move { agent.run(&req).await })
    }
}
