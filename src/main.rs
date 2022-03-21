use crate::plan::Builder;
use signaller::Signaller;
use std::time::Duration;
use wrkr::Rate;

mod cli;
mod client;
mod error;
mod plan;
mod serve;
mod signaller;
mod test;
mod wait;

#[tokio::main]
async fn main() {
    // Maximal throughput plan that runs for 60 seconds.
    let plan = Builder::new()
        .duration(Duration::from_secs(60))
        .build()
        .unwrap();

    // Create an on-demand signaller that always produces the current time
    // when a signal is requested.
    let signaller = Signaller::new_on_demand(plan);
    let sig = signaller.recv().await.unwrap();
    println!("The next request should be sent at {}", sig.due);

    // 100 RPS plan that runs for 60 seconds.
    let plan = Builder::new()
        .duration(Duration::from_secs(60))
        .rate(Rate(100))
        .build()
        .unwrap();

    // Create a blocking-thread signaller that uses a dedicated thread to place
    // signals on a channel at the appropriate time.
    let signaller = Signaller::new_blocking_thread(plan);
    let sig = signaller.recv().await.unwrap();
    println!("The next request should be sent at {}", sig.due);

    // Create a cooperative signaller that uses a dedicated task to place
    // signals on a channel at the appropriate time.
    let signaller = Signaller::new_blocking_thread(plan);
    let sig = signaller.recv().await.unwrap();
    println!("The next request should be sent at {}", sig.due);
}

// use crate::cli::{Cli, Command};
// use anyhow::Result;
// use serve::serve;
// use test::run;

// fn main() -> Result<()> {
//     use Command::*;

//     let cli = Cli::parse_args();
//     match cli.command {
//         Test(cli) => {
//             let res = run(&cli.into())?;
//             println!("{:?}", res);
//         }
//         Serve(cli) => serve(&cli.into())?,
//     };

//     Ok(())
// }
