use wrkr::LogLevel;

pub struct Config {
    pub port: u16,
    pub worker_threads: usize,
    pub log_level: LogLevel,
}
