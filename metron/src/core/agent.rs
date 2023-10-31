use std::{
    future::Future,
    pin::Pin,
    sync::{Arc, Mutex},
    task::Poll,
    time::Instant,
};

use thiserror::Error;

// Aha! This is still an agent. It should be run in the same steps as it would be when
// deployed distributed. E.g.
// let config = cli::parse()?;
// match config {
//     RunConfig(agent_config: core::agent::Config, load_test: core::LoadTest, log_level: LogLevel) => {
//         let agent = Agent::new(agent_config);
//         let result = agent.run(load_test)?;
//         ...
//         // Is this right? Better way?
//     },
//     ...
// }
// let agent =
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Config {
    pub segments: Vec<PlanSegment>,
    pub connections: usize,
    pub http_method: HttpMethod,
    pub targets: Vec<Url>,
    pub headers: Vec<Header>,
    pub payload: Option<String>,
    pub runtime: runtime::Config,
    pub signaller_kind: SignallerKind,
    pub no_latency_correction: bool,
    pub stop_on_client_error: bool,
    pub stop_on_non_2xx: bool,
    pub log_level: LogLevel,
}

#[derive(Error, Debug)]
#[non_exhaustive]
pub enum Error {
    // TODO: Specify correct inner error type once you know what the contract looks like.
    #[error("could not write test results to sink")]
    SinkError(#[from] anyhow::Error),
}

#[derive(Clone)]
pub struct LoadTest {
    name: String,
}

#[derive(Debug)]
pub struct Sample {
    pub due: Instant,
    pub sent: Instant,
    pub done: Instant,
    // TODO: This will need to become a type parameter or enum or something.
    pub status: Result<u16, Error>,
}

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

///
#[derive(Clone)]
pub struct Agent<S> {
    sink: S,
}

impl<S> Agent<S>
where
    S: tower::Service<Sample> + Clone + 'static,
    S::Error: std::error::Error + Send + Sync + 'static,
{
    pub fn new(sink: S) -> Self {
        Self { sink }
    }

    pub async fn run(&mut self, _test: &LoadTest) -> Result<(), Error> {
        // TODO: Execute the test plan (essentially call old Profiler code)
        // and have code send the results to the results_sink service.
        let sample = Sample {
            due: Instant::now(),
            sent: Instant::now(),
            done: Instant::now(),
            status: Ok(200),
        };
        if let Err(e) = self.sink.call(sample).await {
            return Err(Error::SinkError(e.into()));
        }

        Ok(())
    }
}

/// `Service` implementation that invokes `Agent::run` directly.
impl<S> tower::Service<LoadTest> for Agent<S>
where
    S: tower::Service<Sample> + Clone + 'static,
    S::Error: std::error::Error + Send + Sync + 'static,
{
    type Response = ();
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(
        &mut self,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::result::Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: LoadTest) -> Self::Future {
        // TODO: Perhaps check if load test is currently running?
        let mut cloned = self.clone();
        Box::pin(async move { cloned.run(&req).await })
    }
}

#[derive(Clone)]
pub struct SimpleSink {
    // TODO: Replace this with histrograms or OTEL or whatever.
    counter: Arc<Mutex<u64>>,
}

// But what about the simple application of an in-mem metrics db?
// I guess I'd wrap it in an Arc which is Clone which seems correct.
impl SimpleSink {
    pub fn new() -> Self {
        Self {
            counter: Arc::new(Mutex::new(0)),
        }
    }

    async fn sink(&mut self, _res: Sample) -> anyhow::Result<()> {
        // TODO: Don't know why I can't use ? rather than unwrap() below.
        // Also should it be a Tokio Mutex so it doesn't lock the entire thread?
        let mut counter = self.counter.lock().unwrap();
        *counter += 1;
        Ok(())
    }
}

impl Default for SimpleSink {
    fn default() -> Self {
        Self::new()
    }
}

impl tower::Service<Sample> for SimpleSink {
    type Response = ();
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(&mut self, _cx: &mut std::task::Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Sample) -> Self::Future {
        let mut clone = self.clone();
        Box::pin(async move { clone.sink(req).await.map_err(Error::SinkError) })
    }
}

// pub struct GrpcServerAgent<S> {
//     inner: S,
//     port: u16,
// }

// impl<S> GrpcServerAgent<S> {
//     pub fn new(inner: S, port: u16) -> Self {
//         Self { inner, port }
//     }

//     // TODO: Change to use custom error for grpc package/crate
//     pub async fn run() -> anyhow::Result<()> {
//         // TODO: Actually start the gRPC server here.
//         Ok(())
//     }
// }

// // TODO: GrpcServerAgent potentially doesn't need to be a Service.
// // Have a look at Tonic as it will have preferences around  middlewear
// // already. Important thing is that the GrpcServerAgent wraps a
// // Service.
// /// `Service` implementation that runs a gRPC server.
// impl<R, S> tower::Service<R> for GrpcServerAgent<S>
// where
//     S: tower::Service<R> + Clone + 'static,
//     R: 'static,
// {
//     type Response = S::Response;
//     type Error = S::Error;
//     type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

//     fn poll_ready(
//         &mut self,
//         cx: &mut std::task::Context<'_>,
//     ) -> std::task::Poll<std::result::Result<(), Self::Error>> {
//         self.inner.poll_ready(cx)
//     }

//     fn call(&mut self, req: R) -> Self::Future {
//         // See https://docs.rs/tower/latest/tower/trait.Service.html#be-careful-when-cloning-inner-services.
//         let clone = self.inner.clone();
//         let mut inner = std::mem::replace(&mut self.inner, clone);
//         Box::pin(async move { inner.call(req).await })
//     }
// }

// pub struct LogLayer {}

// impl<S> tower::Layer<S> for LogLayer {
//     type Service = LogService<S>;

//     // fn layer(&self, service: S) -> Self::Service {
//     //     // LogService {
//     //     //     target: self.target,
//     //     //     service,
//     //     // }
//     // }
// }

// TODO: If the tower load balancing mechanism isn't an exact fit, could I write my own that follows same pattern?
// use tower::{balance::p2c::Balance, load::Load};

#[derive(Clone)]
pub struct Controller<S> {
    agents: Vec<Agent<S>>,
}

/// `Service` implementation that balances load across agetns.
impl<S> tower::Service<LoadTest> for Controller<S>
where
    S: tower::Service<Sample> + Clone + 'static,
    S::Error: std::error::Error + Send + Sync + 'static,
{
    type Response = ();
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(
        &mut self,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::result::Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, _req: LoadTest) -> Self::Future {
        // TODO: Balance load across the agents.
        todo!()
    }
}

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
