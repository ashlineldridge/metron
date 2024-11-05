use std::future::Future;

use thiserror::Error;

use crate::{Plan, Report};

#[derive(Error, Debug)]
pub enum AgentError {
    #[error(transparent)]
    Unexpected(#[from] anyhow::Error),
}

pub trait Agent {
    fn test(&self, plan: &Plan) -> impl Future<Output = Result<(), AgentError>> + Send;
    fn cancel(&self) -> impl Future<Output = Result<(), AgentError>> + Send;
}
