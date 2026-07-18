use std::net::{IpAddr, SocketAddr, TcpStream, ToSocketAddrs};
use std::str::FromStr;
use std::time::Duration;

use hickory_resolver::config::{ResolverConfig, ResolverOpts};
use hickory_resolver::lookup::Lookup;
use hickory_resolver::proto::rr::record_type::RecordType;
use hickory_resolver::proto::rr::{Name, RData};
use hickory_resolver::Resolver;
use mdns_sd::{ServiceDaemon, ServiceEvent};

use crate::cache::DnsCache;
use crate::{DnsRecord, DnsResult};

fn new_resolver() -> anyhow::Result<Resolver> {
    Ok(Resolver::new(ResolverConfig::default(), ResolverOpts::default())?)
}

fn lookup_to_records(lookup: &Lookup, rtype: &str) -> Vec<DnsRecord> {
    let name = lookup.query().name().to_string();
    lookup
        .iter()
        .map(|rdata| {
            let value = match rdata {
                RData::A(ip) => ip.to_string(),
                RData::AAAA(ip) => ip.to_string(),
                RData::CNAME(cname) => cname.to_string(),
                RData::MX(mx) => format!("{} pref={}", mx.exchange(), mx.preference()),
                RData::SRV(srv) => format!(
                    "{}:{} prio={} weight={}",
                    srv.target(),
                    srv.port(),
                    srv.priority(),
                    srv.weight()
                ),
                RData::TXT(txt) => txt
                    .iter()
                    .map(|b| String::from_utf8_lossy(b).to_string())
                    .collect::<Vec<_>>()
                    .join(""),
                RData::PTR(ptr) => ptr.to_string(),
                _ => format!("{rdata:?}"),
            };
            DnsRecord {
                name: name.clone(),
                record_type: rtype.to_string(),
                value,
                ttl: 0,
            }
        })
        .collect()
}

pub struct DnsResolver {
    cache: DnsCache,
}

impl DnsResolver {
    pub fn new() -> Self {
        Self {
            cache: DnsCache::new(Duration::from_secs(60)),
        }
    }

    pub fn with_cache_ttl(mut self, ttl: Duration) -> Self {
        self.cache = DnsCache::new(ttl);
        self
    }

    pub fn cache(&self) -> &DnsCache {
        &self.cache
    }

    pub fn resolve(&self, host: &str) -> anyhow::Result<DnsResult> {
        if let Some(cached) = self.cache.get(host) {
            return Ok(cached);
        }

        let addr_str = format!("{host}:0");
        let addrs: Vec<SocketAddr> = addr_str.to_socket_addrs()?.collect();
        let ips: Vec<IpAddr> = addrs.iter().map(|a| a.ip()).collect();

        if ips.is_empty() {
            anyhow::bail!("Could not resolve hostname: {host}");
        }

        let records: Vec<DnsRecord> = ips
            .iter()
            .map(|ip| DnsRecord {
                name: host.to_string(),
                record_type: if ip.is_ipv4() { "A" } else { "AAAA" }.to_string(),
                value: ip.to_string(),
                ttl: 0,
            })
            .collect();

        let cname = self.resolve_cname(host).ok().and_then(|r| r.first().map(|r| r.value.clone()));

        let result = DnsResult {
            hostname: host.to_string(),
            addresses: ips,
            records,
            cname,
        };

        self.cache.set(host.to_string(), result.clone());
        Ok(result)
    }

    pub fn resolve_ipv4(&self, host: &str) -> anyhow::Result<Vec<IpAddr>> {
        let addr_str = format!("{host}:0");
        let ips: Vec<IpAddr> = addr_str
            .to_socket_addrs()?
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
        let ips: Vec<IpAddr> = addr_str
            .to_socket_addrs()?
            .filter(|a| a.is_ipv6())
            .map(|a| a.ip())
            .collect();
        if ips.is_empty() {
            anyhow::bail!("No IPv6 address found for: {host}");
        }
        Ok(ips)
    }

