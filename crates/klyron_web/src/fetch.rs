use std::collections::HashMap;

use anyhow::{Context, Result};
use reqwest::Client;

use crate::{AbortSignal, Request, Response};

pub async fn fetch_url(url: &str) -> Result<Response> {
    let client = Client::new();
    let resp = client
        .get(url)
        .send()
        .await
        .context("failed to fetch URL")?;
    let status = resp.status().as_u16();
    let status_text = resp.status().canonical_reason().unwrap_or("").to_string();
    let headers: HashMap<String, String> = resp
        .headers()
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
        .collect();
    let body = resp.bytes().await.context("failed to read response body")?;

    Ok(Response {
        status,
        status_text,
        headers,
        body: body.to_vec(),
        url: url.to_string(),
    })
}

pub async fn fetch(req: &Request) -> Result<Response> {
    let client = Client::new();
    let mut builder = match req.method.to_uppercase().as_str() {
        "GET" => client.get(&req.url),
        "POST" => client.post(&req.url),
        "PUT" => client.put(&req.url),
        "DELETE" => client.delete(&req.url),
        "PATCH" => client.patch(&req.url),
        "HEAD" => client.head(&req.url),
        m => anyhow::bail!("unsupported method: {m}"),
    };

    for (k, v) in &req.headers {
        builder = builder.header(k, v);
    }

    if let Some(ref body) = req.body {
        builder = builder.body(body.clone());
    }

    if let Some(ref signal) = req.signal {
        if signal.is_aborted() {
            anyhow::bail!("request was aborted");
        }
    }

    let resp = builder.send().await.context("failed to send request")?;
    let status = resp.status().as_u16();
    let status_text = resp.status().canonical_reason().unwrap_or("").to_string();
    let headers: HashMap<String, String> = resp
        .headers()
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
        .collect();
    let body = resp.bytes().await.context("failed to read response body")?;

    Ok(Response {
        status,
        status_text,
        headers,
        body: body.to_vec(),
        url: req.url.clone(),
    })
}

pub async fn fetch_with_abort(url: &str, signal: AbortSignal) -> Result<Response> {
    if signal.is_aborted() {
        anyhow::bail!("request was aborted before sending");
    }
    fetch_url(url).await
}
