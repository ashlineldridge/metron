use metron::LogLevel;

pub enum Config {
    Load(crate::profile::Config),
    Server(crate::server::Config),
}

impl Config {
    pub fn log_level(&self) -> LogLevel {
        match self {
            Config::Load(c) => c.log_level,
            Config::Server(c) => c.log_level,
        }
    }
}
