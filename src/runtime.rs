use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use tokio::runtime;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Config {
    pub worker_threads: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            worker_threads: num_cpus::get(),
        }
    }
}

pub fn build(config: &Config) -> Result<runtime::Runtime> {
    if config.worker_threads > 0 {
        runtime::Builder::new_multi_thread()
            .worker_threads(config.worker_threads)
            .enable_all()
            .build()
            .context("Could not build multi-threaded runtime")
    } else {
        runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .context("Could not build single-threaded runtime")
    }
}
