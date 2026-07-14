//! Test utilities for klyron_utils

pub fn setup_test_env() {
    let _ = tracing_subscriber::fmt().with_test_writer().try_init();
}

pub fn create_test_config() -> crate::types::Klyron::UtilsConfig {
    crate::types::Klyron::UtilsConfig::default()
}
