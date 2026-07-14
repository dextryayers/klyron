#![cfg(test)]
use crate::client::RegistryClient;

pub fn create_test_client() -> RegistryClient {
    RegistryClient::new()
}

#[test]
fn test_create_client() {
    let _client = create_test_client();
}
