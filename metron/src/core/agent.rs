use std::{future::Future, pin::Pin, task::Poll};

use thiserror::Error;

#[derive(Error, Debug)]
#[non_exhaustive]
pub enum Error {
    // TODO: Specify correct inner error type once you know what the contract looks like.
    #[error("could not write test results to sink")]
    SinkError(#[from] anyhow::Error),
}
#[derive(Clone)]
pub struct LoadTest {}

pub struct TestResult {}

// Agent is the actual thing that does the performance testing stuff
// Write a Service implementation for Agent
// Write another Service implementation that is also a gRPC server
// The gRPC server agent (Service implementation) is a wrapper around another Service (which would be the actual Agent)
// Write another Service implementation that is the gRPC client
// Write a Controller that does the actual controlling stuff (i.e. dividing up work and giving it to agents)
// Make Agents be given to the Controller as Service implementations - e.g. add_agent<A>(agent: A) where A: Service... (or equiv)
// This way the Controller can be given a plain agent to act in process and also gRPC client agents to act as "remote controls" of remote agents
// The gRPC server agent (Service implementation) is started by the "metron agent start --..." (or equiv)
// Agent should

#[derive(Clone)]
pub struct Agent<S> {
    results_sink: S,
}

impl<S> Agent<S>
where
    S: tower::Service<TestResult> + Clone + 'static,
    S::Error: std::error::Error + Send + Sync + 'static,
{
    // What does the agent return?
    // I want to give the agent a channel and have the agent send results in a typed format
    // down the channel.
    pub async fn run(&mut self, _test: LoadTest) -> Result<(), Error> {
        // TODO: Execute the test plan (essentially call old Profiler code)
        // and have code send the results to the results_sink service.
        let result = TestResult {};
        if let Err(e) = self.results_sink.call(result).await {
            return Err(Error::SinkError(e.into()));
        }

        Ok(())
    }
}

/// `Service` implementation that invokes
impl<S> tower::Service<LoadTest> for Agent<S>
where
    S: tower::Service<TestResult> + Clone + 'static,
    S::Error: std::error::Error + Send + Sync + 'static,
{
    type Response = ();
    type Error = Error;
    // TODO: Is this the Future type I should be using?
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(
        &mut self,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::result::Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: LoadTest) -> Self::Future {
        // TODO: Perhaps check if load test is currently running?
        let mut clone = self.clone();
        Box::pin(async move { clone.run(req).await })
    }
}

pub struct GrpcServerAgent<S> {
    inner: S,
}

/// `Service` implementation that invokes
impl<R, S> tower::Service<R> for GrpcServerAgent<S>
where
    S: tower::Service<R> + Clone + 'static,
    R: 'static,
    // S::Response: Into<()>,
    // S::Error: std::error::Error + Send + Sync + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    // TODO: Is this the Future type I should be using?
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(
        &mut self,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::result::Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: R) -> Self::Future {
        let clone = self.inner.clone();
        let mut inner = std::mem::replace(&mut self.inner, clone);
        Box::pin(async move { inner.call(req).await })
    }
}

// pub struct Controller<C> {
//     connect: C,
//     agents: Vec<Agent<>>,
// }

// // This is where I need to make the "breakthrough". Define the ADT model here.
// // What is a tower_service::Service? Why?

// impl<C> Controller<C> {
//     pub fn new(connect: C, agents: Vec<Agent>) -> Self {
//         Self { connect, agents }
//     }

//     // pub fn
// }

// impl<C> Controller<C>
// where
//     C: Connect,
// {
//     pub fn run(plan: &Plan) -> Result<()> {
//         Ok(())
//     }
// }

// pub trait Connect {}

// TODO: Try out the tower-test crate.
#[cfg(test)]
mod tests {
    // use super::*;

    #[test]
    fn it_works() {
        assert_eq!(1 + 1, 2);
    }
}
