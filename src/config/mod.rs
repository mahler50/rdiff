mod rdiff;
mod xreq;

use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize, de::DeserializeOwned};

pub use rdiff::{DiffConfig, DiffProfile, ResponseProfile};
use std::fmt::{Debug, Write};
use std::str::FromStr;
pub use xreq::RequestConfig;

use async_trait::async_trait;
use reqwest::{
    Client, Method, Response,
    header::{self, HeaderMap, HeaderName, HeaderValue},
};
use serde_json::json;
use url::Url;

use crate::ExtraArgs;

#[async_trait]
pub trait LoadConfig
where
    Self: Sized + Validateable + DeserializeOwned + Debug,
{
    /// Loads a configuration from a YAML file.
    async fn load_yaml(path: &str) -> Result<Self> {
        let content = tokio::fs::read_to_string(path).await?;
        Self::from_yaml(&content)
    }

    /// Parse a YAML string into a configuration.
    fn from_yaml(content: &str) -> Result<Self> {
        let config: Self = serde_yaml::from_str(content)?;
        config.validate()?;
        Ok(config)
    }
}

pub trait Validateable {
    /// Validates the configuration.
    fn validate(&self) -> Result<()>;
}

pub fn is_default<T: Default + PartialEq>(t: &T) -> bool {
    t == &T::default()
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RequestProfile {
    /// HTTP method (GET, POST, etc.)
    /// Defaults to GET.
    #[serde(with = "http_serde::method", default)]
    pub method: Method,
    /// URL to send the request to.
    pub url: Url,
    /// Http request parameters.
    /// Defaults to None.
    #[serde(skip_serializing_if = "empty_json_value", default)]
    pub params: Option<serde_json::Value>,
    #[serde(
        with = "http_serde::header_map",
        skip_serializing_if = "HeaderMap::is_empty",
        default
    )]
    pub headers: HeaderMap,
    #[serde(skip_serializing_if = "empty_json_value", default)]
    pub body: Option<serde_json::Value>,
}

impl FromStr for RequestProfile {
    type Err = anyhow::Error;

    fn from_str(url: &str) -> Result<Self> {
        let mut url = Url::parse(url)?;
        let qs = url.query_pairs();
        let mut params = json!({});
        for (k, v) in qs {
            params[&*k] = v.into();
        }
        url.set_query(None);

        Ok(Self::new(
            Method::GET,
            url.to_string(),
            Some(params),
            HeaderMap::new(),
            None,
        ))
    }
}

#[derive(Debug)]
pub struct ResponseExt(Response);

impl RequestProfile {
    pub fn new(
        method: Method,
        url: String,
        params: Option<serde_json::Value>,
        headers: HeaderMap,
        body: Option<serde_json::Value>,
    ) -> Self {
        Self {
            method,
            url: Url::parse(&url).unwrap(),
            params,
            headers,
            body,
        }
    }

    /// Send request with current `RequestProfile`.
    /// Return an extension response.
    pub async fn send(&self, args: &ExtraArgs) -> Result<ResponseExt> {
        let (headers, query, body) = self.generate(args)?;
        let client = Client::new();

        let req = client
            .request(self.method.clone(), self.url.clone())
            .query(&query)
            .headers(headers)
            .body(body)
            .build()?;

        let resp = client.execute(req).await?;

        Ok(ResponseExt(resp))
    }

    /// Get specfic url string with query params.
    pub fn get_url(&self, args: &ExtraArgs) -> Result<String> {
        let mut url = self.url.clone();
        let (_, params, _) = self.generate(args)?;

        if !params.as_object().unwrap().is_empty() {
            let query = serde_qs::to_string(&params)?;
            url.set_query(Some(&query));
        }

        Ok(url.to_string())
    }

