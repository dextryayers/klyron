mod client;
mod debugger;
mod protocol;

pub use client::DebugClient;
pub use debugger::{Breakpoint, Debugger};
pub use protocol::{CallFrame, CdpMessage, DebuggerDomain, Location, PausedEvent, Scope};
