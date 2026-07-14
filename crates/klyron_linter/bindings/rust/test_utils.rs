#![cfg(test)]
use crate::client::LinterClient;

pub fn create_test_client() -> LinterClient {
    LinterClient::new()
}

#[test]
fn test_create_client() {
    let _client = create_test_client();
}
