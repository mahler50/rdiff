use rdiff::DiffConfig;

fn main() -> anyhow::Result<()> {
    let content = include_str!("./test.yaml");
    let config = DiffConfig::from_yaml(content)?;
    println!("{:#?}", config);

    Ok(())
}
