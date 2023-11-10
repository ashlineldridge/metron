use std::{future::Future, pin::Pin, task::Poll};

use serde::{Deserialize, Serialize};
use thiserror::Error;
use tower::Service;
use tracing::info;

use crate::core::Plan;

// Consider renaming Agent -> Runner and pulling the Runner
// logic into here. It seems like a useless composition at the moment.
#[derive(Clone)]
pub struct Runner {
    config: MetronRunnerConfig,
}

impl Runner {
    pub fn new(config: MetronRunnerConfig) -> Self {
        Self { config }
    }

    pub async fn run(&self, plan: &Plan) -> Result<(), RunnerError> {
        info!(
            "runner is executing the plan against target {}",
            plan.targets.first().unwrap()
        );

        Ok(())
    }
}

impl Service<Plan> for Runner {
    type Response = ();
    type Error = RunnerError;
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

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MetronRunnerConfig {
    pub port: u16,
}

impl Default for MetronRunnerConfig {
    fn default() -> Self {
        // TODO***: How to share defaults between cli and core? IMO, ideally, core should
        // define the defaults since it's "business logic".
        Self { port: 9090 }
    }
}

#[derive(Error, Debug)]
pub enum RunnerError {
    #[error(transparent)]
    Unexpected(#[from] anyhow::Error),
}
