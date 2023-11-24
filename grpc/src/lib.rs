mod proto {
    tonic::include_proto!("proto");
}

use std::{future::Future, net::AddrParseError, pin::Pin, task::Poll, time::Duration};

use anyhow::Context;
use metron::{Action, HttpMethod, Plan, RateSegment};
use thiserror::Error;
use tokio_stream::{Stream, StreamExt};
use tonic::{Request, Response, Streaming};
use tower::Service;

#[derive(Clone)]
pub struct MetronClient {
    inner: proto::metron_client::MetronClient<tonic::transport::Channel>,
}

impl MetronClient {
    pub async fn connect(server_addr: String) -> Result<Self, Error> {
        let inner = proto::metron_client::MetronClient::connect(server_addr).await?;

        Ok(Self { inner })
    }
}

//TODO****NEXT: Flesh out Plan and gRPC Plan

// TODO: I want the client (always run as `metron test` at the moment)
// to have the option of running in "attached" and "detached" modes.
// If you don't specify external agents then you must run in attached
// mode (if you are running the controller or the runner then you also
// must run in attached mode - this TODO really only applies to `metron test`).
// Not yet sure how this will be implemented in the CLI - i.e. whether
// it should be an arg (e.g. `metron test -r 500 -d 10m --interactive http://foo.com` - like `docker run -i`)
// or whether `metron test` should just attach by
// So, there should be a cohesive user experience. Let's start by making
// `metron test` attach by default and stream (or be able to stream) the results
// to stdout. It should also be possible to detach and attach to the `metron test`
// process. Perhaps it could actually be a shell by default and you can run
// in detached mode with --detach.
//
// Got it! So when you run metron as an all-in-one and then detach - you
// are left with the exact same thing as if you just run the controller.
// Note: that does also mean that the controller needs to be able to run
// with a single local runner. Why would you ever want more than one
// local runner? If no benefit allow it at the code level but disallow
// it via config.

// There should be a command `metron attach` that can be used to attach
// (i.e. plug in to) any running metron process. A metron process can
// only be a controller or a runner. When you run `metron test` and
// specify a local runner, what's happening is that a Metron controller
// is being started as a server and the local process is attaching to
// it on the configured port. You can detach and re-attach as you please
// (a number of clients could). Start by making it read only (i.e. the
// attached client just streams results from the local controller/runner
//
// This is good but might change the config a bit. Prob for the better!
//
// Regardless, the MetronClient needs a way to a) stream updates from the
// server to the client and b) send instructions to the server and c)
impl MetronClient {
    async fn run(&mut self, plan: &Plan) -> Result<(), Error> {
        let outbound = async_stream::stream! {
            let mut interval = tokio::time::interval(Duration::from_secs(1));

            loop {
                interval.tick().await;

                let request = proto::MetronRequest {
                    plan: Some(proto::Plan {
                        segments: vec![],
                        actions: vec![],
                    }),
                };

                yield request;
            }
        };

        // TODO: Remove unwraps.
        let response = self.inner.run(Request::new(outbound)).await?;
        let mut inbound = response.into_inner();

        while let Some(res) = inbound.message().await? {
            println!("GOT METRON RESPONSE = {:?}", res);
        }

        Ok(())
    }
}

impl Service<Plan> for MetronClient {
    type Response = ();
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(
        &mut self,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::result::Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Plan) -> Self::Future {
        let mut metron = self.clone();
        Box::pin(async move { metron.run(&req).await })
    }
}

#[derive(Clone)]
pub struct MetronServer<S> {
    inner: S,
    port: u16,
}

impl<S> MetronServer<S>
where
    S: Service<Plan> + Send + Sync + Clone + 'static,
    S::Error: std::fmt::Debug, // This can be removed once proper error handling is in place
    S::Future: Send + 'static,
{
    pub fn new(inner: S, port: u16) -> Self {
        Self { inner, port }
    }

    pub async fn listen(self) -> Result<(), Error> {
        let address = format!("[::1]:{}", self.port)
            .parse()
            .map_err(|e: AddrParseError| Error::Unexpected(e.into()))?;

        let server = proto::metron_server::MetronServer::new(self);

        println!("metron server listening on {}", address);
        tonic::transport::Server::builder()
            .add_service(server)
            .serve(address)
            .await?;

        Ok(())
    }
}

