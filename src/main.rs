use crate::cli::{Cli, Command};
use anyhow::Result;
use serve::serve;
use test::run;

mod cli;
mod client;
mod error;
mod schedule;
mod serve;
mod signaller;
mod test;

fn main() -> Result<()> {
    use Command::*;

    let cli = Cli::parse_args();
    match cli.command {
        Test(cli) => {
            let res = run(&cli.into())?;
            println!("{:?}", res);
        }
        Serve(cli) => serve(&cli.into())?,
    };

    Ok(())
}
