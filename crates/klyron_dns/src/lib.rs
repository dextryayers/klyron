use std::collections::HashMap;
use std::net::{IpAddr, SocketAddr, TcpStream, ToSocketAddrs};
use std::str::FromStr;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use hickory_resolver::config::{ResolverConfig, ResolverOpts};
use hickory_resolver::lookup::Lookup;
use hickory_resolver::proto::rr::record_type::RecordType;
use hickory_resolver::proto::rr::{Name, RData};
use hickory_resolver::Resolver;
use mdns_sd::{ServiceDaemon, ServiceEvent};

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

struct CachedResult {
    result: DnsResult,
    expires: Instant,
}

pub struct DnsResolver {
    cache: Mutex<HashMap<String, CachedResult>>,
    cache_ttl: Duration,
}

fn new_resolver() -> anyhow::Result<Resolver> {
    Ok(Resolver::new(ResolverConfig::default(), ResolverOpts::default())?)
}

fn lookup_to_records(lookup: &Lookup, rtype: &str) -> Vec<DnsRecord> {
    let name = lookup.query().name().to_string();
    lookup.iter().map(|rdata| {
        let value = match rdata {
            RData::A(ip) => ip.to_string(),
            RData::AAAA(ip) => ip.to_string(),
            RData::CNAME(cname) => cname.to_string(),
            RData::MX(mx) => format!("{} pref={}", mx.exchange(), mx.preference()),
            RData::SRV(srv) => format!("{}:{} prio={} weight={}", srv.target(), srv.port(), srv.priority(), srv.weight()),
            RData::TXT(txt) => txt.iter().map(|b| String::from_utf8_lossy(b).to_string()).collect::<Vec<_>>().join(""),
            RData::PTR(ptr) => ptr.to_string(),
            _ => format!("{rdata:?}"),
        };
        DnsRecord {
            name: name.clone(),
            record_type: rtype.to_string(),
            value,
            ttl: 0,
        }
    }).collect()
}

impl DnsResolver {
    pub fn new() -> Self {
        Self {
            cache: Mutex::new(HashMap::new()),
            cache_ttl: Duration::from_secs(60),
        }
    }

    pub fn with_cache_ttl(mut self, ttl: Duration) -> Self {
        self.cache_ttl = ttl;
        self
    }

    fn check_cache(&self, host: &str) -> Option<DnsResult> {
        let cache = self.cache.lock().ok()?;
        if let Some(entry) = cache.get(host) {
            if Instant::now() < entry.expires {
                return Some(entry.result.clone());
            }
        }
        None
    }

    fn set_cache(&self, host: &str, result: DnsResult) {
        if let Ok(mut cache) = self.cache.lock() {
            cache.insert(host.to_string(), CachedResult {
                expires: Instant::now() + self.cache_ttl,
                result,
            });
        }
    }

    pub fn resolve(&self, host: &str) -> anyhow::Result<DnsResult> {
        if let Some(cached) = self.check_cache(host) {
            return Ok(cached);
        }

        let addr_str = format!("{host}:0");
        let addrs: Vec<SocketAddr> = addr_str.to_socket_addrs()?.collect();
        let ips: Vec<IpAddr> = addrs.iter().map(|a| a.ip()).collect();

        if ips.is_empty() {
            anyhow::bail!("Could not resolve hostname: {host}");
        }

        let records: Vec<DnsRecord> = ips.iter().map(|ip| DnsRecord {
            name: host.to_string(),
            record_type: if ip.is_ipv4() { "A".to_string() } else { "AAAA".to_string() },
            value: ip.to_string(),
            ttl: 0,
        }).collect();

        let cname = self.resolve_cname(host).ok()
            .and_then(|r| r.first().map(|r| r.value.clone()));

        let result = DnsResult {
            hostname: host.to_string(),
            addresses: ips,
            records,
            cname,
        };

        self.set_cache(host, result.clone());
        Ok(result)
    }

    pub fn resolve_ipv4(&self, host: &str) -> anyhow::Result<Vec<IpAddr>> {
        let addr_str = format!("{host}:0");
        let ips: Vec<IpAddr> = addr_str.to_socket_addrs()?
            .filter(|a| a.is_ipv4())
            .map(|a| a.ip())
            .collect();
        if ips.is_empty() {
            anyhow::bail!("No IPv4 address found for: {host}");
        }
        Ok(ips)
    }

    pub fn resolve_ipv6(&self, host: &str) -> anyhow::Result<Vec<IpAddr>> {
        let addr_str = format!("{host}:0");
        let ips: Vec<IpAddr> = addr_str.to_socket_addrs()?
            .filter(|a| a.is_ipv6())
            .map(|a| a.ip())
            .collect();
        if ips.is_empty() {
            anyhow::bail!("No IPv6 address found for: {host}");
        }
        Ok(ips)
    }

    pub fn resolve_cname(&self, host: &str) -> anyhow::Result<Vec<DnsRecord>> {
        let resolver = new_resolver()?;
        let name = Name::from_str(host)?;
        let lookup = resolver.lookup(name, RecordType::CNAME)?;
        Ok(lookup_to_records(&lookup, "CNAME"))
    }

    pub fn resolve_mx(&self, host: &str) -> anyhow::Result<Vec<DnsRecord>> {
        let resolver = new_resolver()?;
        let name = Name::from_str(host)?;
        let lookup = resolver.lookup(name, RecordType::MX)?;
        Ok(lookup_to_records(&lookup, "MX"))
    }

    pub fn resolve_txt(&self, host: &str) -> anyhow::Result<Vec<DnsRecord>> {
        let resolver = new_resolver()?;
        let name = Name::from_str(host)?;
        let lookup = resolver.lookup(name, RecordType::TXT)?;
        Ok(lookup_to_records(&lookup, "TXT"))
    }

