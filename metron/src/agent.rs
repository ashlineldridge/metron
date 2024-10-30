use std::{future::Future, pin::Pin, task::Poll};

use thiserror::Error;
use tower::{balance::p2c::Balance, Service};
use tracing::info;

use crate::{Plan, SignallerKind};

#[derive(Error, Debug)]
pub enum AgentError {
    #[error(transparent)]
    Unexpected(#[from] anyhow::Error),
}

#[derive(Clone)]
pub struct Agent {
    name: String,
    signaller: SignallerKind,
    worker_threads: usize,
}

impl Agent {
    pub fn new(name: String, signaller: SignallerKind, worker_threads: usize) -> Self {
        Self {
            name,
            signaller,
            worker_threads,
        }
    }

    pub async fn run(&self, plan: &Plan) -> Result<(), AgentError> {
        info!("agent is executing the plan {:?}", plan);

        Ok(())
    }
}

struct AgentRequest {
    plan: Plan,
    // start_time: Option<Time>,
}

impl Service<Plan> for Agent {
    type Response = ();
    type Error = AgentError;
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
