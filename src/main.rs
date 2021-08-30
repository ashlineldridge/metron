use argh::FromArgs;
use hyper::Client;
use hyper_tls::HttpsConnector;

#[derive(FromArgs, Debug)]
/// Your new favorite load testing tool.
struct Cli {
    /// queries per second (defaults to 0 which implies maximum)
    #[argh(option, default = "0")]
    qps: u64,

    /// number of threads (defaults to 4)
    #[argh(option, default = "4")]
    threads: u64,

    /// test duration (defaults to 0s which implies forever)
    #[argh(
        option,
        default = "humantime::Duration::from(std::time::Duration::ZERO)"
    )]
    duration: humantime::Duration,

    /// target URL
    #[argh(positional)]
    target: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let cli: Cli = argh::from_env();
    println!("CLI arguments: {:?}", cli);

    let uri = cli.target.parse()?;
    let https = HttpsConnector::new();
    let client = Client::builder().build::<_, hyper::Body>(https);
    // let client = Client::new();
    let resp = client.get(uri).await?;

    println!("Response status: {}", resp.status());

    Ok(())
}
