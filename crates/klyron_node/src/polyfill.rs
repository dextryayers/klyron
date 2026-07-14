use std::collections::HashMap;

pub struct ModuleMap {
    modules: HashMap<String, ModuleImpl>,
}

pub struct ModuleImpl {
    pub name: String,
    pub exports: Vec<&'static str>,
}

impl ModuleMap {
    pub fn new() -> Self {
        let mut modules = HashMap::new();

        modules.insert("fs".to_string(), ModuleImpl {
            name: "fs".to_string(),
            exports: vec![
                "readFileSync", "writeFileSync", "readdirSync", "mkdirSync",
                "statSync", "existsSync", "unlinkSync", "rmdirSync",
                "renameSync", "copyFileSync", "chmodSync", "accessSync",
                "readFile", "writeFile", "readdir", "mkdir",
                "stat", "unlink", "rmdir", "rename", "copyFile",
                "createReadStream", "createWriteStream", "watch",
                "exists", "promises",
            ],
        });

        modules.insert("path".to_string(), ModuleImpl {
            name: "path".to_string(),
            exports: vec![
                "join", "resolve", "dirname", "basename", "extname",
                "relative", "normalize", "parse", "format",
                "sep", "delimiter", "posix", "win32",
            ],
        });

        modules.insert("http".to_string(), ModuleImpl {
            name: "http".to_string(),
            exports: vec![
                "createServer", "request", "get", "Server",
                "ServerResponse", "IncomingMessage", "Agent",
                "METHODS", "STATUS_CODES",
            ],
        });

        modules.insert("https".to_string(), ModuleImpl {
            name: "https".to_string(),
            exports: vec![
                "createServer", "request", "get", "Agent",
            ],
        });

        modules.insert("os".to_string(), ModuleImpl {
            name: "os".to_string(),
            exports: vec![
                "platform", "arch", "cpus", "totalmem", "freemem",
                "homedir", "hostname", "networkInterfaces",
                "tmpdir", "EOL", "type", "release", "uptime",
                "loadavg", "userInfo",
            ],
        });

        modules.insert("crypto".to_string(), ModuleImpl {
            name: "crypto".to_string(),
            exports: vec![
                "createHash", "randomBytes", "createCipheriv",
                "createDecipheriv", "sign", "verify",
                "generateKeyPair", "randomUUID",
                "scrypt", "pbkdf2",
            ],
        });

        modules.insert("stream".to_string(), ModuleImpl {
            name: "stream".to_string(),
            exports: vec![
                "Readable", "Writable", "Transform", "Duplex",
                "pipeline", "finished", "PassThrough",
            ],
        });

        modules.insert("events".to_string(), ModuleImpl {
            name: "events".to_string(),
            exports: vec![
                "EventEmitter", "once", "on", "listenerCount",
                "getMaxListeners",
                "Event", "errorMonitor",
            ],
        });

        modules.insert("util".to_string(), ModuleImpl {
            name: "util".to_string(),
            exports: vec![
                "promisify", "callbackify", "inherits", "inspect",
                "types", "deprecate", "format",
            ],
        });

        modules.insert("child_process".to_string(), ModuleImpl {
            name: "child_process".to_string(),
            exports: vec![
                "spawn", "exec", "execFile", "fork",
                "ChildProcess", "spawnSync", "execSync",
            ],
        });

        modules.insert("assert".to_string(), ModuleImpl {
            name: "assert".to_string(),
            exports: vec![
                "strict", "deepEqual", "doesNotThrow",
                "rejects", "throws", "ok", "equal",
                "notEqual", "deepStrictEqual",
            ],
        });

        modules.insert("buffer".to_string(), ModuleImpl {
            name: "buffer".to_string(),
            exports: vec![
                "Buffer", "Blob", "SlowBuffer", "constants",
                "INSPECT_MAX_BYTES",
            ],
        });

        modules.insert("url".to_string(), ModuleImpl {
            name: "url".to_string(),
            exports: vec![
                "URL", "URLSearchParams", "fileURLToPath",
                "pathToFileURL", "format", "parse", "resolve",
            ],
        });

        modules.insert("timers".to_string(), ModuleImpl {
            name: "timers".to_string(),
            exports: vec![
                "setImmediate", "clearImmediate",
                "setTimeout", "clearTimeout",
                "setInterval", "clearInterval",
            ],
        });

        modules.insert("process".to_string(), ModuleImpl {
            name: "process".to_string(),
            exports: vec![
                "env", "argv", "cwd", "chdir", "exit",
                "nextTick", "stdout", "stderr", "stdin",
                "on", "emit", "version", "platform", "arch",
                "pid", "uptime", "memoryUsage",
            ],
        });

        modules.insert("console".to_string(), ModuleImpl {
            name: "console".to_string(),
            exports: vec![
                "log", "warn", "error", "info", "debug",
                "time", "timeLog", "timeEnd", "count",
                "countReset", "group", "groupEnd",
                "Console", "table", "dir", "trace",
            ],
        });

        modules.insert("module".to_string(), ModuleImpl {
            name: "module".to_string(),
            exports: vec![
                "require", "Module", "_resolveFilename",
                "_extensions", "_cache",
            ],
        });

        Self { modules }
    }

    pub fn get(&self, name: &str) -> Option<&ModuleImpl> {
        self.modules.get(name)
    }

    pub fn has(&self, name: &str) -> bool {
        self.modules.contains_key(name)
    }

    pub fn is_core_module(name: &str) -> bool {
        matches!(name, "fs" | "path" | "http" | "https" | "os" | "crypto"
            | "stream" | "events" | "util" | "child_process" | "assert"
            | "buffer" | "url" | "timers" | "process" | "console" | "module")
    }

    pub fn core_module_names() -> &'static [&'static str] {
        &["fs", "path", "http", "https", "os", "crypto", "stream",
          "events", "util", "child_process", "assert", "buffer",
          "url", "timers", "process", "console", "module"]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_core_modules() {
        assert!(ModuleMap::is_core_module("fs"));
        assert!(ModuleMap::is_core_module("path"));
        assert!(!ModuleMap::is_core_module("nonexistent"));
    }

    #[test]
    fn test_module_exports() {
        let map = ModuleMap::new();
        let fs = map.get("fs").unwrap();
        assert!(fs.exports.contains(&"readFileSync"));
    }
}
