//! Klyron polyglot runtime SDK – Rust support modules.
//!
//! Each submodule provides std-only utilities commonly needed
//! by generated Klyron projects.

pub mod types;
pub mod process;
pub mod http_client;
pub mod fs_util;
pub mod crypto_util;
pub mod logger;
pub mod json_util;

// Re-export the most common types at the crate root for convenience.
pub use types::{JsonValue, KlyronError, Result};
pub use json_util::{parse, stringify};
pub use logger::{info, warn, error, debug};
pub use fs_util::{read_file, write_file, ensure_dir};
pub use process::{run_command, capture_output};
pub use crypto_util::{hash, uuid};
