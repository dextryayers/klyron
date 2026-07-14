use crate::types;
use crate::errors::NodeGlobalsError;

pub struct NodeGlobalsClient;
impl NodeGlobalsClient {
    pub fn new() -> Self { Self }
}
impl Default for NodeGlobalsClient { fn default() -> Self { Self::new() } }
