mod client;
mod server;
pub(crate) mod proto {
    tonic::include_proto!("proto");
}

use std::io::Cursor;

use anyhow::Context;
pub use client::*;
use hdrhistogram::serialization::{Deserializer, Serializer, V2Serializer};
use metron_core::{Action, HttpMethod, Plan, Report, ReportKind, Segment};
pub use server::*;

// ----------------------------------------------------------------------------
// RPC Requests & Responses

// ----------------------------------------------------------------------------
// Report

impl TryFrom<Report> for proto::ReportResponse {
    type Error = anyhow::Error;

    fn try_from(value: Report) -> Result<Self, Self::Error> {
        TryFrom::try_from(&value)
    }
}

impl TryFrom<&Report> for proto::ReportResponse {
    type Error = anyhow::Error;

    fn try_from(value: &Report) -> Result<Self, Self::Error> {
        let mut vec = vec![];
        V2Serializer::new().serialize(&value.data, &mut vec)?;
        Ok(proto::ReportResponse { histogram: vec })
    }
}

impl TryFrom<proto::ReportResponse> for Report {
    type Error = anyhow::Error;

    fn try_from(value: proto::ReportResponse) -> Result<Self, Self::Error> {
        TryFrom::try_from(&value)
    }
}

impl TryFrom<&proto::ReportResponse> for Report {
    type Error = anyhow::Error;

    fn try_from(value: &proto::ReportResponse) -> Result<Self, Self::Error> {
        let mut cursor = Cursor::new(&value.histogram);
        let histogram = Deserializer::new().deserialize(&mut cursor)?;
        Ok(Report { data: histogram })
    }
}

impl TryFrom<ReportKind> for proto::ReportKind {
    type Error = anyhow::Error;

    fn try_from(value: ReportKind) -> Result<Self, Self::Error> {
        TryFrom::try_from(&value)
    }
}

impl TryFrom<&ReportKind> for proto::ReportKind {
    type Error = anyhow::Error;

    fn try_from(value: &ReportKind) -> Result<Self, Self::Error> {
        let kind = match value {
            ReportKind::DelayLatency => proto::ReportKind::DelayLatency,
            ReportKind::ResponseLatency => proto::ReportKind::ResponseLatency,
            ReportKind::ErrorLatency => proto::ReportKind::ErrorLatency,
        };

        Ok(kind)
    }
}

impl TryFrom<proto::ReportKind> for ReportKind {
    type Error = anyhow::Error;

    fn try_from(value: proto::ReportKind) -> Result<Self, Self::Error> {
        TryFrom::try_from(&value)
    }
}

impl TryFrom<&proto::ReportKind> for ReportKind {
    type Error = anyhow::Error;

    fn try_from(value: &proto::ReportKind) -> Result<Self, Self::Error> {
        let kind = match value {
            proto::ReportKind::DelayLatency => ReportKind::DelayLatency,
            proto::ReportKind::ResponseLatency => ReportKind::ResponseLatency,
            proto::ReportKind::ErrorLatency => ReportKind::ErrorLatency,
        };

        Ok(kind)
    }
}

// ----------------------------------------------------------------------------
// Plan

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
