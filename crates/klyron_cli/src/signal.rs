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

    pub fn setup() -> Self {
        let mut handler = Self::new();
        handler.on(Signal::Sigint, Box::new(Self::graceful_shutdown));
        handler.on(Signal::Sigterm, Box::new(Self::graceful_shutdown));
        handler.on(Signal::Sighup, Box::new(Self::config_reload));
        handler.on(Signal::Sigusr1, Box::new(Self::toggle_debug));
        handler.on(Signal::Sigusr2, Box::new(Self::dump_state));

        let handler_clone = std::sync::Arc::new(std::sync::Mutex::new(handler));

        let hup_clone = handler_clone.clone();
        let term_clone = handler_clone.clone();
        let usr1_clone = handler_clone.clone();
        let usr2_clone = handler_clone.clone();

        ctrlc::set_handler(move || {
            let guard = handler_clone.lock().unwrap_or_else(|e| e.into_inner());
            if !guard.blocked_signals.load(std::sync::atomic::Ordering::Relaxed) {
                if let Some(handlers) = guard.handlers.get(&Signal::Sigint) {
                    for h in handlers {
                        h();
                    }
                }
            }
            std::process::exit(130);
        })
        .expect("Error setting Ctrl-C handler");

        std::thread::spawn(move || {
            let sigterm = Signal::Sigterm;
            let sighup = Signal::Sighup;
            let sigusr1 = Signal::Sigusr1;
            let sigusr2 = Signal::Sigusr2;
            loop {
                std::thread::sleep(std::time::Duration::from_millis(100));
                // In a real implementation, we'd use signal blocking via libc::sigwait
                // For now, these are triggered externally
                let _ = (&sigterm, &sighup, &sigusr1, &sigusr2);
            }
        });

        handler_clone.lock().unwrap_or_else(|e| e.into_inner())
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
        let _ = crate::ConfigManager::load_all();
        eprintln!("\u{2705} Configuration reloaded.");
    }

    pub fn toggle_debug() {
        static DEBUG_ENABLED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
        let enabled = DEBUG_ENABLED.fetch_xor(true, std::sync::atomic::Ordering::SeqCst);
        if !enabled {
            crate::logger::set_log_level(crate::logger::LogLevel::Debug);
            eprintln!("\u{1F50D} Debug logging enabled");
        } else {
            crate::logger::set_log_level(crate::logger::LogLevel::Info);
            eprintln!("\u{1F50D} Debug logging disabled");
        }
    }

    pub fn dump_state() {
        eprintln!("\u{1F4CA} State dump:");
        eprintln!("  PID: {}", std::process::id());
        eprintln!("  CWD: {}", std::env::current_dir().unwrap_or_default().display());
        eprintln!("  Args: {:?}", std::env::args().collect::<Vec<_>>());
        if let Ok(cache) = dirs::cache_dir() {
            eprintln!("  Cache: {}", cache.join("klyron").display());
        }
        eprintln!("  Version: {}", env!("CARGO_PKG_VERSION"));
    }
}

impl Default for SignalHandler {
    fn default() -> Self {
        Self::new()
    }
}
