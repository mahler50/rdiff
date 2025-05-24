use rdiff::{LoadConfig, RequestConfig};

fn main() -> anyhow::Result<()> {
    let content = include_str!("../config/xreq_test.yaml");
    let config = RequestConfig::from_yaml(content)?;
    println!("{:#?}", config);

    Ok(())
}
