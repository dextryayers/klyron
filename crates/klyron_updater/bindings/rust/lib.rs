//! klyron_updater — Self-update mechanism
//!
//! ## Crate bindings
//! This module exposes klyron_updater across Rust, TypeScript, C++, C, and Bash.

pub mod types;
pub mod errors;
pub mod builder;
pub mod config;
pub mod ffi;
pub mod wasm;
pub mod client;
pub mod server;
pub mod test_utils;

pub use types::{Klyron::UpdaterConfig, Klyron::UpdaterResult, Klyron::UpdaterStatus};
pub use errors::{Klyron::UpdaterError};
pub use builder::{Klyron::UpdaterBuilder, Klyron::UpdaterInstance};
pub use config::{Klyron::UpdaterSettings};
pub use client::{Klyron::UpdaterClient};
pub use server::{Klyron::UpdaterServer};
