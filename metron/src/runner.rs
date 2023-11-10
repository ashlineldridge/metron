use std::{future::Future, pin::Pin, task::Poll};

use thiserror::Error;
use tower::Service;
use tracing::info;

use crate::LoadTestPlan;

#[derive(Clone, Default)]
pub struct Runner {}

impl Runner {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn run(&self, plan: &LoadTestPlan) -> Result<(), RunnerError> {
        info!(
            "runner is executing the plan against target {}",
            plan.targets.first().unwrap()
        );

        Ok(())
    }
}

impl Service<LoadTestPlan> for Runner {
    type Response = ();
    type Error = RunnerError;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(
        &mut self,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::result::Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: LoadTestPlan) -> Self::Future {
        let agent = self.clone();
        Box::pin(async move { agent.run(&req).await })
    }
}

#[derive(Error, Debug)]
pub enum RunnerError {
    #[error(transparent)]
    Unexpected(#[from] anyhow::Error),
}
