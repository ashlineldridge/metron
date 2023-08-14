use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use tokio::runtime;

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum Config {
    MultiThreaded { worker_threads: usize },
    SingleThreaded,
}

impl Config {
    pub fn is_single_threaded(&self) -> bool {
        *self == Self::SingleThreaded
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::MultiThreaded {
            worker_threads: num_cpus::get(),
        }
    }
}

pub fn build(config: &Config) -> Result<runtime::Runtime> {
    match config {
        Config::MultiThreaded { worker_threads } => runtime::Builder::new_multi_thread()
            .worker_threads(*worker_threads)
            .enable_all()
            .build()
            .context("Could not build multi-threaded runtime"),
        Config::SingleThreaded => runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .context("Could not build single-threaded runtime"),
    }
}
