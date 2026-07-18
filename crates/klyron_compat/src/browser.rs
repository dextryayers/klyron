use serde::{Deserialize, Serialize};

use crate::{BrowserSupport, CompatCheck, CompatStatus};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserCompatConfig {
    pub target: String,
    pub min_version: String,
}

pub fn check_browser_api(name: &str) -> Vec<CompatCheck> {
    let apis: Vec<(&str, &[&str])> = vec![
        ("fetch", &["chrome", "firefox", "safari", "edge"]),
        ("WebSocket", &["chrome", "firefox", "safari", "edge"]),
        ("WebAssembly", &["chrome", "firefox", "safari", "edge"]),
        ("WebGPU", &["chrome", "firefox"]),
        ("ServiceWorker", &["chrome", "firefox", "safari", "edge"]),
        ("IndexedDB", &["chrome", "firefox", "safari", "edge"]),
        ("WebRTC", &["chrome", "firefox", "safari", "edge"]),
        ("WebAudio", &["chrome", "firefox", "safari", "edge"]),
    ];

    let mut checks = Vec::new();
    for (api, browsers) in &apis {
        if name.is_empty() || name == *api {
            checks.push(CompatCheck {
                name: format!("browser_{}", api.to_lowercase()),
                status: CompatStatus::Compatible,
                message: format!(
                    "{api} is supported in: {}",
                    browsers.join(", ")
                ),
                suggestion: None,
            });
        }
    }

    checks
}

pub fn browser_support_matrix() -> Vec<BrowserSupport> {
    vec![
        BrowserSupport {
            browser: "Chrome".into(),
            version: "120".into(),
            supported: true,
            notes: Some("Full ES2024 support".into()),
        },
        BrowserSupport {
            browser: "Firefox".into(),
            version: "120".into(),
            supported: true,
            notes: Some("Full ES2024 support".into()),
        },
        BrowserSupport {
            browser: "Safari".into(),
            version: "17".into(),
            supported: true,
            notes: Some("ES2023 support".into()),
        },
        BrowserSupport {
            browser: "Edge".into(),
            version: "120".into(),
            supported: true,
            notes: Some("Full ES2024 support".into()),
        },
    ]
}
