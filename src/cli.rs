use anyhow::{Result, anyhow};
use clap::{Parser, Subcommand};

use crate::ExtraArgs;

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeyValType {
    Query,
    Header,
    Body,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyVal {
    key_val_type: KeyValType,
    key: String,
    value: String,
}

/// Parse the key value pair from the command line arguments.
fn parse_key_val(s: &str) -> Result<KeyVal> {
    let mut parts = s.splitn(2, "=");
    let key = parts
        .next()
        .ok_or_else(|| anyhow!("Invalid key value pair"))?
        .trim();
    let value = parts
        .next()
        .ok_or_else(|| anyhow!("Invalid key value pair"))?
        .trim();

    let (key_val_type, key) = match key.chars().next() {
        Some('@') => (KeyValType::Body, &key[1..]),
        Some('%') => (KeyValType::Header, &key[1..]),
        Some(k) if k.is_alphabetic() => (KeyValType::Query, key),
        _ => return Err(anyhow!("Invalid key value pair")),
    };

    Ok(KeyVal {
        key_val_type,
        key: key.to_string(),
        value: value.to_string(),
    })
}

impl From<Vec<KeyVal>> for ExtraArgs {
    fn from(key_vals: Vec<KeyVal>) -> Self {
        let mut query = Vec::new();
        let mut header = Vec::new();
        let mut body = Vec::new();

        for key_val in key_vals {
            match key_val.key_val_type {
                KeyValType::Query => query.push((key_val.key, key_val.value)),
                KeyValType::Header => header.push((key_val.key, key_val.value)),
                KeyValType::Body => body.push((key_val.key, key_val.value)),
            }
        }

        ExtraArgs {
            query,
            header,
            body,
        }
    }
}
