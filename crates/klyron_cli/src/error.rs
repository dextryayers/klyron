use std::process::ExitCode;

#[derive(Debug)]
pub enum KlyronError {
    Io(std::io::Error),
    Network(String),
    Tls(String),
    Dns(String),
    Timeout(String),
    ConfigNotFound(String),
    ConfigParse(String),
    ConfigInvalid(String),
    ConfigMissing(String),
    EngineFailure(String),
    ScriptError { exit_code: i32, stderr: String },
    ModuleNotFound(String),
    SyntaxError(String),
    PackageNotFound(String),
    VersionNotFound(String),
    LockfileStale,
    LockfileCorrupt(String),
    IntegrityError(String),
    ResolutionError(String),
    DownloadError(String),
    RegistryError(String),
    PermissionDenied(String),
    NetworkBlocked(String),
    FileSystemBlocked(String),
    Internal(String),
    Unimplemented(String),
    Bug(String),
}

impl KlyronError {
    pub fn exit_code(&self) -> ExitCode {
        match self {
            KlyronError::Io(_) => ExitCode::from(1),
            KlyronError::Network(_) | KlyronError::Tls(_) | KlyronError::Dns(_) | KlyronError::Timeout(_) => ExitCode::from(2),
            KlyronError::ConfigNotFound(_) => ExitCode::from(10),
            KlyronError::ConfigParse(_) | KlyronError::ConfigInvalid(_) | KlyronError::ConfigMissing(_) => ExitCode::from(11),
            KlyronError::EngineFailure(_) => ExitCode::from(20),
            KlyronError::ScriptError { exit_code, .. } => ExitCode::from(*exit_code as u8),
            KlyronError::ModuleNotFound(_) => ExitCode::from(21),
            KlyronError::SyntaxError(_) => ExitCode::from(22),
            KlyronError::PackageNotFound(_) => ExitCode::from(30),
            KlyronError::VersionNotFound(_) => ExitCode::from(31),
            KlyronError::LockfileStale => ExitCode::from(32),
            KlyronError::LockfileCorrupt(_) => ExitCode::from(33),
            KlyronError::IntegrityError(_) => ExitCode::from(34),
            KlyronError::ResolutionError(_) => ExitCode::from(35),
            KlyronError::DownloadError(_) => ExitCode::from(36),
            KlyronError::RegistryError(_) => ExitCode::from(37),
            KlyronError::PermissionDenied(_) => ExitCode::from(40),
            KlyronError::NetworkBlocked(_) => ExitCode::from(41),
            KlyronError::FileSystemBlocked(_) => ExitCode::from(42),
            KlyronError::Internal(_) => ExitCode::from(70),
            KlyronError::Unimplemented(_) => ExitCode::from(80),
            KlyronError::Bug(_) => ExitCode::from(99),
        }
    }

    pub fn context(&self) -> String {
        match self {
            KlyronError::Io(e) => format!("I/O operation failed: {e}"),
            KlyronError::Network(e) => format!("Network error: {e}"),
            KlyronError::Tls(e) => format!("TLS error: {e}"),
            KlyronError::Dns(e) => format!("DNS resolution failed: {e}"),
            KlyronError::Timeout(e) => format!("Operation timed out: {e}"),
            KlyronError::ConfigNotFound(p) => format!("Configuration file not found: {p}"),
            KlyronError::ConfigParse(e) => format!("Failed to parse configuration: {e}"),
            KlyronError::ConfigInvalid(e) => format!("Invalid configuration: {e}"),
            KlyronError::ConfigMissing(k) => format!("Required configuration key missing: {k}"),
            KlyronError::EngineFailure(e) => format!("JavaScript engine failure: {e}"),
            KlyronError::ScriptError { exit_code, stderr } => format!("Script exited with code {exit_code}: {stderr}"),
            KlyronError::ModuleNotFound(m) => format!("Module not found: {m}"),
            KlyronError::SyntaxError(e) => format!("Syntax error: {e}"),
            KlyronError::PackageNotFound(p) => format!("Package not found: {p}"),
            KlyronError::VersionNotFound(v) => format!("Version not found: {v}"),
            KlyronError::LockfileStale => "Lockfile is stale, run `klyron install` to update".into(),
            KlyronError::LockfileCorrupt(e) => format!("Lockfile is corrupt: {e}"),
            KlyronError::IntegrityError(e) => format!("Integrity check failed: {e}"),
            KlyronError::ResolutionError(e) => format!("Dependency resolution failed: {e}"),
            KlyronError::DownloadError(e) => format!("Download failed: {e}"),
            KlyronError::RegistryError(e) => format!("Registry error: {e}"),
            KlyronError::PermissionDenied(r) => format!("Permission denied: {r}"),
            KlyronError::NetworkBlocked(r) => format!("Network access blocked: {r}"),
            KlyronError::FileSystemBlocked(r) => format!("Filesystem access blocked: {r}"),
            KlyronError::Internal(e) => format!("Internal error: {e}"),
            KlyronError::Unimplemented(f) => format!("Feature not implemented: {f}"),
            KlyronError::Bug(e) => format!("Unexpected bug: {e}"),
        }
    }

