use crate::types;
use crate::errors::CacheManagerError;

pub struct CacheManagerClient;
impl CacheManagerClient {
    pub fn new() -> Self { Self }
}
impl Default for CacheManagerClient { fn default() -> Self { Self::new() } }
