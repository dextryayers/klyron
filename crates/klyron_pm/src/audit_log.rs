use chrono::Utc;
use serde::Serialize;
use std::path::PathBuf;
use std::sync::Mutex;

/// Security audit logging for Klyron.
/// Logs security-relevant events to ~/.klyron/logs/audit.log
/// Automatically rotates logs at 10MB.

const MAX_LOG_SIZE: u64 = 10 * 1024 * 1024; // 10 MB
const LOG_DIR: &str = ".klyron/logs";
const LOG_FILE: &str = "audit.log";

#[derive(Debug, Clone, Serialize)]
pub struct AuditEvent {
    pub timestamp: String,
    pub action: String,
    pub user: String,
    pub ip: String,
    pub success: bool,
    pub details: String,
}

pub struct AuditLogger {
    log_path: PathBuf,
    current_size: Mutex<u64>,
}

impl AuditLogger {
    pub fn new() -> Self {
        let log_dir = dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("/tmp"))
            .join(LOG_DIR);
        std::fs::create_dir_all(&log_dir).ok();

        let log_path = log_dir.join(LOG_FILE);
        let current_size = std::fs::metadata(&log_path)
            .map(|m| m.len())
            .unwrap_or(0);

        Self {
            log_path,
            current_size: Mutex::new(current_size),
        }
    }

    pub fn log(
        &self,
        action: &str,
        user: &str,
        ip: &str,
        success: bool,
        details: &str,
    ) -> std::io::Result<()> {
        let event = AuditEvent {
            timestamp: Utc::now().to_rfc3339(),
            action: action.to_string(),
            user: user.to_string(),
            ip: ip.to_string(),
            success,
            details: details.to_string(),
        };

        let line = serde_json::to_string(&event)? + "\n";
        let line_len = line.len() as u64;

        let mut size = self.current_size.lock().unwrap();
        if *size + line_len > MAX_LOG_SIZE {
            self.rotate()?;
            *size = 0;
        }

        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.log_path)?;

        use std::io::Write;
        file.write_all(line.as_bytes())?;
        *size += line_len;

        Ok(())
    }

    fn rotate(&self) -> std::io::Result<()> {
        let rotated = self.log_path.with_extension("audit.old.log");
        let _ = std::fs::remove_file(&rotated);
        std::fs::rename(&self.log_path, &rotated)?;
        Ok(())
    }

    pub fn log_login(&self, user: &str, ip: &str, success: bool) {
        let _ = self.log("login", user, ip, success, if success { "Login successful" } else { "Login failed" });
    }

    pub fn log_token_usage(&self, user: &str, ip: &str, action: &str) {
        let _ = self.log("token_usage", user, ip, true, &format!("Token used for {action}"));
    }

    pub fn log_permission_grant(&self, module: &str, permission: &str, resource: &str) {
        let _ = self.log(
            "permission_grant",
            module,
            "local",
            true,
            &format!("Granted {permission} for {resource}"),
        );
    }

    pub fn log_permission_deny(&self, module: &str, permission: &str, resource: &str) {
        let _ = self.log(
            "permission_deny",
            module,
            "local",
            false,
            &format!("Denied {permission} for {resource}"),
        );
    }

    pub fn log_security_event(&self, event_type: &str, details: &str) {
        let _ = self.log(event_type, "system", "internal", true, details);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audit_log_creation() {
        let logger = AuditLogger::new();
        assert!(logger.log_path.to_string_lossy().contains("audit.log"));
    }

    #[test]
    fn test_audit_event_serialization() {
        let event = AuditEvent {
            timestamp: "2026-01-01T00:00:00Z".into(),
            action: "test".into(),
            user: "user".into(),
            ip: "127.0.0.1".into(),
            success: true,
            details: "test event".into(),
        };
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("test"));
        assert!(json.contains("127.0.0.1"));
    }

    #[test]
    fn test_log_write() {
        let tmp = std::env::temp_dir().join("klyron_audit_test");
        std::fs::create_dir_all(&tmp).ok();
        std::env::set_var("HOME", &tmp);

        let logger = AuditLogger::new();
        logger.log("test_action", "tester", "::1", true, "unit test").unwrap();

        let content = std::fs::read_to_string(&logger.log_path).unwrap();
        assert!(content.contains("test_action"));
        assert!(content.contains("tester"));

        let _ = std::fs::remove_dir_all(&tmp);
    }
}
