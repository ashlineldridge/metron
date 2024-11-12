use std::future::Future;

use quanta::Instant;

use crate::Plan;

#[derive(Clone, Debug)]
pub struct AgentRequest {
    pub plan: Plan,
    pub start: Instant,
}

pub trait Agent {
    fn execute(&self, req: AgentRequest) -> impl Future<Output = Result<(), anyhow::Error>> + Send;
}
