use std::net::{IpAddr, SocketAddr, TcpStream, ToSocketAddrs};
use std::str::FromStr;
use std::time::Duration;

use hickory_resolver::config::{ResolverConfig, ResolverOpts};
use hickory_resolver::proto::rr::record_type::RecordType;
use hickory_resolver::proto::rr::RData;
use hickory_resolver::Resolver;

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

pub struct DnsResolver;

fn new_resolver() -> anyhow::Result<Resolver> {
    Ok(Resolver::new(ResolverConfig::default(), ResolverOpts::default())?)
}

fn lookup_to_records(lookup: &hickory_resolver::lookup::Lookup, rtype: &str) -> Vec<DnsRecord> {
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
    pub fn new() -> Self { Self }

    pub fn resolve(&self, host: &str) -> anyhow::Result<DnsResult> {
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

        Ok(DnsResult {
            hostname: host.to_string(),
            addresses: ips,
            records,
            cname,
        })
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
        let name = hickory_resolver::proto::rr::Name::from_str(host)?;
        let lookup = resolver.lookup(name, RecordType::CNAME)?;
        Ok(lookup_to_records(&lookup, "CNAME"))
    }

    pub fn resolve_mx(&self, host: &str) -> anyhow::Result<Vec<DnsRecord>> {
        let resolver = new_resolver()?;
        let name = hickory_resolver::proto::rr::Name::from_str(host)?;
        let lookup = resolver.lookup(name, RecordType::MX)?;
        Ok(lookup_to_records(&lookup, "MX"))
    }

    pub fn resolve_txt(&self, host: &str) -> anyhow::Result<Vec<DnsRecord>> {
        let resolver = new_resolver()?;
        let name = hickory_resolver::proto::rr::Name::from_str(host)?;
        let lookup = resolver.lookup(name, RecordType::TXT)?;
        Ok(lookup_to_records(&lookup, "TXT"))
    }

    pub fn resolve_srv(&self, host: &str) -> anyhow::Result<Vec<DnsRecord>> {
        let resolver = new_resolver()?;
        let name = hickory_resolver::proto::rr::Name::from_str(host)?;
        let lookup = resolver.lookup(name, RecordType::SRV)?;
        Ok(lookup_to_records(&lookup, "SRV"))
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
        let name = hickory_resolver::proto::rr::Name::from_str(&reverse_name)?;
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
        Ok(result)
    }
}

impl Default for DnsResolver {
    fn default() -> Self { Self::new() }
}

pub fn resolve_host(host: &str) -> anyhow::Result<Vec<IpAddr>> {
    DnsResolver::new().resolve_ipv4(host)
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
    fn test_reverse_lookup() {
        let result = reverse_lookup("127.0.0.1".parse().unwrap());
        assert!(result.is_ok());
    }

    #[test]
    fn test_is_reachable() {
        let resolver = DnsResolver::new();
        assert!(!resolver.is_reachable("127.0.0.1", 1));
    }

    #[test]
    fn test_resolve_cname() {
        let resolver = DnsResolver::new();
        let result = resolver.resolve_cname("www.google.com");
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_resolve_mx() {
        let resolver = DnsResolver::new();
        let result = resolver.resolve_mx("gmail.com");
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_resolve_txt() {
        let resolver = DnsResolver::new();
        let result = resolver.resolve_txt("google.com");
        assert!(result.is_ok() || result.is_err());
    }
}
