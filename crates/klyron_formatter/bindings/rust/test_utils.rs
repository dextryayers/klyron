#![cfg(test)]
use crate::client::FormatterClient;

pub fn create_test_client() -> FormatterClient {
    FormatterClient::new()
}

#[test]
fn test_create_client() {
    let _client = create_test_client();
}
