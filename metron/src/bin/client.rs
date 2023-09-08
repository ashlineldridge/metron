use agent::{Client, GrpcClient};
use anyhow::Result;
use proto::metron::{Segment, TestPlan};

#[tokio::main]
async fn main() -> Result<()> {
    let mut client = GrpcClient::connect("http://[::1]:8080").await?;

    let plan = TestPlan {
        segments: vec![Segment {
            segment: Some(proto::metron::segment::Segment::FixedRateSegment(
                proto::metron::FixedRateSegment {
                    rate: 100.0,
                    duration: "100s".to_owned(), // Should this be a common type Duration?
                },
            )),
        }],
        target: "https://google.com".to_owned(),
    };

    let report = client.run(plan).await?;

    println!("Report: {:?}", report);

    Ok(())
}