    pub fn suggestion(&self) -> Option<String> {
        match self {
            KlyronError::Io(_) => Some("Check file permissions and disk space".into()),
            KlyronError::Network(_) => Some("Check your internet connection or proxy settings".into()),
            KlyronError::Tls(_) => Some("Check your system time and CA certificates".into()),
            KlyronError::Dns(_) => Some("Check your DNS configuration".into()),
            KlyronError::Timeout(_) => Some("The operation took too long. Try increasing timeout or check network".into()),
            KlyronError::ConfigNotFound(p) => Some(format!("Run `klyron init` to create a config file at {p}")),
            KlyronError::ConfigParse(_) => Some("Check the config file syntax (JSON or TOML)".into()),
            KlyronError::ConfigInvalid(_) => Some("Review the config file for unsupported values".into()),
            KlyronError::ConfigMissing(k) => Some(format!("Add '{k}' to your configuration file")),
            KlyronError::EngineFailure(_) => Some("Try a different engine with --engine (v8, boa, quickjs, jsc)".into()),
            KlyronError::ScriptError { .. } => Some("Review your script for errors".into()),
            KlyronError::ModuleNotFound(m) => Some(format!("Install the module: `klyron add {m}`")),
            KlyronError::SyntaxError(_) => Some("Check your code for syntax errors".into()),
            KlyronError::PackageNotFound(p) => Some(format!("Verify the package name: `klyron search {p}`")),
            KlyronError::VersionNotFound(v) => Some(format!("Check available versions for the package: `klyron info <package>` (version: {v})")),
            KlyronError::LockfileStale => Some("Run `klyron install` to update the lockfile".into()),
            KlyronError::LockfileCorrupt(_) => Some("Delete the lockfile and run `klyron install` again".into()),
            KlyronError::IntegrityError(_) => Some("The downloaded package does not match the expected checksum. Try again or check registry.".into()),
            KlyronError::ResolutionError(_) => Some("Try `klyron dedupe` to resolve conflicts".into()),
            KlyronError::DownloadError(_) => Some("Check your internet connection or try again later".into()),
            KlyronError::RegistryError(_) => Some("Check the registry URL in your configuration".into()),
            KlyronError::PermissionDenied(r) => Some(format!("Grant the required permission for: {r}")),
            KlyronError::NetworkBlocked(_) => Some("Use --allow-net to enable network access".into()),
            KlyronError::FileSystemBlocked(_) => Some("Use --allow-read/--allow-write to enable filesystem access".into()),
            KlyronError::Internal(_) => Some("This is likely a bug. Please report it.".into()),
            KlyronError::Unimplemented(f) => Some(format!("{f} is not yet implemented. Check future releases.")),
            KlyronError::Bug(_) => Some("Please report this bug at https://github.com/dextryayers/klyron/issues".into()),
        }
    }

    pub fn user_message(&self) -> String {
        match self {
            KlyronError::ScriptError { exit_code, stderr } => {
                if stderr.is_empty() {
                    format!("Script failed with exit code {exit_code}")
                } else {
                    format!("Script failed (exit {exit_code}): {stderr}")
                }
            }
            other => other.context(),
        }
    }

    pub fn should_report_bug(&self) -> bool {
        matches!(self, KlyronError::Bug(_) | KlyronError::Internal(_))
    }
}

impl std::fmt::Display for KlyronError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.user_message())
    }
}

impl std::error::Error for KlyronError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            KlyronError::Io(e) => Some(e),
            _ => None,
        }
    }
}

impl From<std::io::Error> for KlyronError {
    fn from(e: std::io::Error) -> Self {
        KlyronError::Io(e)
    }
}

use crate::colors::{Color, supports_color};

