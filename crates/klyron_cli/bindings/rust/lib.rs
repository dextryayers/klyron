//! klyron_cli — CLI entry point
//!
//! ## Crate bindings
//! This module exposes klyron_cli across Rust, TypeScript, C++, C, and Bash.

pub mod types;
pub mod errors;
pub mod builder;
pub mod config;
pub mod ffi;
pub mod wasm;
pub mod client;
pub mod server;
pub mod test_utils;

pub use types::{Klyron::CliConfig, Klyron::CliResult, Klyron::CliStatus};
pub use errors::{Klyron::CliError};
pub use builder::{Klyron::CliBuilder, Klyron::CliInstance};
pub use config::{Klyron::CliSettings};
pub use client::{Klyron::CliClient};
pub use server::{Klyron::CliServer};
