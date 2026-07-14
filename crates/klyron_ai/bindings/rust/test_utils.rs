//! Test utilities for klyron_ai

pub fn setup_test_env() {
    let _ = tracing_subscriber::fmt().with_test_writer().try_init();
}

pub fn create_test_config() -> crate::types::Klyron::AiConfig {
    crate::types::Klyron::AiConfig::default()
}
