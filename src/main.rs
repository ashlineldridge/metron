use argh::FromArgs;
use hyper::Client;
use hyper_tls::HttpsConnector;

#[derive(FromArgs, Debug)]
/// Your new favorite performance characterization tool.
struct Cli {
    /// connections to keep open (defaults to 10)
    #[argh(option, short = 'c', default = "10")]
    connections: u64,

    /// number of threads (defaults to 4)
    #[argh(option, short = 't', default = "4")]
    threads: u64,

    /// work rate (throughput) in requests/second (defaults to 0 which implies maximum)
    #[argh(option, short = 'r', default = "0")]
    rate: u64,

    /// test duration (defaults to 0s which implies forever)
    #[argh(
        option,
        short = 'd',
        default = "humantime::Duration::from(std::time::Duration::ZERO)"
    )]
    duration: humantime::Duration,

    /// number of threads (defaults to 4)
    #[argh(option, short = 'h')]
    header: Vec<String>,

    /// print version information
    #[argh(switch, short = 'v')]
    version: bool,

    /// target URL
    #[argh(positional)]
    target: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let cli: Cli = argh::from_env();
    println!("CLI arguments: {:?}", cli);

    if cli.version {
        println!("wrk3 version 0.0.1");
        return Ok(());
    }

    match cli.target {
        Some(target) => {
            let uri = target.parse()?;
            let https = HttpsConnector::new();
            let client = Client::builder().build::<_, hyper::Body>(https);
            // let client = Client::new();
            let resp = client.get(uri).await?;

            println!("Response status: {}", resp.status());

            Ok(())
        }
        None => {
            panic!("No target specified");
        }
    }
}
