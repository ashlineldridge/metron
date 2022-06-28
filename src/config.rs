use metron::LogLevel;

use crate::runtime;

#[derive(Clone, Debug)]
pub enum Config {
    Profile(crate::profile::Config),
    Server(crate::server::Config),
}

impl Config {
    pub fn log_level(&self) -> LogLevel {
        match self {
            Config::Profile(c) => c.log_level,
            Config::Server(c) => c.log_level,
        }
    }

    pub fn runtime(&self) -> &runtime::Config {
        match self {
            Config::Profile(c) => &c.runtime,
            Config::Server(c) => &c.runtime,
        }
    }
}
