use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    // TODO: Specify correct inner error type once you know what the contract looks like.
    #[error("could not write results to sink")]
    SinkError(#[from] anyhow::Error),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Config {
    pub port: u16,
}

pub struct Agent {
    config: Config,
}

impl Agent {
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    pub async fn run() -> Result<(), Error> {
        todo!()
    }
}

impl Default for Config {
    fn default() -> Self {
        // TODO***: How to share defaults between cli and core? IMO, ideally, core should
        // define the defaults since it's "business logic".
        Self { port: 9090 }
    }
}
