use std::{ops::Deref, str::FromStr, time::Duration};

use clap::ValueEnum;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Deserialize, Serialize)]
#[serde(transparent)]
pub struct Rate(u32);

impl Rate {
    pub fn as_interval(&self) -> Duration {
        Duration::from_secs(1) / self.0
    }
}

impl std::fmt::Debug for Rate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl Deref for Rate {
    type Target = u32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FromStr for Rate {
    type Err = std::num::ParseIntError;

    fn from_str(s: &str) -> core::result::Result<Self, Self::Err> {
        Ok(Rate(u32::from_str(s)?))
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Off,
    Info,
    Debug,
    Warn,
    Error,
}

impl From<LogLevel> for log::LevelFilter {
    fn from(level: LogLevel) -> Self {
        match level {
            LogLevel::Off => log::LevelFilter::Off,
            LogLevel::Error => log::LevelFilter::Error,
            LogLevel::Warn => log::LevelFilter::Warn,
            LogLevel::Info => log::LevelFilter::Info,
            LogLevel::Debug => log::LevelFilter::Debug,
        }
    }
}

impl Default for LogLevel {
    fn default() -> Self {
        Self::Off
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Header {
    pub name: String,
    pub value: String,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Patch,
    Delete,
    Head,
    Options,
    Trace,
    Connect,
}

impl From<HttpMethod> for hyper::Method {
    fn from(method: HttpMethod) -> Self {
        match method {
            HttpMethod::Get => hyper::Method::GET,
            HttpMethod::Post => hyper::Method::POST,
            HttpMethod::Put => hyper::Method::PUT,
            HttpMethod::Patch => hyper::Method::PATCH,
            HttpMethod::Delete => hyper::Method::DELETE,
            HttpMethod::Head => hyper::Method::HEAD,
            HttpMethod::Options => hyper::Method::OPTIONS,
            HttpMethod::Trace => hyper::Method::TRACE,
            HttpMethod::Connect => hyper::Method::CONNECT,
        }
    }
}

impl Default for HttpMethod {
    fn default() -> Self {
        Self::Get
    }
}
