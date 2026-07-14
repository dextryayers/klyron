//! Test utilities for klyron_cli

pub fn setup_test_env() {
    let _ = tracing_subscriber::fmt().with_test_writer().try_init();
}

pub fn create_test_config() -> crate::types::Klyron::CliConfig {
    crate::types::Klyron::CliConfig::default()
}
