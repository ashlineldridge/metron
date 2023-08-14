use std::process::Command;

use assert_cmd::prelude::*;

#[test]
fn simple_profile() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("metron")?;

    // TODO: Start the echo server, then start the profiler
    // (rename to "run"?)
    // also target with k6 and set up the server with tracing to
    // be able to output metrics. Then measure the performance
    // capabilities and accuracy of both (using server metrics/tracing).
    //
    // metron
    cmd.arg("profile")
        .arg("--rate=10")
        .arg("--duration=5s")
        .arg("--target=https://httpbin.org/get");

    cmd.assert().success();

    Ok(())
}
