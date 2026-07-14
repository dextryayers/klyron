#![cfg(test)]
use crate::client::BenchClient;

pub fn create_test_client() -> BenchClient {
    BenchClient::new()
}

#[test]
fn test_create_client() {
    let _client = create_test_client();
}
