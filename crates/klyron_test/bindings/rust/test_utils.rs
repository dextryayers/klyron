#![cfg(test)]
use crate::client::TestClient;

pub fn create_test_client() -> TestClient {
    TestClient::new()
}

#[test]
fn test_create_client() {
    let _client = create_test_client();
}
