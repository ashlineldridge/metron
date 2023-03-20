use std::process::Command;

use assert_cmd::prelude::*; // Add methods on commands
// use predicates::prelude::*; // Used for writing assertions // Run programs

#[test]
fn simple_profile() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("metron")?;

    cmd
        .arg("profile")
        .arg("--rate=100")
        .arg("--duration=5s")
        .arg("--target=http://localhost:9000");

    cmd.assert().success();

    Ok(())
}
