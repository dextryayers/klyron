//! klyron_ai — AI features
//!
//! ## Crate bindings
//! This module exposes klyron_ai across Rust, TypeScript, C++, C, and Bash.

pub mod types;
pub mod errors;
pub mod builder;
pub mod config;
pub mod ffi;
pub mod wasm;
pub mod client;
pub mod server;
pub mod test_utils;

pub use types::{Klyron::AiConfig, Klyron::AiResult, Klyron::AiStatus};
pub use errors::{Klyron::AiError};
pub use builder::{Klyron::AiBuilder, Klyron::AiInstance};
pub use config::{Klyron::AiSettings};
pub use client::{Klyron::AiClient};
pub use server::{Klyron::AiServer};
