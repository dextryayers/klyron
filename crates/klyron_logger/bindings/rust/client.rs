use crate::types;
use crate::errors::LoggerError;

pub struct LoggerClient;
impl LoggerClient {
    pub fn new() -> Self { Self }
}
impl Default for LoggerClient { fn default() -> Self { Self::new() } }
