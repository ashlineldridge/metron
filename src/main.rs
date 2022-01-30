use crate::cli::{Cli, Command};
use anyhow::Result;
use serve::serve;
use test::test;

mod cli;
mod serve;
mod test;

fn main() -> Result<()> {
    use Command::*;

    let cli = Cli::parse_args();
    match cli.command {
        Test(cli) => test(&cli.into())?,
        Serve(cli) => serve(&cli.into())?,
    };

    Ok(())
}
