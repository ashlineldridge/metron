use metron::LogLevel;

use crate::runtime;

#[derive(Clone, Debug)]
pub enum Config {
    Operator(crate::operator::Config),
    Echo(crate::echo::Config),
    Node(crate::node::Config),
    Profile(crate::profile::Config),
    Control(crate::control::Config),
}

impl Config {
    pub fn log_level(&self) -> LogLevel {
        match self {
            Config::Operator(c) => c.log_level,
            Config::Echo(c) => c.log_level,
            Config::Node(c) => c.log_level,
            Config::Profile(c) => c.log_level,
            Config::Control(c) => c.log_level,
        }
    }

    // TODO: This feels very unidiomatic. Fix.
    pub fn runtime(&self) -> &runtime::Config {
        match self {
            Config::Operator(c) => &c.runtime,
            Config::Echo(c) => &c.runtime,
            Config::Node(c) => &c.runtime,
            Config::Profile(c) => &c.runtime,
            Config::Control(c) => &c.runtime,
        }
    }
}
