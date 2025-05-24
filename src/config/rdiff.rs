use super::{LoadConfig, RequestProfile, Validateable, is_default};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::{ExtraArgs, diff_text};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DiffConfig {
    #[serde(flatten)]
    pub profiles: HashMap<String, DiffProfile>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DiffProfile {
    pub req1: RequestProfile,
    pub req2: RequestProfile,
    #[serde(skip_serializing_if = "is_default", default)]
    pub resp: ResponseProfile,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq)]
pub struct ResponseProfile {
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub skip_headers: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub skip_body: Vec<String>,
}

impl ResponseProfile {
    pub fn new(skip_headers: Vec<String>, skip_body: Vec<String>) -> Self {
        Self {
            skip_headers,
            skip_body,
        }
    }
}

impl LoadConfig for DiffConfig {}

impl DiffConfig {
    pub fn new(profiles: HashMap<String, DiffProfile>) -> Self {
        Self { profiles }
    }

    pub fn get_profile(&self, name: &str) -> Option<&DiffProfile> {
        self.profiles.get(name)
    }
}

impl DiffProfile {
    pub fn new(req1: RequestProfile, req2: RequestProfile, resp: ResponseProfile) -> Self {
        Self { req1, req2, resp }
    }

    pub async fn diff(&self, args: ExtraArgs) -> Result<String> {
        let resp1 = self.req1.send(&args).await?;
        let resp2 = self.req2.send(&args).await?;

        let text1 = resp1.filter_text(&self.resp).await?;
        let text2 = resp2.filter_text(&self.resp).await?;

        diff_text(&text1, &text2)
    }
}

impl Validateable for DiffConfig {
    fn validate(&self) -> Result<()> {
        for (name, profile) in &self.profiles {
            profile
                .validate()
                .context(format!("failed to validate profile: {}", name))?;
        }

        Ok(())
    }
}

impl Validateable for DiffProfile {
    fn validate(&self) -> Result<()> {
        self.req1.validate().context("req1 validate failed")?;
        self.req2.validate().context("req2 validate failed")?;
        Ok(())
    }
}
