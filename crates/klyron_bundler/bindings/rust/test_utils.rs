#![cfg(test)]
use crate::client::BundlerClient;

pub fn create_test_client() -> BundlerClient {
    BundlerClient::new()
}

#[test]
fn test_create_client() {
    let _client = create_test_client();
}
