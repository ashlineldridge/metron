use std::{future::Future, pin::Pin, task::Poll};

use anyhow::{anyhow, Context};
use thiserror::Error;
use tower::Service;

use crate::Plan;

// TODO: Rename Agents
#[derive(Clone)]
pub struct Controller<S> {
    // TODO: This should either be passed a RunnerRegistry
    // or an D: Discover where the implementation is a RunnerRegistry.
    // In either case, the S service type needs to abstract over both remote
    // and local runners so we'll go with a Box implementation to start with.
    runners: Vec<S>,
}

impl<S> Controller<S>
where
    S: Service<Plan> + Clone + Send + Sync + 'static,
    S::Response: Send + Sync + 'static,
    S::Error: std::error::Error + Send + Sync + 'static,
    S::Future: Send + 'static,
{
    pub fn new(runners: Vec<S>) -> Self {
        Self { runners }
    }

    pub async fn run(&self, plan: &Plan) -> Result<(), ControllerError> {
        // TODO: This needs to call the runners in parallel.
        let mut runner = self
            .runners
            .first()
            .cloned()
            .context("at least one runner is required")?;

        runner
            .call(plan.clone())
            .await
            .map_err(|e| ControllerError::Unexpected(e.into()))?;

        Ok(())
    }
}

// For now, the Controller just gives the same plan to all runners.
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
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::result::Result<(), Self::Error>> {
        let mut dead = 0;
        for s in &mut self.runners {
            match s.poll_ready(cx) {
                Poll::Ready(Ok(_)) => return Poll::Ready(Ok(())),
                Poll::Ready(Err(_)) => dead += 1,
                _ => continue,
            }
        }

        if dead == self.runners.len() {
            return Poll::Ready(Err(anyhow!("all runners have terminally failed").into()));
        }

        Poll::Pending
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
