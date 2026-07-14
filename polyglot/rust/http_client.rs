use crate::types::{HttpResponse, KlyronError, Result};
use std::collections::BTreeMap;
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
        return Err(KlyronError::Http("URL must start with http://".into()));
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

    stream.set_read_timeout(Some(Duration::from_secs(10))).ok();
    stream.set_write_timeout(Some(Duration::from_secs(10))).ok();

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
        return Err(KlyronError::Http(format!("HTTP {} {}", status_code, preview)));
    }

    Ok(body_text.to_string())
}

pub fn request_full(method: &str, url: &str, headers: &[(&str, &str)], body: Option<&str>) -> Result<HttpResponse> {
    let (host, port, path) = parse_url(url)?;
    let addr = format!("{}:{}", host, port);

    let mut stream = TcpStream::connect_timeout(
        &addr.parse().map_err(|e| KlyronError::Http(format!("Invalid address: {}", e)))?,
        Duration::from_secs(10),
    )?;

    stream.set_read_timeout(Some(Duration::from_secs(10))).ok();
    stream.set_write_timeout(Some(Duration::from_secs(10))).ok();

    let mut req = format!("{} {} HTTP/1.1\r\nHost: {}\r\n", method, path, host);
    for (k, v) in headers {
        req.push_str(&format!("{}: {}\r\n", k, v));
    }
    if let Some(b) = body {
        req.push_str(&format!("Content-Length: {}\r\n", b.len()));
    }
    req.push_str("Connection: close\r\n\r\n");
    if let Some(b) = body {
        req.push_str(b);
    }

    stream.write_all(req.as_bytes())?;

    let mut raw = String::new();
    stream.read_to_string(&mut raw)?;

    let mut parts = raw.splitn(2, "\r\n\r\n");
    let header_block = parts.next().unwrap_or("");
    let body_text = parts.next().unwrap_or("");

    let mut lines = header_block.lines();
    let status_line = lines.next().unwrap_or("");
    let status_code: u16 = status_line.split_whitespace().nth(1).unwrap_or("500").parse().unwrap_or(500);

    let mut resp_headers = BTreeMap::new();
    for line in lines {
        if let Some(col) = line.find(':') {
            let key = line[..col].trim().to_string();
            let val = line[col + 1..].trim().to_string();
            resp_headers.insert(key, val);
        }
    }

    Ok(HttpResponse {
        status: status_code,
        status_text: status_line.split_whitespace().skip(2).collect::<Vec<&str>>().join(" "),
        headers: resp_headers,
        body: body_text.to_string(),
    })
}

pub fn get(url: &str) -> Result<String> {
    request("GET", url, &[], None)
}

pub fn post(url: &str, json_body: &str) -> Result<String> {
    request("POST", url, &[("Content-Type", "application/json")], Some(json_body))
}

pub fn put(url: &str, json_body: &str) -> Result<String> {
    request("PUT", url, &[("Content-Type", "application/json")], Some(json_body))
}

pub fn delete(url: &str) -> Result<String> {
    request("DELETE", url, &[], None)
}

pub fn patch(url: &str, json_body: &str) -> Result<String> {
    request("PATCH", url, &[("Content-Type", "application/json")], Some(json_body))
}

pub fn fetch(url: &str) -> Result<String> {
    get(url)
}