    /// Generate headers, query params and body with extra arguments.
    fn generate(&self, args: &ExtraArgs) -> Result<(HeaderMap, serde_json::Value, String)> {
        let mut headers = self.headers.clone();
        let mut query = self.params.clone().unwrap_or_else(|| json!({}));
        let mut body = self.body.clone().unwrap_or_else(|| json!({}));

        for (k, v) in &args.header {
            headers.insert(HeaderName::from_str(k)?, HeaderValue::from_str(v)?);
        }

        // If the content type is not set, default to application/json.
        if !headers.contains_key(header::CONTENT_TYPE) {
            headers.insert(
                header::CONTENT_TYPE,
                HeaderValue::from_static("application/json"),
            );
        }

        for (k, v) in &args.query {
            query[k] = v.as_str().into();
        }

        for (k, v) in &args.body {
            body[k] = v.as_str().into();
        }

        // Check the content type and serialize the body accordingly.
        let content_type = get_content_type(&headers);
        match content_type.as_deref() {
            Some("application/json") => {
                let body = serde_json::to_string(&body)?;
                Ok((headers, query, body))
            }
            Some("application/x-www-form-urlencoded" | "multipart/form-data") => {
                let body = serde_urlencoded::to_string(&body)?;
                Ok((headers, query, body))
            }
            _ => Err(anyhow!("Unsupported content type")),
        }
    }
}

impl Validateable for RequestProfile {
    fn validate(&self) -> Result<()> {
        if let Some(ref params) = self.params {
            if !params.is_object() {
                return Err(anyhow!(
                    "params must be an object but got: \n{}",
                    serde_yaml::to_string(params)?
                ));
            }
        }
        if let Some(ref body) = self.body {
            if !body.is_object() {
                return Err(anyhow!(
                    "body must be an object but got: \n{}",
                    serde_yaml::to_string(body)?
                ));
            }
        }

        Ok(())
    }
}

impl ResponseExt {
    pub fn get_inner(self) -> Response {
        self.0
    }

    pub async fn filter_text(self, profile: &ResponseProfile) -> Result<String> {
        let resp = self.0;
        let mut output = get_status_text(&resp)?;
        write!(
            &mut output,
            "{}",
            get_headers_text(&resp, &profile.skip_headers)?
        )?;
        write!(
            &mut output,
            "{}",
            get_body_text(resp, &profile.skip_body).await?
        )?;

        Ok(output)
    }

    pub fn get_header_keys(&self) -> Vec<String> {
        self.0
            .headers()
            .iter()
            .map(|(k, _)| k.to_string())
            .collect()
    }
}

pub fn get_status_text(resp: &Response) -> Result<String> {
    Ok(format!("{:?}:{}", resp.version(), resp.status()))
}

pub fn get_headers_text(resp: &Response, skip_headers: &[String]) -> Result<String> {
    let mut text = String::new();
    for (k, v) in resp.headers() {
        if !skip_headers.contains(&k.to_string()) {
            writeln!(&mut text, "{}: {:?}", k, v)?;
        }
    }
    writeln!(&mut text)?;

    Ok(text)
}

pub async fn get_body_text(resp: Response, skip_body: &[String]) -> Result<String> {
    let content_type = get_content_type(resp.headers());
    let text = resp.text().await?;
    match content_type.as_deref() {
        Some("application/json") => filter_json(&text, skip_body),
        _ => Ok(text),
    }
}

fn filter_json(text: &str, skip: &[String]) -> Result<String> {
    let mut json: serde_json::Value = serde_json::from_str(text)?;
    if let serde_json::Value::Object(ref mut obj) = json {
        for k in skip {
            obj.remove(k);
        }
    }
    Ok(serde_json::to_string_pretty(&json)?)
}

fn get_content_type(headers: &HeaderMap) -> Option<String> {
    headers
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().unwrap().split(";").next())
        .map(|v| v.to_string())
}

/// Check if the JSON value is null or empty object.
fn empty_json_value(v: &Option<serde_json::Value>) -> bool {
    v.as_ref()
        .is_none_or(|v| v.is_null() || (v.is_object() && v.as_object().unwrap().is_empty()))
}
