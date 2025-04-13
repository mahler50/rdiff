use std::collections::HashMap;

use anyhow::Result;
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
    pub resp: ResponseProfile,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ResponseProfile {
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub skip_headers: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub skip_body: Vec<String>,
}

impl DiffConfig {
    /// Loads a `DiffConfig` from a YAML file.
    pub async fn load_yaml(path: &str) -> anyhow::Result<Self> {
        let content = tokio::fs::read_to_string(path).await?;
        Self::from_yaml(&content)
    }

    pub fn from_yaml(content: &str) -> anyhow::Result<Self> {
        let config: DiffConfig = serde_yaml::from_str(content)?;
        Ok(config)
    }

    pub fn get_profile(&self, name: &str) -> Option<&DiffProfile> {
        self.profiles.get(name)
    }
}

impl DiffProfile {
    pub async fn diff(&self, args: ExtraArgs) -> Result<String> {
        let resp1 = self.req1.send(&args).await?;
        let resp2 = self.req2.send(&args).await?;

        let text1 = resp1.filter_text(&self.resp).await?;
        let text2 = resp2.filter_text(&self.resp).await?;

        diff_text(&text1, &text2)
    }
}
