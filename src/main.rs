mod args;
mod client;
mod error;
mod load;
mod plan;
mod server;
mod signaller;
mod wait;

use anyhow::Result;

fn main() -> Result<()> {
    let _args = crate::args::parse_clap();

    Ok(())
}
