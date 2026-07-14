use crate::types;
use crate::errors::DnsResolverError;

pub struct DnsResolverClient;
impl DnsResolverClient {
    pub fn new() -> Self { Self }
}
impl Default for DnsResolverClient { fn default() -> Self { Self::new() } }
