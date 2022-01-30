use anyhow::Result;
use hyper::Uri;

#[derive(Debug)]
pub struct TestConfig {
    pub connections: usize,
    pub threads: usize,
    pub rate: usize,
    pub duration: humantime::Duration,
    pub headers: Vec<Header>,
    pub target: Uri,
}

#[derive(Debug)]
pub struct Header {
    pub name: String,
    pub value: String,
}

pub fn test(config: &TestConfig) -> Result<()> {
    println!("{:?}", config);
    Ok(())
}
