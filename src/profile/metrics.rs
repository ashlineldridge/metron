use std::collections::HashMap;

use anyhow::Result;
use prometheus::{proto::MetricFamily, BasicAuthentication};

use super::profiler::Sample;

pub struct Backend {}

impl Backend {
    pub async fn record(&mut self, _s: &Sample) -> Result<()> {
        Ok(())
    }

    #[allow(dead_code)]
    pub async fn record2(&mut self, _s: &Sample) -> Result<()> {
        // let job = "metron_job";
        // let grouping = HashMap::new();
        // let url = "http://localhost:9091";
        // let basic_auth = None;

        // let mut l = LabelPair::new();
        // l.set_name("something".into());
        // l.set_value("value1".into());

        // let mut c = Counter::new();
        // c.set_value(90.9);

        // let mut m = Metric::new();
        // m.set_label(RepeatedField::from_vec(vec![l]));
        // m.set_counter(c);

        // let mut mf = MetricFamily::new();
        // mf.set_name("metron_metric1".into());
        // mf.set_field_type(prometheus::proto::MetricType::COUNTER);
        // mf.set_help("A metric I made up is that OK".into());
        // mf.set_metric(RepeatedField::from_vec(vec![m]));

        // let mfs = vec![mf];

        // push(job, grouping, url, mfs, "POST", basic_auth).await?;

        Ok(())
    }

    // pub fn record2(&mut self, s: &Sample) -> Result<()> {
    // }
}

#[allow(dead_code)]
const LABEL_NAME_JOB: &str = "job";

#[allow(dead_code)]
async fn push(
    _job: &str,
    _grouping: HashMap<String, String>,
    _url: &str,
    _mfs: Vec<MetricFamily>,
    _method: &str,
    _basic_auth: Option<BasicAuthentication>,
) -> Result<()> {
    // Suppress clippy warning needless_pass_by_value.
    // let grouping = grouping;

    // let mut push_url = if url.contains("://") {
    //     url.to_owned()
    // } else {
    //     format!("http://{}", url)
    // };

    // if push_url.ends_with('/') {
    //     push_url.pop();
    // }

    // let mut url_components = Vec::new();
    // if job.contains('/') {
    //     bail!("job contains '/': {}", job);
    // }

    // // TODO: escape job
    // url_components.push(job.to_owned());

    // for (ln, lv) in &grouping {
    //     // TODO: check label name
    //     if lv.contains('/') {
    //         bail!("value of grouping label {} contains '/': {}", ln, lv);
    //     }
    //     url_components.push(ln.to_owned());
    //     url_components.push(lv.to_owned());
    // }

    // push_url = format!("{}/metrics/job/{}", push_url, url_components.join("/"));

    // let encoder = TextEncoder::new();
    // let mut buf = Vec::new();

    // for mf in mfs {
    //     // Check for pre-existing grouping labels:
    //     for m in mf.get_metric() {
    //         for lp in m.get_label() {
    //             if lp.get_name() == LABEL_NAME_JOB {
    //                 bail!(
    //                     "pushed metric {} already contains a job label",
    //                     mf.get_name()
    //                 );
    //             }
    //             if grouping.contains_key(lp.get_name()) {
    //                 bail!(
    //                     "pushed metric {} already contains grouping label {}",
    //                     mf.get_name(),
    //                     lp.get_name()
    //                 );
    //             }
    //         }
    //     }
    //     // Ignore error, `no metrics` and `no name`.
    //     let _ = encoder.encode(&[mf], &mut buf);
    // }

    // let https = HttpsConnector::new();
    // let client = Client::builder().build::<_, hyper::Body>(https);

    // let target_uri = push_url.parse::<hyper::Uri>()?;
    // let req = hyper::Request::builder()
    //     .method(method)
    //     .uri(target_uri)
    //     .header("Content-Type", encoder.format_type())
    //     .body(hyper::Body::from(buf))?;

    // let resp = client.request(req).await?;

    // match resp.status() {
    //     StatusCode::OK => Ok(()),
    //     StatusCode::ACCEPTED => Ok(()),
    //     _ => bail!(
    //         "unexpected status code {} while pushing to {}",
    //         resp.status(),
    //         push_url
    //     ),
    // }

    Ok(())
}
