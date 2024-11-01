use serde::{Deserialize, Serialize};

/// Load testing report.
///
/// A [Report] describes the results of a (in-progress or complete) load test.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Report {}
