mod client;
mod debugger;
pub mod inspector;
pub mod protocol;
pub mod stepper;

pub use client::DebugClient;
pub use debugger::{Breakpoint, Debugger};
pub use protocol::{CallFrame, CdpMessage, DebuggerDomain, Location, PausedEvent, Scope};
