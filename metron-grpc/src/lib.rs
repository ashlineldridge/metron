mod proto {
    tonic::include_proto!("proto");
}

use std::{future::Future, net::AddrParseError, pin::Pin, task::Poll, time::Duration};

use anyhow::Context;
use metron_core::{Action, HttpMethod, Plan, RateSegment};
use thiserror::Error;
use tokio_stream::{Stream, StreamExt};
use tonic::{Request, Response, Status, Streaming};
use tower::Service;

#[derive(Clone)]
pub struct AgentClient {
    inner: proto::agent_client::AgentClient<tonic::transport::Channel>,
}

impl AgentClient {
    pub async fn connect(server_addr: String) -> Result<Self, Error> {
        let inner = proto::agent_client::AgentClient::connect(server_addr).await?;
        Ok(Self { inner })
    }
}

impl AgentClient {
    // TODO(NEXT): Add support for the other RPC methods. Below just demonstrates the Control RPC.
    async fn run(&mut self, _plan: &Plan) -> Result<(), Error> {
        let outbound = async_stream::stream! {
            let mut interval = tokio::time::interval(Duration::from_secs(1));

            loop {
                interval.tick().await;

                let request = proto::ProxyRequest {
                    plan: Some(proto::Plan {
                        segments: vec![],
                        actions: vec![],
                    }),
                    start_time: None,
                };

                yield request;
            }
        };

        // TODO: Remove unwraps.
        let response = self.inner.proxy(Request::new(outbound)).await?;
        let mut inbound = response.into_inner();

        while let Some(res) = inbound.message().await? {
            println!("GOT METRON RESPONSE = {:?}", res);
        }

        Ok(())
    }
}

// TODO: We need a general AgentRequest enum that contains the different types
// then we need a Service implementation of AgentClient that pattern matches on
// that enum and calls the actual gRPC methods (i.e. control, run, stop, poll).
impl Service<Plan> for AgentClient {
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
pub struct AgentServer<S> {
    inner: S,
    port: u16,
}

impl<S> AgentServer<S>
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

        let server = proto::agent_server::AgentServer::new(self);

        println!("metron server listening on {}", address);
        tonic::transport::Server::builder()
            .add_service(server)
            .serve(address)
            .await?;

        Ok(())
    }
}

#[tonic::async_trait]
impl<S> proto::agent_server::Agent for AgentServer<S>
where
    S: Service<Plan> + Send + Sync + Clone + 'static,
    S::Error: std::fmt::Debug,
    S::Future: Send + 'static,
{
    type ProxyStream =
        Pin<Box<dyn Stream<Item = Result<proto::ProxyResponse, tonic::Status>> + Send + 'static>>;

    async fn test(
        &self,
        _request: Request<proto::TestRequest>,
    ) -> Result<Response<proto::TestResponse>, Status> {
        Ok(Response::new(proto::TestResponse {}))
    }

    async fn cancel(
        &self,
        _request: Request<proto::CancelRequest>,
    ) -> Result<Response<proto::CancelResponse>, Status> {
        Ok(Response::new(proto::CancelResponse {}))
    }

    async fn report(
        &self,
        _request: Request<proto::ReportRequest>,
    ) -> Result<Response<proto::ReportResponse>, tonic::Status> {
        Ok(Response::new(proto::ReportResponse { stats: None }))
    }

    async fn proxy(
        &self,
        request: Request<Streaming<proto::ProxyRequest>>,
    ) -> Result<Response<Self::ProxyStream>, tonic::Status> {
        let mut stream = request.into_inner();

        let mut inner = self.inner.clone();
        let output = async_stream::try_stream! {
            while let Some(req) = stream.next().await {
                let req = req?;
                let plan = req.plan.ok_or_else(|| tonic::Status::invalid_argument("missing plan"))?;
                let plan: Plan = plan.try_into().unwrap();
                let _target = "TODO".to_string();

                inner.call(plan).await.expect("service call failed");

                yield proto::ProxyResponse { };
            }
        };

        Ok(Response::new(Box::pin(output) as Self::ProxyStream))
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
            name: "TODO".to_owned(),
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
                let duration = s.duration.map(TryInto::try_into).transpose()?;
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
                    name: "TODO".to_owned(),
                    action: Some(proto::action::Action::HttpAction(proto::HttpAction {
                        method,
                        headers,
                        payload: payload.into_bytes(),
                        target: target.to_string(),
                    })),
                }
            }
            Action::Udp { payload, target } => Self {
                name: "TODO".to_owned(),
                action: Some(proto::action::Action::UdpAction(proto::UdpAction {
                    payload: payload.into_bytes(),
                    target: target.to_string(),
                })),
            },
            Action::Exec { command, args, env } => Self {
                name: "TODO".to_owned(),
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
                let method = proto::HttpMethod::try_from(a.method)
                    .context("invalid HTTP method")?
                    .try_into()?;
                Self::Http {
                    method,
                    headers: a.headers,
                    payload: "TODO".to_owned(),
                    target: a.target.parse()?,
                }
            }
            proto::action::Action::UdpAction(a) => Self::Udp {
                payload: "TODO".to_owned(),
                target: a.target.parse()?,
            },
            proto::action::Action::ExecAction(a) => Self::Exec {
                command: a.command,
                args: a.args,
                env: a.env,
            },
            proto::action::Action::WasmAction(_) => Self::Wasm {},
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
