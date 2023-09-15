//! Entry point for the main `metron` binary.

use anyhow::Result;
use metron::core::agent::Agent;

fn main() -> Result<()> {
    let agent = Agent {
        results_sink: todo!(),
    };

    Ok(())
}
