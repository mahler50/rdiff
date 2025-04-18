use anyhow::Result;
use clap::Parser;
use dialoguer::theme::ColorfulTheme;
use dialoguer::{Input, MultiSelect};
use rdiff::cli::{Action, Args, RunArgs};
use rdiff::{DiffConfig, DiffProfile, ExtraArgs, RequestProfile, ResponseProfile};

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    match args.action {
        Action::Run(args) => run(args).await?,
        Action::Parse => parse().await?,
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

async fn parse() -> Result<()> {
    let theme = ColorfulTheme::default();
    let url1: String = Input::with_theme(&theme)
        .with_prompt("Url1")
        .interact_text()?;
    let url2: String = Input::with_theme(&theme)
        .with_prompt("Url2")
        .interact_text()?;
    let req1: RequestProfile = url1.parse()?;
    let req2: RequestProfile = url2.parse()?;
    // Send a preflight request to get the headers.
    let resp = req1.send(&ExtraArgs::default()).await?;
    let headers = resp.get_header_keys();

    let profile_name: String = Input::with_theme(&theme)
        .with_prompt("Profile name")
        .interact_text()?;

    let chosen = MultiSelect::with_theme(&theme)
        .with_prompt("Select headers to skip")
        .items(&headers)
        .interact()?;
    let skip_headers = chosen
        .iter()
        .map(|&i| headers[i].to_string())
        .collect::<Vec<_>>();

    let res = ResponseProfile::new(skip_headers, vec![]);
    let profile = DiffProfile::new(req1, req2, res);
    let config = DiffConfig::new(vec![(profile_name, profile)].into_iter().collect());
    let result = serde_yaml::to_string(&config)?;
    println!("---\n{}", result);

    Ok(())
}
