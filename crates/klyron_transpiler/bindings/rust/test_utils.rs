#![cfg(test)]
use crate::client::TranspilerClient;

pub fn create_test_client() -> TranspilerClient {
    TranspilerClient::new()
}

#[test]
fn test_create_client() {
    let _client = create_test_client();
}
