use crate::{Agent, Plan, Report, Sink};

#[derive(Clone, Copy, Debug)]
pub enum SignallerKind {
    Dedicated,
    Cooperative,
}

#[derive(Clone)]
#[allow(unused)]
pub struct Runner {
    name: String,
    signaller: SignallerKind,
    worker_threads: usize,
    sinks: Vec<Sink>,
}

impl Runner {
    pub fn new(
        name: String,
        signaller: SignallerKind,
        worker_threads: usize,
        sinks: Vec<Sink>,
    ) -> Self {
        Self {
            name,
            signaller,
            worker_threads,
            sinks,
        }
    }
}

impl Agent for Runner {
    async fn test(&self, _plan: &Plan) -> Result<(), crate::AgentError> {
        Ok(())
    }

    async fn cancel(&self) -> Result<(), crate::AgentError> {
        Ok(())
    }

    async fn report(&self) -> Result<Report, crate::AgentError> {
        Ok(Report {})
    }

    async fn proxy(&self) -> Result<Report, crate::AgentError> {
        Ok(Report {})
    }
}
