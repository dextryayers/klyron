use std::collections::HashMap;
use std::sync::Mutex;
use rand::RngCore;

/// CORS and CSRF protection for the dev server.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CorsPolicy {
    SameOrigin,
    Open,
    SpecificOrigin,
}

#[derive(Debug, Clone)]
pub struct CorsConfig {
    pub policy: CorsPolicy,
    pub allowed_origin: String,
    pub allowed_methods: Vec<String>,
    pub allowed_headers: Vec<String>,
    pub max_age_seconds: u32,
}

impl Default for CorsConfig {
    fn default() -> Self {
        Self {
            policy: CorsPolicy::SameOrigin,
            allowed_origin: String::new(),
            allowed_methods: vec!["GET".into(), "HEAD".into(), "OPTIONS".into()],
            allowed_headers: vec!["Content-Type".into(), "Authorization".into(), "X-CSRF-Token".into()],
            max_age_seconds: 86400,
        }
    }
}

impl CorsConfig {
    pub fn open() -> Self {
        Self {
            policy: CorsPolicy::Open,
            allowed_origin: "*".into(),
            allowed_methods: vec![
                "GET".into(), "HEAD".into(), "POST".into(), "PUT".into(),
                "DELETE".into(), "PATCH".into(), "OPTIONS".into(),
            ],
            allowed_headers: vec!["*".into()],
            max_age_seconds: 86400,
        }
    }

    pub fn specific(origin: &str) -> Self {
        Self {
            policy: CorsPolicy::SpecificOrigin,
            allowed_origin: origin.to_string(),
            allowed_methods: vec![
                "GET".into(), "HEAD".into(), "POST".into(), "PUT".into(),
                "DELETE".into(), "PATCH".into(), "OPTIONS".into(),
            ],
            allowed_headers: vec!["Content-Type".into(), "Authorization".into(), "X-CSRF-Token".into()],
            max_age_seconds: 86400,
        }
    }

    pub fn validate_origin(&self, origin: &str, host: &str) -> bool {
        match self.policy {
            CorsPolicy::Open => true,
            CorsPolicy::SameOrigin => {
                origin == host
                    || origin == format!("http://{host}")
                    || origin == format!("https://{host}")
                    || origin == format!("http://{host}/")
                    || origin == format!("https://{host}/")
            }
            CorsPolicy::SpecificOrigin => {
                origin == self.allowed_origin
                    || origin.trim_end_matches('/') == self.allowed_origin.trim_end_matches('/')
            }
        }
    }
}

/// CSRF token management
pub struct CsrfProtection {
    tokens: Mutex<HashMap<String, CsrfToken>>,
}

#[derive(Debug, Clone)]
pub struct CsrfToken {
    pub token: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub used: bool,
}

impl CsrfProtection {
    pub fn new() -> Self {
        Self {
            tokens: Mutex::new(HashMap::new()),
        }
    }

    pub fn generate_token(&self, session_id: &str) -> String {
        let mut bytes = vec![0u8; 32];
        rand::rngs::OsRng.fill_bytes(&mut bytes);
        let token = hex::encode(&bytes);

        let mut tokens = self.tokens.lock().unwrap();
        tokens.insert(session_id.to_string(), CsrfToken {
            token: token.clone(),
            created_at: chrono::Utc::now(),
            used: false,
        });

        token
    }

    pub fn validate_token(&self, session_id: &str, token: &str) -> bool {
        let mut tokens = self.tokens.lock().unwrap();
        if let Some(stored) = tokens.get_mut(session_id) {
            if stored.used {
                return false;
            }
            if stored.token == token {
                let age = chrono::Utc::now() - stored.created_at;
                if age.num_minutes() < 30 {
                    stored.used = true;
                    return true;
                }
            }
        }
        false
    }

    pub fn invalidate_session(&self, session_id: &str) {
        self.tokens.lock().unwrap().remove(session_id);
    }

    pub fn cleanup_expired(&self) {
        let mut tokens = self.tokens.lock().unwrap();
        tokens.retain(|_, t| {
            let age = chrono::Utc::now() - t.created_at;
            age.num_hours() < 1
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cors_same_origin() {
        let config = CorsConfig::default();
        assert!(config.validate_origin("http://localhost:3000", "localhost:3000"));
        assert!(!config.validate_origin("http://evil.com", "localhost:3000"));
    }

    #[test]
    fn test_cors_open() {
        let config = CorsConfig::open();
        assert!(config.validate_origin("http://evil.com", "localhost"));
        assert!(config.validate_origin("http://anywhere.org", "test"));
    }

    #[test]
    fn test_cors_specific() {
        let config = CorsConfig::specific("https://myapp.com");
        assert!(config.validate_origin("https://myapp.com", "host"));
        assert!(!config.validate_origin("https://evil.com", "host"));
    }

    #[test]
    fn test_csrf_generate_and_validate() {
        let csrf = CsrfProtection::new();
        let session = "session-abc-123";
        let token = csrf.generate_token(session);

        assert!(!token.is_empty());
        assert!(csrf.validate_token(session, &token));
        // Second use should fail (token marked as used)
        assert!(!csrf.validate_token(session, &token));
    }

    #[test]
    fn test_csrf_invalid_token() {
        let csrf = CsrfProtection::new();
        let session = "session-xyz";
        csrf.generate_token(session);

        assert!(!csrf.validate_token(session, "wrong-token"));
    }

    #[test]
    fn test_csrf_invalidate_session() {
        let csrf = CsrfProtection::new();
        let session = "session-to-invalidate";
        let token = csrf.generate_token(session);
        csrf.invalidate_session(session);

        assert!(!csrf.validate_token(session, &token));
    }

    #[test]
    fn test_csrf_expired_cleanup() {
        let csrf = CsrfProtection::new();
        csrf.generate_token("old-session");
        csrf.cleanup_expired();
        // Should not panic
    }
}
