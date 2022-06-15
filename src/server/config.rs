use metron::LogLevel;
use serde::{Deserialize, Serialize};

use crate::runtime;

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Config {
    pub port: u16,
    pub runtime: runtime::Config,
    pub log_level: LogLevel,
}
