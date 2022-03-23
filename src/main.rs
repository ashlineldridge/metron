mod cli;
mod config;
mod error;
mod load;
mod server;
mod wait;

use anyhow::Result;

fn main() -> Result<()> {
    let _config = crate::cli::parse();

    Ok(())
}
