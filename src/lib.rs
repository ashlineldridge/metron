use std::{ops::Deref, str::FromStr, time::Duration};

use serde::{Serialize, Deserialize};

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

pub type LogLevel = log::LevelFilter;
