use anyhow::{Result, anyhow};

use crate::ExtraArgs;

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
pub fn parse_key_val(s: &str) -> Result<KeyVal> {
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
