use anyhow::Result;
use clap::{Parser, Subcommand};
use dialoguer::theme::ColorfulTheme;
use dialoguer::{Input, MultiSelect};
use rdiff::cli::{KeyVal, parse_key_val};
use rdiff::{
    DiffConfig, DiffProfile, ExtraArgs, LoadConfig, RequestProfile, ResponseProfile, highlight_text,
};

/// Diff two requests and compare the difference of responses.
#[derive(Debug, Clone, Parser)]
#[clap(version, author, about, long_about = None)]
pub struct Args {
    #[clap(subcommand)]
    pub action: Action,
}

#[derive(Debug, Clone, Subcommand)]
#[non_exhaustive]
pub enum Action {
    /// Diff two API responses based on the given profile.
    Run(RunArgs),
    Parse,
}

#[derive(Debug, Clone, Parser)]
pub struct RunArgs {
    /// Profile name.
    #[clap(short, long, value_parser)]
    pub profile: String,

    /// Overrides args. Could be used to overrides params, headers and body of the request.
    /// For query params, use `-e key=value`.
    /// For headers, use `-e %key=value`.
    /// For body, use `-e @key=value`.
    #[clap(short, long, value_parser=parse_key_val, number_of_values=1)]
    pub extra_params: Vec<KeyVal>,

    /// Path to the YAML config file.
    #[clap(short, long, value_parser)]
    pub config: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    match args.action {
        Action::Run(args) => run(args).await?,
        Action::Parse => parse().await?,
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

/// Parse config content from cli.
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

    // Send a pre-flight request to get the headers.
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
    let highlighten_text = highlight_text(&result, "yaml", None)?;
    println!("{}", highlighten_text);

    Ok(())
}
