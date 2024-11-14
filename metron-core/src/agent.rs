use std::future::Future;

use anyhow::Result;

use crate::Plan;

pub trait Agent {
    fn exec(&self, plan: Plan) -> impl Future<Output = Result<()>> + Send;
    fn stop(&self) -> impl Future<Output = Result<()>> + Send;
    fn report(&self, kind: ReportKind) -> impl Future<Output = Result<Report>> + Send;
}

#[derive(Clone, Debug)]
pub enum ReportKind {
    DelayLatency,
    ResponseLatency,
    ErrorLatency,
}

#[derive(Clone, Debug)]
pub struct Report {
    pub data: hdrhistogram::Histogram<u64>,
}
