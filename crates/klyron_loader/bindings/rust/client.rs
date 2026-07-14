use crate::types;
use crate::errors::ModuleResolverError;

pub struct ModuleResolverClient;
impl ModuleResolverClient {
    pub fn new() -> Self { Self }
}
impl Default for ModuleResolverClient { fn default() -> Self { Self::new() } }
