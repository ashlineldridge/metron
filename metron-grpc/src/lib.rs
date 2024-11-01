mod client;
mod server;
pub(crate) mod proto {
    tonic::include_proto!("proto");
}

use anyhow::Context;
pub use client::*;
use metron_core::{Action, HttpMethod, Plan, RateSegment};
pub use server::*;

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
            .collect::<anyhow::Result<Vec<proto::RateSegment>>>()?;
        let actions = value
            .actions
            .iter()
            .map(|a| a.try_into())
            .collect::<anyhow::Result<Vec<proto::Action>>>()?;

        Ok(proto::Plan { segments, actions })
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
            .collect::<anyhow::Result<Vec<RateSegment>>>()?;
        let actions = value
            .actions
            .iter()
            .map(|a| a.try_into())
            .collect::<anyhow::Result<Vec<Action>>>()?;

        Ok(Plan { segments, actions })
    }
}

impl TryFrom<RateSegment> for proto::RateSegment {
    type Error = anyhow::Error;

    fn try_from(value: RateSegment) -> Result<Self, Self::Error> {
        TryFrom::try_from(&value)
    }
}

impl TryFrom<&RateSegment> for proto::RateSegment {
    type Error = anyhow::Error;

    fn try_from(value: &RateSegment) -> Result<Self, Self::Error> {
        let segment = match value {
            &RateSegment::Fixed { rate, duration } => {
                let duration = duration.map(TryInto::try_into).transpose()?;
                proto::rate_segment::Segment::FixedRateSegment(proto::FixedRateSegment {
                    rate,
                    duration,
                })
            }
            &RateSegment::Linear {
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
        TryFrom::try_from(&value)
    }
}

impl TryFrom<&proto::RateSegment> for RateSegment {
    type Error = anyhow::Error;

    fn try_from(value: &proto::RateSegment) -> Result<Self, Self::Error> {
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
        TryFrom::try_from(&value)
    }
}

impl TryFrom<&Action> for proto::Action {
    type Error = anyhow::Error;

    fn try_from(value: &Action) -> Result<Self, Self::Error> {
        let action = match value.clone() {
            Action::Http {
                method,
                headers,
                payload,
                target,
            } => Self {
                name: "TODO".to_owned(),
                action: Some(proto::action::Action::HttpAction(proto::HttpAction {
                    method: TryInto::<proto::HttpMethod>::try_into(method)? as i32,
                    headers,
                    payload,
                    target: target.to_string(),
                })),
            },
            Action::Udp { payload, target } => Self {
                name: "TODO".to_owned(),
                action: Some(proto::action::Action::UdpAction(proto::UdpAction {
                    payload,
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
        TryFrom::try_from(&value)
    }
}

impl TryFrom<&proto::Action> for Action {
    type Error = anyhow::Error;

    fn try_from(value: &proto::Action) -> Result<Self, Self::Error> {
        let action = value.action.clone().context("missing action")?;
        let action = match action {
            proto::action::Action::HttpAction(a) => {
                let method = proto::HttpMethod::try_from(a.method)
                    .context("invalid HTTP method")?
                    .try_into()?;
                Self::Http {
                    method,
                    headers: a.headers,
                    payload: "TODO".as_bytes().to_vec(),
                    target: a.target.parse()?,
                }
            }
            proto::action::Action::UdpAction(a) => Self::Udp {
                payload: "TODO".as_bytes().to_vec(),
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
