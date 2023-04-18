use std::process::Command;

use assert_cmd::prelude::*;

#[test]
fn simple_profile() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("metron")?;

    cmd.arg("profile")
        .arg("--rate=10")
        .arg("--duration=5s")
        .arg("--target=https://httpbin.org/get");

    cmd.assert().success();

    Ok(())
}
