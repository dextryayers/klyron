use serde::{Deserialize, Serialize};

use crate::{BrowserSupport, CompatCheck, CompatStatus};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DenoCompatConfig {
    pub version: String,
    pub permissions: Vec<String>,
}

pub fn check_deno_compat(version: &str) -> Vec<CompatCheck> {
    let mut checks = Vec::new();
    let ver: u64 = version
        .trim_start_matches('v')
        .split('.')
        .next()
        .unwrap_or("0")
        .parse()
        .unwrap_or(0);

    let features: Vec<(&str, u64)> = vec![
        ("Node_compat", 1),
        ("npm_imports", 1),
        ("JSR_support", 2),
        ("Fresh_framework", 1),
        ("WebGPU", 2),
    ];

    for (feature, min_version) in features {
        let supported = ver >= min_version;
        checks.push(CompatCheck {
            name: format!("deno_{}", feature.to_lowercase().replace(' ', "_")),
            status: if supported {
                CompatStatus::Compatible
            } else {
                CompatStatus::Partial
            },
            message: if supported {
                format!("Deno {feature} is supported in v{version}")
            } else {
                format!(
                    "Deno {feature} requires v{min_version}+, current is v{version}"
                )
            },
            suggestion: if !supported {
                Some(format!(
                    "Upgrade to Deno v{min_version}+ for {feature} support"
                ))
            } else {
                None
            },
        });
    }

    checks
}

pub fn deno_browser_matrix() -> Vec<BrowserSupport> {
    vec![
        BrowserSupport {
            browser: "deno".into(),
            version: "2".into(),
            supported: true,
            notes: Some("Full Node.js compat layer".into()),
        },
        BrowserSupport {
            browser: "deno".into(),
            version: "1".into(),
            supported: true,
            notes: Some("Limited Node compat".into()),
        },
    ]
}