    pub fn resolve_srv(&self, host: &str) -> anyhow::Result<Vec<DnsRecord>> {
        let resolver = new_resolver()?;
        let name = Name::from_str(host)?;
        let lookup = resolver.lookup(name, RecordType::SRV)?;
        Ok(lookup_to_records(&lookup, "SRV"))
    }

    pub fn resolve_all(&self, host: &str) -> anyhow::Result<DnsResult> {
        let mut result = self.resolve(host)?;
        if let Ok(cname) = self.resolve_cname(host) {
            if let Some(rec) = cname.first() {
                result.cname = Some(rec.value.clone());
                result.records.extend(cname);
            }
        }
        if let Ok(mx) = self.resolve_mx(host) {
            result.records.extend(mx);
        }
        if let Ok(txt) = self.resolve_txt(host) {
            result.records.extend(txt);
        }
        self.set_cache(host, result.clone());
        Ok(result)
    }

    pub fn resolve_mdns(&self, service_type: &str) -> anyhow::Result<Vec<DnsRecord>> {
        let daemon = ServiceDaemon::new()?;
        let receiver = daemon.browse(service_type)?;
        let mut records = Vec::new();
        loop {
            match receiver.recv_timeout(Duration::from_secs(2)) {
                Ok(ServiceEvent::ServiceResolved(info)) => {
                    for addr in info.get_addresses() {
                        records.push(DnsRecord {
                            name: info.get_fullname().to_string(),
                            record_type: "mDNS".to_string(),
                            value: addr.to_string(),
                            ttl: info.get_host_ttl(),
                        });
                    }
                    if !records.is_empty() {
                        break;
                    }
                }
                _ => break,
            }
        }
        if records.is_empty() {
            anyhow::bail!("No mDNS services found for: {service_type}");
        }
        Ok(records)
    }

    pub async fn resolve_doh(&self, host: &str) -> anyhow::Result<DnsResult> {
        let host_str = host.to_string();
        let host_clone = host_str.clone();
        let result: DnsResult = tokio::task::spawn_blocking(move || -> anyhow::Result<DnsResult> {
            let resolver = new_resolver()?;
            let name = Name::from_str(&host_str)?;
            let lookup = resolver.lookup(name, RecordType::A)?;
            let records = lookup_to_records(&lookup, "A");
            let addresses: Vec<IpAddr> = lookup.iter()
                .filter_map(|rdata| {
                    if let RData::A(ip) = rdata {
                        Some(IpAddr::V4(ip.0))
                    } else {
                        None
                    }
                })
                .collect();
            Ok(DnsResult {
                hostname: host_str.clone(),
                addresses,
                records,
                cname: None,
            })
        }).await??;
        self.set_cache(&host_clone, result.clone());
        Ok(result)
    }

    pub async fn resolve_async(&self, host: &str) -> anyhow::Result<DnsResult> {
        if let Some(cached) = self.check_cache(host) {
            return Ok(cached);
        }
        let host = host.to_string();
        let host_for_closure = host.clone();
        let result: DnsResult = tokio::time::timeout(
            Duration::from_secs(5),
            tokio::task::spawn_blocking(move || DnsResolver::new().resolve(&host_for_closure))
        ).await
            .map_err(|_| anyhow::anyhow!("DNS resolution timed out for: {host}"))?
            .map_err(|e| anyhow::anyhow!("Task join failed: {e}"))??;
        self.set_cache(&host, result.clone());
        Ok(result)
    }

    pub fn reverse_lookup(&self, addr: IpAddr) -> anyhow::Result<String> {
        let resolver = new_resolver()?;
        let reverse_name = match addr {
            IpAddr::V4(v4) => {
                let o = v4.octets();
                format!("{}.{}.{}.{}.in-addr.arpa", o[3], o[2], o[1], o[0])
            }
            IpAddr::V6(v6) => {
                let mut s = String::new();
                for b in v6.octets().iter().rev() {
                    s.push_str(&format!("{:x}.{:x}.", b & 0xf, b >> 4));
                }
                s.push_str("ip6.arpa");
                s
            }
        };
        let name = Name::from_str(&reverse_name)?;
        match resolver.lookup(name, RecordType::PTR) {
            Ok(lookup) => {
                for rdata in lookup.iter() {
                    if let RData::PTR(ptr) = rdata {
                        return Ok(ptr.to_string());
                    }
                }
                anyhow::bail!("No PTR record found for {addr}")
            }
            Err(e) => anyhow::bail!("Reverse lookup failed for {addr}: {e}"),
        }
    }

    pub fn is_reachable(&self, host: &str, port: u16) -> bool {
        let addr = format!("{host}:{port}");
        TcpStream::connect_timeout(
            &addr.parse().unwrap_or_else(|_| ([0, 0, 0, 0], port).into()),
            Duration::from_secs(3),
        ).is_ok()
    }
}

impl Default for DnsResolver {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

#[inline]
pub fn resolve_host(host: &str) -> anyhow::Result<Vec<IpAddr>> {
    DnsResolver::new().resolve_ipv4(host)
}

#[inline]
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
        let resolver = DnsResolver::new().with_cache_ttl(Duration::from_secs(60));
        let r1 = resolver.resolve("localhost").unwrap();
        let r2 = resolver.resolve("localhost").unwrap();
        assert_eq!(r1.hostname, r2.hostname);
    }

    #[test]
    fn test_is_reachable() {
        let resolver = DnsResolver::new();
        assert!(!resolver.is_reachable("127.0.0.1", 1));
    }
}
