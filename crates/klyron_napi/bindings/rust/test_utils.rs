#![cfg(test)]
use crate::NapiLoader;

pub fn create_test_loader() -> NapiLoader {
    NapiLoader::new()
}

pub fn create_mock_module(name: &str) -> crate::NapiModule {
    use std::collections::HashMap;
    crate::NapiModule {
        name: name.to_string(),
        exports: HashMap::new(),
    }
}

#[test]
fn test_test_utils() {
    let mut loader = create_test_loader();
    assert!(loader.list_loaded().is_empty());
    let _module = create_mock_module("test");
}
