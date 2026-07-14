//! Test utilities for klyron_postgres

pub fn setup_test_env() {
    let _ = tracing_subscriber::fmt().with_test_writer().try_init();
}

pub fn create_test_config() -> crate::types::Klyron::PostgresConfig {
    crate::types::Klyron::PostgresConfig::default()
}
