#[derive(Clone, Debug, Default)]
pub struct Config {
    pub log_level: LogLevel,
}

#[derive(Clone, Copy, Debug)]
pub enum LogLevel {
    Off,
    Info,
    Debug,
    Warn,
    Error,
}

impl Default for LogLevel {
    fn default() -> Self {
        Self::Off
    }
}
