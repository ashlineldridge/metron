use thiserror::Error;

use super::TestPlan;

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Unexpected(#[from] anyhow::Error),
}

pub struct Config {}

pub struct Runner {
    config: Config,
}

impl Runner {
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    pub async fn run(plan: TestPlan) -> Result<(), Error> {}
}