    pub fn resolve_cname(&self, host: &str) -> anyhow::Result<Vec<DnsRecord>> {
        let cache_key = format!("cname:{}", host);
        if let Some(records) = self.cache.get(&cache_key) {
            return Ok(records.records);
        }

        let resolver = new_resolver()?;
        let name = Name::from_str(host)?;
        let lookup = resolver.lookup(name, RecordType::CNAME)?;
        let records = lookup_to_records(&lookup, "CNAME");

        let result = DnsResult {
            hostname: host.to_string(),
            addresses: vec![],
            records: records.clone(),
            cname: None,
        };
        self.cache.set(cache_key, result);

        Ok(records)
    }

    pub fn resolve_mx(&self, host: &str) -> anyhow::Result<Vec<DnsRecord>> {
        let cache_key = format!("mx:{}", host);
        if let Some(records) = self.cache.get(&cache_key) {
            return Ok(records.records);
        }

        let resolver = new_resolver()?;
        let name = Name::from_str(host)?;
        let lookup = resolver.lookup(name, RecordType::MX)?;
        let records = lookup_to_records(&lookup, "MX");

        let result = DnsResult {
            hostname: host.to_string(),
            addresses: vec![],
            records: records.clone(),
            cname: None,
        };
        self.cache.set(cache_key, result);

        Ok(records)
    }

    pub fn resolve_txt(&self, host: &str) -> anyhow::Result<Vec<DnsRecord>> {
        let cache_key = format!("txt:{}", host);
        if let Some(records) = self.cache.get(&cache_key) {
            return Ok(records.records);
        }

        let resolver = new_resolver()?;
        let name = Name::from_str(host)?;
        let lookup = resolver.lookup(name, RecordType::TXT)?;
        let records = lookup_to_records(&lookup, "TXT");

        let result = DnsResult {
            hostname: host.to_string(),
            addresses: vec![],
            records: records.clone(),
            cname: None,
        };
        self.cache.set(cache_key, result);

        Ok(records)
    }

    pub fn resolve_srv(&self, host: &str) -> anyhow::Result<Vec<DnsRecord>> {
        let cache_key = format!("srv:{}", host);
        if let Some(records) = self.cache.get(&cache_key) {
            return Ok(records.records);
        }

        let resolver = new_resolver()?;
        let name = Name::from_str(host)?;
        let lookup = resolver.lookup(name, RecordType::SRV)?;
        let records = lookup_to_records(&lookup, "SRV");

        let result = DnsResult {
            hostname: host.to_string(),
            addresses: vec![],
            records: records.clone(),
            cname: None,
        };
        self.cache.set(cache_key, result);

        Ok(records)
    }

    pub fn resolve_ns(&self, host: &str) -> anyhow::Result<Vec<DnsRecord>> {
        let resolver = new_resolver()?;
        let name = Name::from_str(host)?;
        let lookup = resolver.lookup(name, RecordType::NS)?;
        Ok(lookup_to_records(&lookup, "NS"))
    }

    pub fn resolve_soa(&self, host: &str) -> anyhow::Result<Vec<DnsRecord>> {
        let resolver = new_resolver()?;
        let name = Name::from_str(host)?;
        let lookup = resolver.lookup(name, RecordType::SOA)?;
        Ok(lookup_to_records(&lookup, "SOA"))
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
        if let Ok(srv) = self.resolve_srv(host) {
            result.records.extend(srv);
        }
        self.cache.set(host.to_string(), result.clone());
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
        TcpStream::connect_timeout(&addr.parse().unwrap_or_else(|_| ([0, 0, 0, 0], port).into()), Duration::from_secs(3)).is_ok()
    }

    pub async fn resolve_async(&self, host: &str) -> anyhow::Result<DnsResult> {
        if let Some(cached) = self.cache.get(host) {
            return Ok(cached);
        }
        let host_str = host.to_string();
        let display_host = host_str.clone();
        let result: DnsResult = tokio::time::timeout(
            Duration::from_secs(5),
            tokio::task::spawn_blocking(move || DnsResolver::new().resolve(&host_str)),
        )
        .await
        .map_err(|_| anyhow::anyhow!("DNS resolution timed out for: {display_host}"))?
        .map_err(|e| anyhow::anyhow!("Task join failed: {e}"))??;

        self.cache.set(host.to_string(), result.clone());
        Ok(result)
    }
}

impl Default for DnsResolver {
    fn default() -> Self {
        Self::new()
    }
}
