//! HTTP client helpers using only the standard library.
//!
//! Builds raw HTTP/1.1 requests over `TcpStream`.
//! NOTE: HTTPS is not supported – use `http://` URLs only.

use crate::types::{KlyronError, Result};
use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::Duration;

fn parse_url(url: &str) -> Result<(String, u16, String)> {
    let url = url.trim();
    let without_scheme = if url.starts_with("https://") {
        return Err(KlyronError::Http(
            "HTTPS is not supported with std-only TcpStream; use http://".into(),
        ));
    } else if url.starts_with("http://") {
        &url[7..]
    } else {
        return Err(KlyronError::Http(
            "URL must start with http://".into(),
        ));
    };

    let (host_port, path) = match without_scheme.find('/') {
        Some(pos) => (&without_scheme[..pos], &without_scheme[pos..]),
        None => (without_scheme, "/"),
    };

    let (host, port) = match host_port.find(':') {
        Some(pos) => {
            let p: u16 = host_port[pos + 1..]
                .parse()
                .map_err(|_| KlyronError::Http("Invalid port".into()))?;
            (&host_port[..pos], p)
        }
        None => (host_port, 80u16),
    };

    Ok((host.to_string(), port, path.to_string()))
}

/// Perform an arbitrary HTTP request.
///
/// * `method`  – HTTP method (GET, POST, PUT, DELETE, …)
/// * `url`     – full URL (http:// only)
/// * `headers` – list of `(name, value)` pairs
/// * `body`    – optional request body
///
/// Returns the response body as a `String`.
pub fn request(
    method: &str,
    url: &str,
    headers: &[(&str, &str)],
    body: Option<&str>,
) -> Result<String> {
    let (host, port, path) = parse_url(url)?;
    let addr = format!("{}:{}", host, port);

    let mut stream = TcpStream::connect_timeout(
        &addr
            .parse()
            .map_err(|e| KlyronError::Http(format!("Invalid address: {}", e)))?,
        Duration::from_secs(10),
    )?;

    stream
        .set_read_timeout(Some(Duration::from_secs(10)))
        .ok();
    stream
        .set_write_timeout(Some(Duration::from_secs(10)))
        .ok();

    let mut request_line = format!("{} {} HTTP/1.1\r\nHost: {}\r\n", method, path, host);
    for (k, v) in headers {
        request_line.push_str(&format!("{}: {}\r\n", k, v));
    }
    if let Some(b) = body {
        request_line.push_str(&format!("Content-Length: {}\r\n", b.len()));
    }
    request_line.push_str("Connection: close\r\n\r\n");
    if let Some(b) = body {
        request_line.push_str(b);
    }

    stream.write_all(request_line.as_bytes())?;

    let mut raw = String::new();
    stream.read_to_string(&mut raw)?;

    // Split header block from body.
    let mut parts = raw.splitn(2, "\r\n\r\n");

    let status_line = parts.next().unwrap_or("");
    let body_text = parts.next().unwrap_or("");

    let status_code: u16 = status_line
        .split_whitespace()
        .nth(1)
        .unwrap_or("500")
        .parse()
        .unwrap_or(500);

    if status_code >= 400 {
        let preview = body_text.lines().next().unwrap_or("").to_string();
        return Err(KlyronError::Http(format!(
            "HTTP {} {}",
            status_code, preview
        )));
    }

    Ok(body_text.to_string())
}

/// HTTP GET request.
pub fn get(url: &str) -> Result<String> {
    request("GET", url, &[], None)
}

/// HTTP POST request with a JSON body.
pub fn post(url: &str, json_body: &str) -> Result<String> {
    request(
        "POST",
        url,
        &[("Content-Type", "application/json")],
        Some(json_body),
    )
}

/// Convenience wrapper around `get`.
pub fn fetch(url: &str) -> Result<String> {
    get(url)
}
