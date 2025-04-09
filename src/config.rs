use std::collections::HashMap;

use anyhow::Result;
use reqwest::{Method, header::HeaderMap};
use serde::{Deserialize, Serialize};
use url::Url;

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
pub struct RequestProfile {
    /// HTTP method (GET, POST, etc.)
    /// Defaults to GET.
    #[serde(with = "http_serde::method")]
    pub method: Method,
    /// URL to send the request to.
    pub url: Url,
    /// Http request parameters.
    /// Defaults to None.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub params: Option<serde_json::Value>,
    #[serde(
        with = "http_serde::header_map",
        skip_serializing_if = "HeaderMap::is_empty",
        default
    )]
    pub headers: HeaderMap,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub body: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ResponseProfile {
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub skip_headers: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub skip_body: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct DiffArgs {}

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
    pub async fn diff(&self, _args: DiffArgs) -> Result<String> {
        // let resp1 = self.req1.send(&args).await?;
        // let resp2 = self.req2.send(&args).await?;

        // let text1 = resp1.filter_text(&self.resp).await?;
        // let text2 = resp2.filter_text(&self.resp).await?;

        // diff_test(text1, text2)

        todo!()
    }
}
