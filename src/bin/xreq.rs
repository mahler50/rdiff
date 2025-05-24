use anyhow::Result;
use clap::{Parser, Subcommand};
use dialoguer::Input;
use dialoguer::theme::ColorfulTheme;
use rdiff::cli::{KeyVal, parse_key_val};
use rdiff::{
    ExtraArgs, LoadConfig, RequestConfig, RequestProfile, get_body_text, get_headers_text,
    get_status_text, highlight_text,
};
use std::fmt::Write as _;

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
    let config = RequestConfig::load_yaml(&config_file).await?;
    let profile = config.get_profile(&args.profile).ok_or_else(|| {
        anyhow::anyhow!(
            "Profile {} not found in file {}",
            args.profile,
            &config_file
        )
    })?;

    let extra_args: ExtraArgs = args.extra_params.into();
    let url = profile.get_url(&extra_args)?;

    let resp = profile.send(&extra_args).await?.get_inner();

    let mut output = String::new();
    let status = get_status_text(&resp)?;
    let headers = get_headers_text(&resp, &[])?;
    let body = get_body_text(resp, &[]).await?;

    writeln!(&mut output, "Utl: {}\n", url)?;
    writeln!(&mut output, "{}", status)?;
    writeln!(
        &mut output,
        "{}",
        highlight_text(&headers, "yaml", Some("InspiredGitHub"))?
    )?;
    writeln!(&mut output, "{}", highlight_text(&body, "json", None)?)?;

    println!("{}", output);

    Ok(())
}

/// Parse config content from cli.
async fn parse() -> Result<()> {
    let theme = ColorfulTheme::default();
    let url: String = Input::with_theme(&theme)
        .with_prompt("Url")
        .interact_text()?;
    let profile: RequestProfile = url.parse()?;

    let name: String = Input::with_theme(&theme)
        .with_prompt("Profile name")
        .interact_text()?;

    let config = RequestConfig::new(vec![(name, profile)].into_iter().collect());

    let result = serde_yaml::to_string(&config)?;
    let highlighten_text = highlight_text(&result, "yaml", None)?;
    println!("{}", highlighten_text);

    Ok(())
}
