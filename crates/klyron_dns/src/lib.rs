use std::net::{SocketAddr, ToSocketAddrs};
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct DnsRecord {
    pub name: String,
    pub address: String,
    pub record_type: String,
}

pub struct DnsResolver;

impl DnsResolver {
    pub fn new() -> Self { Self }

    pub fn resolve(&self, host: &str) -> anyhow::Result<Vec<DnsRecord>> {
        let addr_str = format!("{host}:0");
        let addrs: Vec<SocketAddr> = addr_str.to_socket_addrs()?
            .filter(|a| a.is_ipv4())
            .collect();

        if addrs.is_empty() {
            anyhow::bail!("Could not resolve hostname: {host}");
        }

        Ok(addrs.into_iter().map(|addr| DnsRecord {
            name: host.to_string(),
            address: addr.ip().to_string(),
            record_type: if addr.is_ipv4() { "A".to_string() } else { "AAAA".to_string() },
        }).collect())
    }

    pub fn resolve_ipv4(&self, host: &str) -> anyhow::Result<Vec<String>> {
        let addr_str = format!("{host}:0");
        let ips: Vec<String> = addr_str.to_socket_addrs()?
            .filter(|a| a.is_ipv4())
            .map(|a| a.ip().to_string())
            .collect();
        if ips.is_empty() {
            anyhow::bail!("No IPv4 address found for: {host}");
        }
        Ok(ips)
    }

    pub fn resolve_ipv6(&self, host: &str) -> anyhow::Result<Vec<String>> {
        let addr_str = format!("{host}:0");
        let ips: Vec<String> = addr_str.to_socket_addrs()?
            .filter(|a| a.is_ipv6())
            .map(|a| a.ip().to_string())
            .collect();
        if ips.is_empty() {
            anyhow::bail!("No IPv6 address found for: {host}");
        }
        Ok(ips)
    }

    pub fn is_reachable(&self, host: &str, port: u16) -> bool {
        use std::net::TcpStream;
        let addr = format!("{host}:{port}");
        TcpStream::connect_timeout(&addr.parse().unwrap_or_else(|_| {
            ([0, 0, 0, 0], port).into()
        }), Duration::from_secs(3)).is_ok()
    }
}

pub fn resolve_host(host: &str) -> anyhow::Result<Vec<String>> {
    DnsResolver::new().resolve_ipv4(host)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_resolve_localhost() {
        let ips = resolve_host("localhost").unwrap();
        assert!(ips.contains(&"127.0.0.1".to_string()));
    }

    #[test]
    fn test_invalid_host() {
        let result = resolve_host("invalid-host-name-xyz.local");
        assert!(result.is_err());
    }
}
