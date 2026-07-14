//! klyron_engine — Polyglot engine trait + bridges
//!
//! ## Crate bindings
//! This module exposes klyron_engine across Rust, TypeScript, C++, C, and Bash.

pub mod types;
pub mod errors;
pub mod builder;
pub mod config;
pub mod ffi;
pub mod wasm;
pub mod client;
pub mod server;
pub mod test_utils;

pub use types::{Klyron::EngineConfig, Klyron::EngineResult, Klyron::EngineStatus};
pub use errors::{Klyron::EngineError};
pub use builder::{Klyron::EngineBuilder, Klyron::EngineInstance};
pub use config::{Klyron::EngineSettings};
pub use client::{Klyron::EngineClient};
pub use server::{Klyron::EngineServer};
