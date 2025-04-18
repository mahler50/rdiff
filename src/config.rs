use std::collections::HashMap;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::{ExtraArgs, RequestProfile, diff_text};

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

fn is_default<T: Default + PartialEq>(t: &T) -> bool {
    t == &T::default()
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

impl DiffConfig {
    pub fn new(profiles: HashMap<String, DiffProfile>) -> Self {
        Self { profiles }
    }

    /// Loads a `DiffConfig` from a YAML file.
    pub async fn load_yaml(path: &str) -> anyhow::Result<Self> {
        let content = tokio::fs::read_to_string(path).await?;
        Self::from_yaml(&content)
    }

    pub fn from_yaml(content: &str) -> anyhow::Result<Self> {
        let config: DiffConfig = serde_yaml::from_str(content)?;
        config.validate()?;
        Ok(config)
    }

    pub fn get_profile(&self, name: &str) -> Option<&DiffProfile> {
        self.profiles.get(name)
    }

    fn validate(&self) -> Result<()> {
        for (name, profile) in &self.profiles {
            profile
                .validate()
                .context(format!("failed to validate profile: {}", name))?;
        }

        Ok(())
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

    fn validate(&self) -> Result<()> {
        self.req1.validate().context("req1 validate failed")?;
        self.req2.validate().context("req2 validate failed")?;
        Ok(())
    }
}
