use std::{future::Future, pin::Pin, task::Poll};

use thiserror::Error;
use tower::Service;
use tracing::info;

use crate::Plan;

#[derive(Clone, Default)]
pub struct Runner {}

impl Runner {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn run(&self, plan: &Plan) -> Result<(), RunnerError> {
        info!("runner is executing the plan {:?}", plan);

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
        let runner = self.clone();
        Box::pin(async move { runner.run(&req).await })
    }
}

#[derive(Error, Debug)]
pub enum RunnerError {
    #[error(transparent)]
    Unexpected(#[from] anyhow::Error),
}
