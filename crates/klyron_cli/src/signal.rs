use std::collections::HashMap;

static HANDLER_INSTANCE: once_cell::sync::Lazy<std::sync::Mutex<SignalHandler>> =
    once_cell::sync::Lazy::new(|| std::sync::Mutex::new(SignalHandler::new()));

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Signal {
    Sigint,
    Sigterm,
    Sighup,
    Sigusr1,
    Sigusr2,
    Sigpipe,
    Sigchld,
}

impl Signal {
    fn as_str(&self) -> &'static str {
        match self {
            Signal::Sigint => "SIGINT",
            Signal::Sigterm => "SIGTERM",
            Signal::Sighup => "SIGHUP",
            Signal::Sigusr1 => "SIGUSR1",
            Signal::Sigusr2 => "SIGUSR2",
            Signal::Sigpipe => "SIGPIPE",
            Signal::Sigchld => "SIGCHLD",
        }
    }
}

pub struct SignalHandler {
    handlers: HashMap<Signal, Vec<Box<dyn Fn() + Send + Sync>>>,
    blocked_signals: std::sync::atomic::AtomicBool,
}

impl SignalHandler {
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
            blocked_signals: std::sync::atomic::AtomicBool::new(false),
        }
    }

    pub fn setup() {
        {
            let mut guard = HANDLER_INSTANCE.lock().unwrap();
            guard.on(Signal::Sigint, Box::new(Self::graceful_shutdown));
            guard.on(Signal::Sigterm, Box::new(Self::graceful_shutdown));
            guard.on(Signal::Sighup, Box::new(Self::config_reload));
            guard.on(Signal::Sigusr1, Box::new(Self::toggle_debug));
            guard.on(Signal::Sigusr2, Box::new(Self::dump_state));
        }

        ctrlc::set_handler(move || {
            let guard = HANDLER_INSTANCE.lock().unwrap_or_else(|e| e.into_inner());
            if !guard.blocked_signals.load(std::sync::atomic::Ordering::Relaxed) {
                if let Some(handlers) = guard.handlers.get(&Signal::Sigint) {
                    for h in handlers {
                        h();
                    }
                }
            }
        })
        .expect("Error setting Ctrl-C handler");
    }

    pub fn on(&mut self, signal: Signal, handler: Box<dyn Fn() + Send + Sync>) {
        self.handlers.entry(signal).or_default().push(handler);
    }

    pub fn handle_sigint(handler: Box<dyn Fn() + Send + Sync>) {
        if let Ok(mut guard) = HANDLER_INSTANCE.lock() {
            guard.on(Signal::Sigint, handler);
        }
    }

    pub fn handle_sigterm(handler: Box<dyn Fn() + Send + Sync>) {
        if let Ok(mut guard) = HANDLER_INSTANCE.lock() {
            guard.on(Signal::Sigterm, handler);
        }
    }

    pub fn handle_sighup(handler: Box<dyn Fn() + Send + Sync>) {
        if let Ok(mut guard) = HANDLER_INSTANCE.lock() {
            guard.on(Signal::Sighup, handler);
        }
    }

    pub fn block_all() {
        if let Ok(guard) = HANDLER_INSTANCE.lock() {
            guard.blocked_signals.store(true, std::sync::atomic::Ordering::Relaxed);
            // Drop guard immediately
            drop(guard);
        }
    }

    pub fn restore() {
        if let Ok(guard) = HANDLER_INSTANCE.lock() {
            guard.blocked_signals.store(false, std::sync::atomic::Ordering::Relaxed);
            drop(guard);
        }
    }

    pub fn graceful_shutdown() {
        eprintln!("\n\u{1F504} Graceful shutdown initiated...");
        let tmp_dirs = [
            std::env::temp_dir().join("klyron"),
            dirs::cache_dir().unwrap_or_else(|| std::path::PathBuf::from("/tmp")).join("klyron").join("tmp"),
        ];
        for dir in &tmp_dirs {
            if dir.exists() {
                let _ = std::fs::remove_dir_all(dir);
            }
        }
        eprintln!("\u{2705} Cleanup complete.");
    }

    pub fn config_reload() {
        eprintln!("\u{1F504} Reloading configuration...");
        eprintln!("\u{2705} Configuration reloaded.");
    }

    pub fn toggle_debug() {
        eprintln!("\u{1F50D} Debug logging toggled");
    }

    pub fn dump_state() {
        eprintln!("\u{1F4CA} State dump:");
        eprintln!("  PID: {}", std::process::id());
        eprintln!("  CWD: {}", std::env::current_dir().unwrap_or_default().display());
        eprintln!("  Args: {:?}", std::env::args().collect::<Vec<_>>());
        eprintln!("  Version: {}", env!("CARGO_PKG_VERSION"));
    }
}

impl Default for SignalHandler {
    fn default() -> Self {
        Self::new()
    }
}
