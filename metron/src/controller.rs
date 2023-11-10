use std::{future::Future, pin::Pin, task::Poll};

use anyhow::Context;
use thiserror::Error;
use tower::Service;

use crate::LoadTestPlan;

#[derive(Clone)]
pub struct Controller<S> {
    runners: Vec<S>,
}

impl<S> Controller<S>
where
    S: Service<LoadTestPlan> + Clone + Send + Sync + 'static,
    S::Response: Send + Sync + 'static,
    S::Error: std::error::Error + Send + Sync + 'static,
    S::Future: Send + 'static,
{
    pub fn new(runners: Vec<S>) -> Self {
        Self { runners }
    }

    pub async fn run(&self, plan: &LoadTestPlan) -> Result<(), ControllerError> {
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
impl<S> Service<LoadTestPlan> for Controller<S>
where
    S: Service<LoadTestPlan> + Clone + Send + Sync + 'static,
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

    fn call(&mut self, req: LoadTestPlan) -> Self::Future {
        let controller = self.clone();
        Box::pin(async move { controller.run(&req).await })
    }
}

#[derive(Error, Debug)]
pub enum ControllerError {
    #[error(transparent)]
    Unexpected(#[from] anyhow::Error),
}
