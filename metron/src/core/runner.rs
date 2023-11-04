use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::core::Plan;

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Unexpected(#[from] anyhow::Error),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Config {
    // TODO: I don't feel like this belongs here...
    // See how it plays out when we wire up the gRPC scenarios.
    pub plan: Plan,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            plan: Default::default(),
        }
    }
}

#[derive(Clone)]
pub struct Runner {
    config: Config,
}

impl Runner {
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    pub async fn run(&self, plan: &Plan) -> Result<(), Error> {
        println!("Plan: {:?}", plan);

        Ok(())
    }
}
