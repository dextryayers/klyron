use crate::PmError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Provenance {
    pub version: String,
    pub package_name: String,
    pub build_date: String,
    pub builder: String,
    pub source_repo: Option<String>,
    pub build_command: Option<String>,
    pub build_environment: Option<String>,
}

pub fn generate_provenance(
    package_name: &str,
    version: &str,
    build_command: Option<&str>,
) -> Result<String, PmError> {
    let now = chrono_now_iso();
    let source_repo = detect_git_remote();

    let provenance = Provenance {
        version: version.to_string(),
        package_name: package_name.to_string(),
        build_date: now,
        builder: format!("klyron/{}", env!("CARGO_PKG_VERSION")),
        source_repo,
        build_command: build_command.map(|s| s.to_string()),
        build_environment: Some(format!("{}/{}", std::env::consts::OS, std::env::consts::ARCH)),
    };

    serde_json::to_string_pretty(&provenance)
        .map_err(|e| PmError::IoError(format!("Serialization error: {e}")))
}

pub fn verify_provenance(provenance_json: &str) -> bool {
    match serde_json::from_str::<Provenance>(provenance_json) {
        Ok(p) => {
            if p.package_name.is_empty() || p.version.is_empty() || p.build_date.is_empty() {
                return false;
            }
            if p.builder.is_empty() {
                return false;
            }
            true
        }
        Err(_) => false,
    }
}

pub fn parse_provenance(provenance_json: &str) -> Result<Provenance, PmError> {
    serde_json::from_str(provenance_json)
        .map_err(|e| PmError::IoError(format!("Invalid provenance: {e}")))
}

pub fn provenance_to_attestation_json(provenance: &Provenance) -> Result<String, PmError> {
    let attestation = serde_json::json!({
        "type": "https://slsa.dev/provenance/v1",
        "predicateType": "https://slsa.dev/provenance/v1/attestation",
        "subject": [{
            "name": provenance.package_name.clone(),
            "version": provenance.version.clone(),
        }],
        "predicate": {
            "buildDefinition": {
                "buildType": provenance.build_command.clone().unwrap_or_default(),
                "externalParameters": {},
                "resolvedParameters": {},
            },
            "runDetails": {
                "builder": { "id": provenance.builder.clone() },
                "metadata": {
                    "buildStartedOn": provenance.build_date.clone(),
                    "buildFinishedOn": chrono_now_iso(),
                },
            },
        },
    });
    serde_json::to_string_pretty(&attestation)
        .map_err(|e| PmError::IoError(format!("Attestation error: {e}")))
}

fn detect_git_remote() -> Option<String> {
    let output = std::process::Command::new("git")
        .args(["config", "--get", "remote.origin.url"])
        .output()
        .ok()?;
    if output.status.success() {
        Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        None
    }
}

fn chrono_now_iso() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let d = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default();
    let secs = d.as_secs();
    let secs_per_day = 86400;
    let days = secs / secs_per_day;
    let t = secs % secs_per_day;
    let h = t / 3600;
    let m = (t % 3600) / 60;
    let s = t % 60;
    let mut y = 1970i64;
    let mut rem = days as i64;
    loop {
        let di = if (y % 4 == 0 && y % 100 != 0) || y % 400 == 0 { 366 } else { 365 };
        if rem < di { break; }
        rem -= di;
        y += 1;
    }
    let md = if (y % 4 == 0 && y % 100 != 0) || y % 400 == 0 {
        [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };
    let mut mo = 1;
    for &d in &md {
        if rem < d { break; }
        rem -= d;
        mo += 1;
    }
    format!("{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z", y, mo, rem + 1, h, m, s)
}
