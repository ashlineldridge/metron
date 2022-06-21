use std::{ops::Deref, str::FromStr, time::Duration};

use log::LevelFilter;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Deserialize, Serialize)]
#[serde(transparent)]
pub struct Rate(pub u32);

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

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Off,
    Error,
    Warn,
    Info,
    Debug,
}

impl LogLevel {
    pub fn as_filter(&self) -> LevelFilter {
        match self {
            LogLevel::Off => LevelFilter::Off,
            LogLevel::Error => LevelFilter::Error,
            LogLevel::Warn => LevelFilter::Warn,
            LogLevel::Info => LevelFilter::Info,
            LogLevel::Debug => LevelFilter::Debug,
        }
    }
}

impl Default for LogLevel {
    fn default() -> Self {
        Self::Off
    }
}

impl FromStr for LogLevel {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(serde_yaml::from_str(s)?)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Header {
    pub name: String,
    pub value: String,
}