#[tonic::async_trait]
impl<S> proto::metron_server::Metron for MetronServer<S>
where
    S: Service<Plan> + Send + Sync + Clone + 'static,
    S::Error: std::fmt::Debug,
    S::Future: Send + 'static,
{
    type RunStream =
        Pin<Box<dyn Stream<Item = Result<proto::MetronResponse, tonic::Status>> + Send + 'static>>;

    async fn run(
        &self,
        request: Request<Streaming<proto::MetronRequest>>,
    ) -> Result<Response<Self::RunStream>, tonic::Status> {
        let mut stream = request.into_inner();

        let mut inner = self.inner.clone();
        let output = async_stream::try_stream! {
            while let Some(req) = stream.next().await {
                let req = req?;
                let plan = req.plan.ok_or_else(|| tonic::Status::invalid_argument("missing plan"))?;
                let plan: Plan = plan.try_into().unwrap();
                let target = "TODO".to_string();

                inner.call(plan).await.expect("service call failed");

                yield proto::MetronResponse { target };
            }
        };

        Ok(Response::new(Box::pin(output) as Self::RunStream))
    }
}

impl TryFrom<Plan> for proto::Plan {
    type Error = anyhow::Error;

    fn try_from(value: Plan) -> Result<Self, Self::Error> {
        let segments = value
            .segments
            .iter()
            .map(|s| s.clone().try_into())
            .collect::<anyhow::Result<Vec<proto::RateSegment>>>()?;
        let actions = value
            .actions
            .iter()
            .map(|a| a.clone().try_into())
            .collect::<anyhow::Result<Vec<proto::Action>>>()?;

        Ok(proto::Plan { segments, actions })
    }
}

impl TryFrom<proto::Plan> for Plan {
    // TODO: These are good examples of where need to be thinking whether this
    // should be in the public API of this crate.
    type Error = anyhow::Error;

    fn try_from(value: proto::Plan) -> Result<Self, Self::Error> {
        let segments = value
            .segments
            .iter()
            .map(|s| s.clone().try_into())
            .collect::<anyhow::Result<Vec<RateSegment>>>()?;
        let actions = value
            .actions
            .iter()
            .map(|a| a.clone().try_into())
            .collect::<anyhow::Result<Vec<Action>>>()?;

        Ok(Plan { segments, actions })
    }
}

impl TryFrom<RateSegment> for proto::RateSegment {
    type Error = anyhow::Error;

    fn try_from(value: RateSegment) -> Result<Self, Self::Error> {
        let segment = match value {
            RateSegment::Fixed { rate, duration } => {
                let duration = duration.map(TryInto::try_into).transpose()?;
                proto::rate_segment::Segment::FixedRateSegment(proto::FixedRateSegment {
                    rate,
                    duration,
                })
            }
            RateSegment::Linear {
                rate_start,
                rate_end,
                duration,
            } => {
                let duration = Some(duration.try_into()?);
                proto::rate_segment::Segment::LinearRateSegment(proto::LinearRateSegment {
                    rate_start,
                    rate_end,
                    duration,
                })
            }
        };

        Ok(proto::RateSegment {
            segment: Some(segment),
        })
    }
}

impl TryFrom<proto::RateSegment> for RateSegment {
    type Error = anyhow::Error;

    fn try_from(value: proto::RateSegment) -> Result<Self, Self::Error> {
        let segment = value.segment.as_ref().context("missing rate segment")?;
        let segment = match segment {
            proto::rate_segment::Segment::FixedRateSegment(s) => {
                let duration = s.duration.clone().map(TryInto::try_into).transpose()?;
                RateSegment::Fixed {
                    rate: s.rate,
                    duration,
                }
            }
            proto::rate_segment::Segment::LinearRateSegment(s) => RateSegment::Linear {
                rate_start: s.rate_start,
                rate_end: s.rate_end,
                duration: s
                    .duration
                    .clone()
                    .context("linear rate segments must specify a duration")?
                    .try_into()?,
            },
        };

        Ok(segment)
    }
}

