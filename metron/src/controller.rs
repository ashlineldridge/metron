use std::{future::Future, pin::Pin, task::Poll};

use anyhow::Context;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tower::Service;

use crate::{MetronControllerConfig, Plan};

#[derive(Clone)]
pub struct Controller<S> {
    config: MetronControllerConfig,
    agents: Vec<S>,
}

impl<S> Controller<S>
where
    S: Service<Plan> + Clone + Send + Sync + 'static,
    S::Response: Send + Sync + 'static,
    S::Error: std::error::Error + Send + Sync + 'static,
    S::Future: Send + 'static,
{
    pub fn new(config: MetronControllerConfig, agents: Vec<S>) -> Self {
        Self { config, agents }
    }

    pub async fn run(&self, plan: &Plan) -> Result<(), ControllerError> {
        // TODO: This needs to call the agents in parallel.
        let mut agent = self
            .agents
            .first()
            .cloned()
            .context("at least one agent is required")?;

        agent
            .call(plan.clone())
            .await
            .map_err(|e| ControllerError::Unexpected(e.into()))?;

        Ok(())
    }
}

// For now, the Controller just gives the same plan to all agents.
impl<S> Service<Plan> for Controller<S>
where
    S: Service<Plan> + Clone + Send + Sync + 'static,
    S::Response: Send + Sync + 'static,
    S::Error: std::error::Error + Send + Sync + 'static,
    S::Future: Send + 'static,
{
    type Response = ();
    type Error = ControllerError;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

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

#[derive(Error, Debug)]
pub enum ControllerError {
    #[error(transparent)]
    Unexpected(#[from] anyhow::Error),
}
