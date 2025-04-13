use anyhow::Result;
use clap::Parser;
use rdiff::cli::{Action, Args, RunArgs};
use rdiff::{DiffConfig, ExtraArgs};

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    match args.action {
        Action::Run(args) => run(args).await?,
        _ => panic!("Unsupported action"),
    }

    Ok(())
}

async fn run(args: RunArgs) -> Result<()> {
    let config_file = args.config.unwrap_or_else(|| "./rdiff.yaml".to_string());
    let config = DiffConfig::load_yaml(&config_file).await?;
    let profile = config.get_profile(&args.profile).ok_or_else(|| {
        anyhow::anyhow!(
            "Profile {} not found in file {}",
            args.profile,
            &config_file
        )
    })?;

    let extra_args: ExtraArgs = args.extra_params.into();

    let output = profile.diff(extra_args).await?;
    println!("{}", output);

    Ok(())
}