impl TryFrom<Action> for proto::Action {
    type Error = anyhow::Error;

    fn try_from(value: Action) -> Result<Self, Self::Error> {
        let action = match value {
            Action::Http {
                method,
                headers,
                payload,
                target,
            } => {
                let method = TryInto::<proto::HttpMethod>::try_into(method)? as i32;
                Self {
                    action: Some(proto::action::Action::HttpAction(proto::HttpAction {
                        method,
                        headers,
                        payload,
                        target: target.to_string(),
                    })),
                }
            }
            Action::Udp { payload, target } => Self {
                action: Some(proto::action::Action::UdpAction(proto::UdpAction {
                    payload,
                    target: target.to_string(),
                })),
            },
            Action::Exec { command, args, env } => Self {
                action: Some(proto::action::Action::ExecAction(proto::ExecAction {
                    command,
                    args,
                    env,
                })),
            },
            Action::Wasm {} => todo!(),
        };

        Ok(action)
    }
}

impl TryFrom<proto::Action> for Action {
    type Error = anyhow::Error;

    fn try_from(value: proto::Action) -> Result<Self, Self::Error> {
        let action = value.action.context("missing action")?;
        let action = match action {
            proto::action::Action::HttpAction(a) => {
                let method = proto::HttpMethod::from_i32(a.method)
                    .context("invalid HTTP method")?
                    .try_into()?;
                Self::Http {
                    method,
                    headers: a.headers,
                    payload: a.payload,
                    target: a.target.parse()?,
                }
            }
            proto::action::Action::UdpAction(a) => Self::Udp {
                payload: a.payload,
                target: a.target.parse()?,
            },
            proto::action::Action::ExecAction(a) => Self::Exec {
                command: a.command,
                args: a.args,
                env: a.env,
            },
            proto::action::Action::WasmAction(a) => Self::Wasm {},
        };

        Ok(action)
    }
}

impl TryFrom<HttpMethod> for proto::HttpMethod {
    type Error = anyhow::Error;

    fn try_from(value: HttpMethod) -> Result<Self, Self::Error> {
        Ok(match value {
            HttpMethod::Get => Self::Get,
            HttpMethod::Post => Self::Post,
            HttpMethod::Put => Self::Put,
            HttpMethod::Patch => Self::Patch,
            HttpMethod::Delete => Self::Delete,
            HttpMethod::Head => Self::Head,
            HttpMethod::Options => Self::Options,
            HttpMethod::Trace => Self::Trace,
            HttpMethod::Connect => Self::Connect,
        })
    }
}

impl TryFrom<proto::HttpMethod> for HttpMethod {
    type Error = anyhow::Error;

    fn try_from(value: proto::HttpMethod) -> Result<Self, Self::Error> {
        Ok(match value {
            proto::HttpMethod::Get => HttpMethod::Get,
            proto::HttpMethod::Post => HttpMethod::Post,
            proto::HttpMethod::Put => HttpMethod::Put,
            proto::HttpMethod::Patch => HttpMethod::Patch,
            proto::HttpMethod::Delete => HttpMethod::Delete,
            proto::HttpMethod::Head => HttpMethod::Head,
            proto::HttpMethod::Options => HttpMethod::Options,
            proto::HttpMethod::Trace => HttpMethod::Trace,
            proto::HttpMethod::Connect => HttpMethod::Connect,
        })
    }
}

// TODO: Create a separate MetronClientError and a MetronServerError
// following best practices.
#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    TransportError(#[from] tonic::transport::Error),

    #[error(transparent)]
    StatusError(#[from] tonic::Status),

    #[error(transparent)]
    Unexpected(#[from] anyhow::Error),
}
