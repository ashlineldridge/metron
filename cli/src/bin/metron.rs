//! Entry point for the main `metron` binary.

use anyhow::Result;
use metron::core::{DefaultSignaller, Signaller};

fn main() -> Result<()> {
    let signaller = DefaultSignaller {};
    signaller.run(100)?;

    Ok(())
}