fn code_for_error(err: &KlyronError) -> &str {
    match err {
        KlyronError::Io(_) => "FS001",
        KlyronError::Network(_) => "NET001",
        KlyronError::Tls(_) => "NET002",
        KlyronError::Dns(_) => "NET003",
        KlyronError::Timeout(_) => "NET004",
        KlyronError::ConfigNotFound(_) => "CFG001",
        KlyronError::ConfigParse(_) => "CFG002",
        KlyronError::ConfigInvalid(_) => "CFG003",
        KlyronError::ConfigMissing(_) => "CFG004",
        KlyronError::EngineFailure(_) => "ENG001",
        KlyronError::ScriptError { .. } => "SCR001",
        KlyronError::ModuleNotFound(_) => "MOD001",
        KlyronError::SyntaxError(_) => "SYN001",
        KlyronError::PackageNotFound(_) => "PM001",
        KlyronError::VersionNotFound(_) => "PM002",
        KlyronError::LockfileStale => "PM003",
        KlyronError::LockfileCorrupt(_) => "PM004",
        KlyronError::IntegrityError(_) => "SEC001",
        KlyronError::ResolutionError(_) => "PM005",
        KlyronError::DownloadError(_) => "PM006",
        KlyronError::RegistryError(_) => "REG001",
        KlyronError::PermissionDenied(_) => "SEC002",
        KlyronError::NetworkBlocked(_) => "SEC003",
        KlyronError::FileSystemBlocked(_) => "SEC004",
        KlyronError::Internal(_) => "INT001",
        KlyronError::Unimplemented(_) => "GEN001",
        KlyronError::Bug(_) => "BUG001",
    }
}

pub fn format_error(err: &KlyronError, verbose: bool) -> String {
    let use_color = supports_color();
    let code = code_for_error(err);
    let msg = err.user_message();

    if !use_color {
        let mut out = format!("error[{code}]: {msg}");
        if let Some(suggestion) = err.suggestion() {
            out.push_str(&format!("\n  Suggestion: {suggestion}"));
        }
        if verbose {
            out.push_str(&format!("\n  Context: {}", err.context()));
            out.push_str(&format!("\n  Exit code: {}", err.exit_code().into_integer()));
        }
        return out;
    }

    let red = Color::RED;
    let yellow = Color::YELLOW;
    let cyan = Color::CYAN;
    let dim = Color::DIM;
    let bold = Color::BOLD;

    let border = "\u{2500}";
    let top = format!(
        "{bold}{red}{border}{border}{border} Klyron Error {border}{border}{border}{border}{border}{border}{border}{border}{border}{border}{border}{border}{border}{border}{border}{border}{border}{border}{border}{border}{border}{border}{border}{border}{border}{border}{border}{border}{border}{border}{border}{border}{reset}",
        bold = bold.code, red = red.code, reset = Color::RESET.code
    );
    let bottom = format!(
        "{bold}{red}{border}{border}{border}{border}{border}{border}{border}{border}{border}{border}{border}{border}{border}{border}{border}{border}{border}{border}{border}{border}{border}{border}{border}{border}{border}{border}{border}{border}{border}{border}{border}{border}{border}{border}{border}{border}{border}{border}{border}{border}{border}{border}{border}{border}{reset}",
        bold = bold.code, red = red.code, reset = Color::RESET.code
    );

    let mut out = String::new();
    out.push_str(&top);
    out.push_str(&format!("\n{red}\u{2718} {bold}error[{code}]: {msg}{reset}", red = red.code, bold = bold.code, reset = Color::RESET.code));

    if verbose {
        out.push_str(&format!("\n{cyan}  Context:{reset} {}", cyan.code, Color::RESET.code, err.context()));
        out.push_str(&format!("\n{dim}  Location: (see trace){reset}", dim = dim.code, reset = Color::RESET.code));
    }

    if let Some(suggestion) = err.suggestion() {
        out.push_str(&format!("\n\n{yellow}  \u{2500}{border} Did you know? {border}{border}{border}{reset}", yellow = yellow.code, border = border, reset = Color::RESET.code));
        for (i, line) in suggestion.lines().enumerate() {
            out.push_str(&format!("\n{yellow}    {}. {}{reset}", yellow.code, line, Color::RESET.code, i + 1));
        }
    }

    if err.should_report_bug() {
        out.push_str(&format!("\n\n{cyan}  This looks like a bug! Report at https://github.com/dextryayers/klyron/issues{reset}", cyan = cyan.code, reset = Color::RESET.code));
    }

    out.push_str(&format!("\n{}", bottom));
    out
}

pub fn format_error_suggestion(err: &KlyronError) -> Option<String> {
    err.suggestion()
}
