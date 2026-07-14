//! klyron_runtime — Core runtime
//!
//! ## Crate bindings
//! This module exposes klyron_runtime across Rust, TypeScript, C++, C, and Bash.

pub mod types;
pub mod errors;
pub mod builder;
pub mod config;
pub mod ffi;
pub mod wasm;
pub mod client;
pub mod server;
pub mod test_utils;

pub use types::{Klyron::RuntimeConfig, Klyron::RuntimeResult, Klyron::RuntimeStatus};
pub use errors::{Klyron::RuntimeError};
pub use builder::{Klyron::RuntimeBuilder, Klyron::RuntimeInstance};
pub use config::{Klyron::RuntimeSettings};
pub use client::{Klyron::RuntimeClient};
pub use server::{Klyron::RuntimeServer};
