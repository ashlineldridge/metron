use crate::{Agent, Plan, Report, Sink};

#[derive(Clone, Copy, Debug)]
pub enum SignallerKind {
    Dedicated,
    Cooperative,
}

#[derive(Clone)]
#[allow(unused)]
pub struct Runner {
    pub name: String,
    pub signaller: SignallerKind,
    pub worker_threads: usize,
    pub sinks: Vec<Sink>,
}

impl Agent for Runner {
    async fn test(&mut self, _plan: &Plan) -> Result<(), crate::AgentError> {
        Ok(())
    }

    async fn cancel(&mut self) -> Result<(), crate::AgentError> {
        Ok(())
    }

    async fn report(&mut self) -> Result<Report, crate::AgentError> {
        Ok(Report {})
    }

    async fn proxy(&mut self) -> Result<Report, crate::AgentError> {
        Ok(Report {})
    }
}

// pub async fn test(&mut self, _plan: &Plan) -> Result<(), AgentError> {
//     info!("running test plan");
//     Ok(())
// }

// pub async fn cancel(&mut self) -> Result<(), AgentError> {
//     info!("cancelling test");
//     Ok(())
// }

// pub async fn report(&self) -> Result<Report, AgentError> {
//     info!("cancelling test");
//     Ok(Report {})
// }

// // pub async fn proxy(&mut self)
