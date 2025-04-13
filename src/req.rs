use std::fmt::Write;
use std::str::FromStr;

use anyhow::{Result, anyhow};
use reqwest::{
    Client, Method, Response,
    header::{self, HeaderMap, HeaderName, HeaderValue},
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use url::Url;

use crate::{ExtraArgs, ResponseProfile};

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

#[derive(Debug)]
pub struct ResponseExt(Response);

impl RequestProfile {
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

impl ResponseExt {
    pub async fn filter_text(self, profile: &ResponseProfile) -> Result<String> {
        let resp = self.0;
        let mut output = get_headers_text(&resp, &profile.skip_headers)?;

        let content_type = get_content_type(resp.headers());
        let text = resp.text().await?;
        match content_type.as_deref() {
            Some("application/json") => {
                let text = filter_json(&text, &profile.skip_body)?;
                writeln!(&mut output, "{}", text)?;
            }
            _ => writeln!(&mut output, "{}", text)?,
        }

        Ok(output)
    }
}

fn get_headers_text(resp: &Response, skip_headers: &[String]) -> Result<String> {
    let mut text = String::new();
    writeln!(&mut text, "{:?}:{}", resp.version(), resp.status())?;
    for (k, v) in resp.headers() {
        if !skip_headers.contains(&k.to_string()) {
            writeln!(&mut text, "{}: {:?}", k, v)?;
        }
    }
    writeln!(&mut text)?;

    Ok(text)
}

fn filter_json(text: &str, skip: &[String]) -> Result<String> {
    let mut json: serde_json::Value = serde_json::from_str(text)?;
    match json {
        serde_json::Value::Object(ref mut obj) => {
            for k in skip {
                obj.remove(k);
            }
        }
        _ => {}
    }
    Ok(serde_json::to_string_pretty(&json)?)
}

fn get_content_type(headers: &HeaderMap) -> Option<String> {
    headers
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().unwrap().split(";").next())
        .map(|v| v.to_string())
}
