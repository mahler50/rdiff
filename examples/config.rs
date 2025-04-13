use rdiff::DiffConfig;

fn main() -> anyhow::Result<()> {
    let content = include_str!("../config/test.yaml");
    let config = DiffConfig::from_yaml(content)?;
    println!("{:#?}", config);

    Ok(())
}
