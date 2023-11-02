use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Unexpected(#[from] anyhow::Error),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Config {}

pub struct Controller {
    config: Config,
}

impl Controller {
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    pub async fn run() -> Result<(), Error> {
        todo!()
    }
}

// TODO: Remove allow when you've got something to add.
#[allow(clippy::derivable_impls)]
impl Default for Config {
    fn default() -> Self {
        Self {}
    }
}
