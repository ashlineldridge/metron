use wrkr::LogLevel;

pub struct Config {
    pub port: u16,
    pub worker_threads: Option<usize>,
    pub log_level: LogLevel,
}
