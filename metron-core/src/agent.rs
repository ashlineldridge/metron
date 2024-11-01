use std::future::Future;

use thiserror::Error;

use crate::{Plan, Report};

#[derive(Error, Debug)]
pub enum AgentError {
    #[error(transparent)]
    Unexpected(#[from] anyhow::Error),
}

pub trait Agent {
    fn test(&mut self, _plan: &Plan) -> impl Future<Output = Result<(), AgentError>> + Send;
    fn cancel(&mut self) -> impl Future<Output = Result<(), AgentError>> + Send;
    fn report(&mut self) -> impl Future<Output = Result<Report, AgentError>> + Send;
    fn proxy(&mut self) -> impl Future<Output = Result<Report, AgentError>> + Send;
}
