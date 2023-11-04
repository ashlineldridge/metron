use std::{future::Future, pin::Pin, task::Poll};

use anyhow::Context;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tower::Service;

use crate::core::Plan;

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Unexpected(#[from] anyhow::Error),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Config {}

#[allow(clippy::derivable_impls)]
impl Default for Config {
    fn default() -> Self {
        Self {}
    }
}

#[derive(Clone)]
pub struct Controller<S> {
    config: Config,
    agents: Vec<S>,
}

impl<S> Controller<S>
where
    S: Service<Plan> + Clone + Send + 'static,
    S::Response: Send + Sync + 'static,
    S::Error: std::error::Error + Send + Sync + 'static,
    // S::Future: Send + 'static,
{
    pub fn new(config: Config, agents: Vec<S>) -> Self {
        Self { config, agents }
    }

    pub async fn run(&self, plan: &Plan) -> Result<(), Error> {
        // TODO: This needs to call the agents in parallel.
        let mut agent = self
            .agents
            .first()
            .cloned()
            .context("at least one agent is required")?;

        agent
            .call(plan.clone())
            .await
            .map_err(|e| Error::Unexpected(e.into()))?;
        Ok(())
    }
}

// For now, the Controller just gives the same plan to all agents.
impl<S> Service<Plan> for Controller<S>
where
    S: tower::Service<Plan> + Clone + Send + 'static,
    S::Response: Send + Sync + 'static,
    S::Error: std::error::Error + Send + Sync + 'static,
    // S::Future: Send + 'static,
{
    type Response = ();
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(
        &mut self,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::result::Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Plan) -> Self::Future {
        let controller = self.clone();
        Box::pin(async move { controller.run(&req).await })
    }
}
