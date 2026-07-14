use crate::types;
use crate::errors::WebApiError;

pub struct WebApiClient;
impl WebApiClient {
    pub fn new() -> Self { Self }
}
impl Default for WebApiClient { fn default() -> Self { Self::new() } }
