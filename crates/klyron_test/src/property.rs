/// Property-based testing utilities for Klyron
///
/// This module provides property-based test strategies for Klyron data types,
/// powered by the `proptest` crate. Enable with the `property-tests` feature.
///
/// # Example
/// ```ignore
/// use klyron_test::property::*;
/// use proptest::prelude::*;
///
/// proptest! {
///     #![proptest_config = ProptestConfig::with_cases(256)]
///     #[test]
///     fn test_lockfile_roundtrip(package_name in package_name_strategy()) {
///         // Property: Lockfile serialization -> deserialization -> same data
///     }
/// }
/// ```

#[cfg(feature = "property-tests")]
pub mod strategies {
    use proptest::prelude::*;
    use proptest::strategy::Strategy;

    /// Generate a valid npm package name
    pub fn package_name() -> impl Strategy<Value = String> {
        proptest::string::string_regex(r"@{0,1}[a-z0-9][a-z0-9\-\._]{0,50}")
            .expect("Invalid regex")
    }

    /// Generate a valid semver version string
    pub fn version_string() -> impl Strategy<Value = String> {
        proptest::string::string_regex(r"\d+\.\d+\.\d+")
            .expect("Invalid regex")
    }

    /// Generate a valid npm version range (simplified)
    pub fn version_range() -> impl Strategy<Value = String> {
        prop_oneof![
            proptest::string::string_regex(r"\^?[0-9]+\.[0-9]+\.[0-9]+").expect("Invalid regex"),
            proptest::string::string_regex(r"~?[0-9]+\.[0-9]+\.[0-9]+").expect("Invalid regex"),
            proptest::string::string_regex(r">=?[0-9]+\.[0-9]+\.[0-9]+").expect("Invalid regex"),
            Just("*".to_string()),
        ].boxed()
    }

    /// Generate a valid tarball URL
    pub fn tarball_url() -> impl Strategy<Value = String> {
        (package_name(), version_string())
            .prop_map(|(name, ver)| {
                let slug = name.replace('@', "");
                format!("https://registry.npmjs.org/{}/-/{}-{}.tgz", slug, name, ver)
            })
    }

    /// Generate a valid SRI hash string
    pub fn integrity_hash() -> impl Strategy<Value = String> {
        proptest::string::string_regex(r"sha[0-9]+-[A-Za-z0-9+/]{20,}={0,2}")
            .expect("Invalid regex")
    }

    /// Generate a valid package.json content
    pub fn package_json() -> impl Strategy<Value = String> {
        (package_name(), version_string(), proptest::collection::vec(
            (package_name(), version_range()), 0..10
        )).prop_map(|(name, version, deps)| {
            let deps_json: Vec<String> = deps.iter()
                .map(|(k, v)| format!(r#""{}": "{}""#, k, v))
                .collect();
            format!(
                r#"{{"name": "{}", "version": "{}", "dependencies": {{{}}} }}"#,
                name, version, deps_json.join(", ")
            )
        })
    }
}

/// No-op module when proptest feature is disabled
#[cfg(not(feature = "property-tests"))]
pub mod strategies {
    pub fn package_name() -> &'static str {
        ""
    }

    pub fn version_string() -> &'static str {
        ""
    }

    pub fn version_range() -> &'static str {
        ""
    }

    pub fn integrity_hash() -> &'static str {
        ""
    }

    pub fn package_json() -> &'static str {
        ""
    }

    pub fn tarball_url() -> &'static str {
        ""
    }
}

#[cfg(test)]
#[cfg(feature = "property-tests")]
mod tests {
    use super::strategies::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_package_name_contains_no_uppercase(name in package_name()) {
            let name_clean = name.replace("@", "").replace("-", "").replace(".", "").replace("_", "");
            if !name_clean.is_empty() {
                assert!(!name_clean.chars().any(|c| c.is_uppercase()));
            }
        }

        #[test]
        fn test_version_string_is_valid(ver in version_string()) {
            let parts: Vec<&str> = ver.split('.').collect();
            assert_eq!(parts.len(), 3);
            for p in parts {
                assert!(p.parse::<u64>().is_ok());
            }
        }

        #[test]
        fn test_package_json_parses(json in package_json()) {
            let parsed: Result<serde_json::Value, _> = serde_json::from_str(&json);
            assert!(parsed.is_ok());
            let val = parsed.unwrap();
            assert!(val.get("name").and_then(|n| n.as_str()).is_some());
            assert!(val.get("version").and_then(|v| v.as_str()).is_some());
        }
    }
}
