use metron::LogLevel;

use crate::runtime;

#[derive(Clone, Debug)]
pub struct Config {
    pub port: u16,
    pub runtime: runtime::Config,
    pub log_level: LogLevel,
}
