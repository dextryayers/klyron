#![cfg(test)]
use crate::client::WatcherClient;

pub fn create_test_client() -> WatcherClient {
    WatcherClient::new()
}

#[test]
fn test_create_client() {
    let _client = create_test_client();
}
