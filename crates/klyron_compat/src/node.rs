use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{BrowserSupport, CompatCheck, CompatStatus};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeCompatConfig {
    pub version: String,
    pub features: HashMap<String, bool>,
}

pub fn check_node_version(version: &str) -> Vec<CompatCheck> {
    let mut checks = Vec::new();
    let ver: u64 = version
        .trim_start_matches('v')
        .split('.')
        .next()
        .unwrap_or("0")
        .parse()
        .unwrap_or(0);

    let features: Vec<(&str, u64)> = vec![
        ("fetch", 18),
        ("WebStreams", 18),
        ("WebSocket", 21),
        ("test_runner", 18),
        ("watch_mode", 22),
    ];

    for (feature, min_version) in features {
        let supported = ver >= min_version;
        checks.push(CompatCheck {
            name: format!("node_{feature}"),
            status: if supported {
                CompatStatus::Compatible
            } else {
                CompatStatus::Partial
            },
            message: if supported {
                format!("Node.js {feature} is supported in v{version}")
            } else {
                format!(
                    "Node.js {feature} requires v{min_version}+, current is v{version}"
                )
            },
            suggestion: if !supported {
                Some(format!("Upgrade to Node.js v{min_version}+ for {feature} support"))
            } else {
                None
            },
        });
    }

    checks
}

pub fn node_browser_matrix() -> Vec<BrowserSupport> {
    vec![
        BrowserSupport {
            browser: "node".into(),
            version: "22".into(),
            supported: true,
            notes: Some("Full ES2024 support".into()),
        },
        BrowserSupport {
            browser: "node".into(),
            version: "20".into(),
            supported: true,
            notes: Some("ES2022 support".into()),
        },
        BrowserSupport {
            browser: "node".into(),
            version: "18".into(),
            supported: true,
            notes: Some("ES2021 support".into()),
        },
    ]
}
