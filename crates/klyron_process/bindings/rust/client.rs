use crate::types;
use crate::errors::ProcessManagerError;

pub struct ProcessManagerClient;
impl ProcessManagerClient {
    pub fn new() -> Self { Self }
}
impl Default for ProcessManagerClient { fn default() -> Self { Self::new() } }
