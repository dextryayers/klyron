use crate::types;
use crate::errors::CryptoProviderError;

pub struct CryptoProviderClient;
impl CryptoProviderClient {
    pub fn new() -> Self { Self }
}
impl Default for CryptoProviderClient { fn default() -> Self { Self::new() } }
