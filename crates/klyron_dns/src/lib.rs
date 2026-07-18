pub mod cache;
pub mod dot;
pub mod resolver;

use std::net::IpAddr;

pub use dot::{DnsOverHttps, DnsOverTls};
pub use resolver::DnsResolver;

#[derive(Debug, Clone)]
pub struct DnsRecord {
    pub name: String,
    pub record_type: String,
    pub value: String,
    pub ttl: u32,
}

#[derive(Debug, Clone)]
pub struct DnsResult {
    pub hostname: String,
    pub addresses: Vec<IpAddr>,
    pub records: Vec<DnsRecord>,
    pub cname: Option<String>,
}

pub fn resolve_host(host: &str) -> anyhow::Result<Vec<IpAddr>> {
    DnsResolver::new().resolve_ipv4(host)
}

pub fn resolve_host_all(host: &str) -> anyhow::Result<Vec<IpAddr>> {
    let result = DnsResolver::new().resolve(host)?;
    Ok(result.addresses)
}

pub fn reverse_lookup(addr: IpAddr) -> anyhow::Result<String> {
    DnsResolver::new().reverse_lookup(addr)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_localhost() {
        let ips = resolve_host("localhost").unwrap();
        assert!(ips.contains(&"127.0.0.1".parse().unwrap()));
    }

    #[test]
    fn test_invalid_host() {
        let result = resolve_host("invalid-host-name-xyz.local");
        assert!(result.is_err());
    }

    #[test]
    fn test_dns_cache() {
        let resolver = DnsResolver::new();
        let r1 = resolver.resolve("localhost").unwrap();
        let r2 = resolver.resolve("localhost").unwrap();
        assert_eq!(r1.hostname, r2.hostname);
    }

    #[test]
    fn test_is_reachable() {
        let resolver = DnsResolver::new();
        assert!(!resolver.is_reachable("127.0.0.1", 1));
    }

    #[test]
    fn test_srv_record() {
        let resolver = DnsResolver::new();
        if let Ok(records) = resolver.resolve_srv("_xmpp._tcp.gmail.com") {
            assert!(!records.is_empty());
        }
    }

    #[test]
    fn test_doh() {
        let doh = DnsOverHttps::new();
        if let Ok(result) = doh.resolve("google.com") {
            assert!(!result.addresses.is_empty());
        }
    }

    #[test]
    fn test_cache_prune() {
        let cache = DnsCache::default();
        cache.set(
            "test.example".into(),
            DnsResult {
                hostname: "test.example".into(),
                addresses: vec![],
                records: vec![],
                cname: None,
            },
        );
        assert_eq!(cache.len(), 1);
        cache.prune_expired();
        assert_eq!(cache.len(), 1);
        cache.clear();
        assert!(cache.is_empty());
    }
}
