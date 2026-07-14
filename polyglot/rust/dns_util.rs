use crate::types::{DnsRecord, KlyronError, Result};
use std::net::{SocketAddr, TcpStream, ToSocketAddrs};
use std::time::Duration;

pub fn resolve(hostname: &str) -> Result<Vec<String>> {
    let addr_str = format!("{}:0", hostname);
    let addrs: Vec<String> = addr_str
        .to_socket_addrs()?
        .map(|a| a.ip().to_string())
        .collect();
    if addrs.is_empty() {
        return Err(KlyronError::Dns(format!("Could not resolve {}", hostname)));
    }
    Ok(addrs)
}

pub fn resolve_ipv4(hostname: &str) -> Result<Vec<String>> {
    let addrs = format!("{}:0", hostname)
        .to_socket_addrs()?
        .filter(|a| a.is_ipv4())
        .map(|a| a.ip().to_string())
        .collect();
    Ok(addrs)
}

pub fn resolve_ipv6(hostname: &str) -> Result<Vec<String>> {
    let addrs = format!("{}:0", hostname)
        .to_socket_addrs()?
        .filter(|a| a.is_ipv6())
        .map(|a| a.ip().to_string())
        .collect();
    Ok(addrs)
}

pub fn is_reachable(host: &str, port: u16, timeout_secs: u64) -> bool {
    let addr = format!("{}:{}", host, port);
    match addr.to_socket_addrs() {
        Ok(mut addrs) => {
            if let Some(addr) = addrs.next() {
                TcpStream::connect_timeout(&addr, Duration::from_secs(timeout_secs)).is_ok()
            } else {
                false
            }
        }
        Err(_) => false,
    }
}

pub fn lookup_mx(hostname: &str) -> Result<Vec<DnsRecord>> {
    let output = std::process::Command::new("host")
        .args(["-t", "MX", hostname])
        .output()
        .map_err(|e| KlyronError::Dns(format!("failed to run host command: {}", e)))?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut records = Vec::new();
    for line in stdout.lines() {
        if line.contains("mail is handled by") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 6 {
                records.push(DnsRecord {
                    name: hostname.to_string(),
                    record_type: "MX".to_string(),
                    value: format!("{} {}", parts[5], parts[6]),
                    ttl: 0,
                });
            }
        }
    }
    Ok(records)
}
