use std::net::IpAddr;

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
impl DnsResolver {
    pub fn new() -> Self { Self }
    pub fn resolve(&self, host: &str) -> anyhow::Result<DnsResult> { anyhow::bail!("resolve not available in bindings") }
    pub fn resolve_ipv4(&self, host: &str) -> anyhow::Result<Vec<IpAddr>> { anyhow::bail!("resolve_ipv4 not available in bindings") }
    pub fn resolve_ipv6(&self, host: &str) -> anyhow::Result<Vec<IpAddr>> { anyhow::bail!("resolve_ipv6 not available in bindings") }
    pub fn resolve_cname(&self, host: &str) -> anyhow::Result<Vec<DnsRecord>> { anyhow::bail!("resolve_cname not available in bindings") }
    pub fn resolve_mx(&self, host: &str) -> anyhow::Result<Vec<DnsRecord>> { anyhow::bail!("resolve_mx not available in bindings") }
    pub fn resolve_txt(&self, host: &str) -> anyhow::Result<Vec<DnsRecord>> { anyhow::bail!("resolve_txt not available in bindings") }
    pub fn resolve_srv(&self, host: &str) -> anyhow::Result<Vec<DnsRecord>> { anyhow::bail!("resolve_srv not available in bindings") }
    pub fn reverse_lookup(&self, addr: IpAddr) -> anyhow::Result<String> { anyhow::bail!("reverse_lookup not available in bindings") }
    pub fn is_reachable(&self, host: &str, port: u16) -> bool { false }
    pub fn resolve_all(&self, host: &str) -> anyhow::Result<DnsResult> { self.resolve(host) }
}
impl Default for DnsResolver { fn default() -> Self { Self::new() } }
