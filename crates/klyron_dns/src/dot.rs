use std::str::FromStr;
use std::time::Duration;

use hickory_resolver::config::{ResolverConfig, ResolverOpts};
use hickory_resolver::proto::rr::record_type::RecordType;
use hickory_resolver::proto::rr::{Name, RData};
use hickory_resolver::Resolver;

use crate::{DnsRecord, DnsResult};
use crate::cache::DnsCache;

fn lookup_to_records(lookup: &hickory_resolver::lookup::Lookup, rtype: &str) -> Vec<DnsRecord> {
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

fn doh_resolver() -> anyhow::Result<Resolver> {
    let resolver = Resolver::new(ResolverConfig::cloudflare_https(), ResolverOpts::default())?;
    Ok(resolver)
}

fn dot_resolver() -> anyhow::Result<Resolver> {
    let resolver = Resolver::new(ResolverConfig::cloudflare_tls(), ResolverOpts::default())?;
    Ok(resolver)
}

pub struct DnsOverHttps {
    cache: DnsCache,
    timeout: Duration,
}

impl DnsOverHttps {
    pub fn new() -> Self {
        Self {
            cache: DnsCache::new(Duration::from_secs(120)),
            timeout: Duration::from_secs(10),
        }
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    pub fn resolve(&self, host: &str) -> anyhow::Result<DnsResult> {
        let cache_key = format!("doh:{}", host);
        if let Some(cached) = self.cache.get(&cache_key) {
            return Ok(cached);
        }

        let resolver = doh_resolver()?;
        let name = Name::from_str(host)?;
        let lookup = resolver.lookup(name, RecordType::A)?;

        let records = lookup_to_records(&lookup, "A");
        let addresses = lookup
            .iter()
            .filter_map(|rdata| {
                if let RData::A(ip) = rdata {
                    Some(std::net::IpAddr::V4(ip.0))
                } else {
                    None
                }
            })
            .collect();

        let result = DnsResult {
            hostname: host.to_string(),
            addresses,
            records,
            cname: None,
        };

        self.cache.set(cache_key, result.clone());
        Ok(result)
    }

    pub async fn resolve_async(&self, host: &str) -> anyhow::Result<DnsResult> {
        let host_str = host.to_string();
        let display_host = host_str.clone();
        let result = tokio::time::timeout(
            self.timeout,
            tokio::task::spawn_blocking(move || {
                let doh = DnsOverHttps::new();
                doh.resolve(&host_str)
            }),
        )
        .await
        .map_err(|_| anyhow::anyhow!("DoH resolution timed out for: {display_host}"))?
        .map_err(|e| anyhow::anyhow!("Task join failed: {e}"))??;

        Ok(result)
    }
}

impl Default for DnsOverHttps {
    fn default() -> Self {
        Self::new()
    }
}

pub struct DnsOverTls {
    cache: DnsCache,
    timeout: Duration,
}

impl DnsOverTls {
    pub fn new() -> Self {
        Self {
            cache: DnsCache::new(Duration::from_secs(120)),
            timeout: Duration::from_secs(10),
        }
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    pub fn resolve(&self, host: &str) -> anyhow::Result<DnsResult> {
        let cache_key = format!("dot:{}", host);
        if let Some(cached) = self.cache.get(&cache_key) {
            return Ok(cached);
        }

        let resolver = dot_resolver()?;
        let name = Name::from_str(host)?;
        let lookup = resolver.lookup(name, RecordType::A)?;

        let records = lookup_to_records(&lookup, "A");
        let addresses = lookup
            .iter()
            .filter_map(|rdata| {
                if let RData::A(ip) = rdata {
                    Some(std::net::IpAddr::V4(ip.0))
                } else {
                    None
                }
            })
            .collect();

        let result = DnsResult {
            hostname: host.to_string(),
            addresses,
            records,
            cname: None,
        };

        self.cache.set(cache_key, result.clone());
        Ok(result)
    }

    pub async fn resolve_async(&self, host: &str) -> anyhow::Result<DnsResult> {
        let host_str = host.to_string();
        let display_host = host_str.clone();
        let result = tokio::time::timeout(
            self.timeout,
            tokio::task::spawn_blocking(move || {
                let dot = DnsOverTls::new();
                dot.resolve(&host_str)
            }),
        )
        .await
        .map_err(|_| anyhow::anyhow!("DoT resolution timed out for: {display_host}"))?
        .map_err(|e| anyhow::anyhow!("Task join failed: {e}"))??;

        Ok(result)
    }
}

impl Default for DnsOverTls {
    fn default() -> Self {
        Self::new()
    }
}
