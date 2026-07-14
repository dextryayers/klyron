use crate::types;
use crate::errors::HttpServerError;

pub struct HttpServerClient;
impl HttpServerClient {
    pub fn new() -> Self { Self }
}
impl Default for HttpServerClient { fn default() -> Self { Self::new() } }
