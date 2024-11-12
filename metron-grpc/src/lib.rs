mod client;
mod server;
pub(crate) mod proto {
    tonic::include_proto!("proto");
}

use anyhow::Context;
pub use client::*;
use metron_core::{Action, AgentRequest, HttpMethod, Plan, Segment};
pub use server::*;

// ----------------------------------------------------------------------------
// Agent Request

impl TryFrom<AgentRequest> for proto::ControlRequest {
    type Error = anyhow::Error;

    fn try_from(value: AgentRequest) -> Result<Self, Self::Error> {
        TryFrom::try_from(&value)
    }
}

impl TryFrom<&AgentRequest> for proto::ControlRequest {
    type Error = anyhow::Error;

    fn try_from(value: &AgentRequest) -> Result<Self, Self::Error> {
        Ok(Self {
            plan: Some(TryInto::try_into(&value.plan)?),
            start: Some(value.start.into()),
        })
    }
}

impl TryFrom<proto::ControlRequest> for AgentRequest {
    type Error = anyhow::Error;

    fn try_from(value: proto::ControlRequest) -> Result<Self, Self::Error> {
        TryFrom::try_from(&value)
    }
}

impl TryFrom<&proto::ControlRequest> for AgentRequest {
    type Error = anyhow::Error;

    fn try_from(value: &proto::ControlRequest) -> Result<Self, Self::Error> {
        let plan = value.plan.as_ref().context("missing plan")?.try_into()?;
        let start = value.start.context("missing plan start time")?.into();
        Ok(Self { plan, start })
    }
}

// ----------------------------------------------------------------------------
// Test Plan

impl TryFrom<Plan> for proto::Plan {
    type Error = anyhow::Error;

    fn try_from(value: Plan) -> Result<Self, Self::Error> {
        TryFrom::try_from(&value)
    }
}

impl TryFrom<&Plan> for proto::Plan {
    type Error = anyhow::Error;

    fn try_from(value: &Plan) -> Result<Self, Self::Error> {
        let segments = value
            .segments
            .iter()
            .map(|s| s.try_into())
            .collect::<anyhow::Result<Vec<proto::Segment>>>()?;
        let actions = value
            .actions
            .iter()
            .map(|a| a.try_into())
            .collect::<anyhow::Result<Vec<proto::Action>>>()?;

        Ok(proto::Plan {
            name: value.name.clone(),
            segments,
            actions,
        })
    }
}

impl TryFrom<proto::Plan> for Plan {
    type Error = anyhow::Error;

    fn try_from(value: proto::Plan) -> Result<Self, Self::Error> {
        TryFrom::try_from(&value)
    }
}

impl TryFrom<&proto::Plan> for Plan {
    type Error = anyhow::Error;

    fn try_from(value: &proto::Plan) -> Result<Self, Self::Error> {
        let segments = value
            .segments
            .iter()
            .map(|s| s.try_into())
            .collect::<anyhow::Result<Vec<Segment>>>()?;
        let actions = value
            .actions
            .iter()
            .map(|a| a.try_into())
            .collect::<anyhow::Result<Vec<Action>>>()?;

        Ok(Plan {
            name: value.name.clone(),
            segments,
            actions,
        })
    }
}

impl TryFrom<Segment> for proto::Segment {
    type Error = anyhow::Error;

    fn try_from(value: Segment) -> Result<Self, Self::Error> {
        TryFrom::try_from(&value)
    }
}

impl TryFrom<&Segment> for proto::Segment {
    type Error = anyhow::Error;

    fn try_from(value: &Segment) -> Result<Self, Self::Error> {
        Ok(Self {
            name: value.name.clone(),
            rate_start: value.rate_start,
            rate_end: value.rate_end,
            duration: Some(value.duration.try_into()?),
        })
    }
}

impl TryFrom<proto::Segment> for Segment {
    type Error = anyhow::Error;

    fn try_from(value: proto::Segment) -> Result<Self, Self::Error> {
        TryFrom::try_from(&value)
    }
}

impl TryFrom<&proto::Segment> for Segment {
    type Error = anyhow::Error;

    fn try_from(value: &proto::Segment) -> Result<Self, Self::Error> {
        let segment = Segment {
            name: value.name.clone(),
            rate_start: value.rate_start,
            rate_end: value.rate_end,
            duration: value
                .duration
                .context("missing segment duration")?
                .try_into()?,
        };

        Ok(segment)
    }
}

impl TryFrom<Action> for proto::Action {
    type Error = anyhow::Error;

    fn try_from(value: Action) -> Result<Self, Self::Error> {
        TryFrom::try_from(&value)
    }
}

impl TryFrom<&Action> for proto::Action {
    type Error = anyhow::Error;

    fn try_from(value: &Action) -> Result<Self, Self::Error> {
        let action = match value.clone() {
            Action::Http {
                name,
                method,
                headers,
                payload,
                target,
            } => Self {
                name,
                action: Some(proto::action::Action::HttpAction(proto::HttpAction {
                    method: TryInto::<proto::HttpMethod>::try_into(method)? as i32,
                    headers,
                    payload,
                    target: target.to_string(),
                })),
            },
            Action::Udp {
                name,
                payload,
                target,
            } => Self {
                name,
                action: Some(proto::action::Action::UdpAction(proto::UdpAction {
                    payload,
                    target: target.to_string(),
                })),
            },
            Action::Exec {
                name,
                command,
                args,
                env,
            } => Self {
                name,
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
        TryFrom::try_from(&value)
    }
}

impl TryFrom<&proto::Action> for Action {
    type Error = anyhow::Error;

    fn try_from(value: &proto::Action) -> Result<Self, Self::Error> {
        let action = value.action.clone().context("missing action")?;
        let action = match action {
            proto::action::Action::HttpAction(a) => Self::Http {
                name: value.name.clone(),
                method: proto::HttpMethod::try_from(a.method)?.try_into()?,
                headers: a.headers,
                payload: a.payload,
                target: a.target.parse()?,
            },
            proto::action::Action::UdpAction(a) => Self::Udp {
                name: value.name.clone(),
                payload: a.payload,
                target: a.target.parse()?,
            },
            proto::action::Action::ExecAction(a) => Self::Exec {
                name: value.name.clone(),
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
        TryFrom::try_from(&value)
    }
}

impl TryFrom<&HttpMethod> for proto::HttpMethod {
    type Error = anyhow::Error;

    fn try_from(value: &HttpMethod) -> Result<Self, Self::Error> {
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
        TryFrom::try_from(&value)
    }
}

impl TryFrom<&proto::HttpMethod> for HttpMethod {
    type Error = anyhow::Error;

    fn try_from(value: &proto::HttpMethod) -> Result<Self, Self::Error> {
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
