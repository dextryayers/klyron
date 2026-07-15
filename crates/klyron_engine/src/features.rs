use std::collections::HashMap;

use crate::engine::JsEngineKind;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JsFeature {
    pub name: &'static str,
    pub es_version: &'static str,
    pub supported: bool,
}

const ES2020_FEATURES: &[&str] = &[
    "optional_chaining",
    "nullish_coalescing",
    "dynamic_import",
    "bigint",
    "global_this",
    "match_all",
    "promise_all_settled",
];

const ES2021_FEATURES: &[&str] = &[
    "string_replace_all",
    "promise_any",
    "weak_ref",
    "logical_assignment",
    "numeric_separator",
];

const ES2022_FEATURES: &[&str] = &[
    "class_fields",
    "private_methods",
    "top_level_await",
    "error_cause",
    "array_at",
    "object_has_own",
];

const ES2023_FEATURES: &[&str] = &[
    "array_find_last",
    "hashbang_grammar",
    "weak_ref_deref",
];

const ES2024_FEATURES: &[&str] = &[
    "group_by",
    "array_group",
    "promise_with_resolvers",
];

const ES2025_FEATURES: &[&str] = &[
    "iterator_helpers",
    "set_methods",
    "duplicate_named_capture_groups",
];

fn engine_feature_map(engine: JsEngineKind) -> HashMap<&'static str, bool> {
    let mut map = HashMap::new();
    match engine {
        JsEngineKind::V8 => {
            for f in ES2020_FEATURES { map.insert(*f, true); }
            for f in ES2021_FEATURES { map.insert(*f, true); }
            for f in ES2022_FEATURES { map.insert(*f, true); }
            for f in ES2023_FEATURES { map.insert(*f, true); }
            for f in ES2024_FEATURES { map.insert(*f, true); }
            for f in ES2025_FEATURES { map.insert(*f, true); }
        }
        JsEngineKind::Boa => {
            for f in ES2020_FEATURES { map.insert(*f, true); }
            for f in ES2021_FEATURES { map.insert(*f, true); }
            for f in ES2022_FEATURES { map.insert(*f, true); }
            for f in ES2023_FEATURES { map.insert(*f, true); }
            for f in ES2024_FEATURES { map.insert(*f, false); }
            for f in ES2025_FEATURES { map.insert(*f, false); }
        }
        JsEngineKind::QuickJS => {
            for f in ES2020_FEATURES { map.insert(*f, true); }
            for f in ES2021_FEATURES { map.insert(*f, true); }
            for f in ES2022_FEATURES { map.insert(*f, true); }
            for f in ES2023_FEATURES { map.insert(*f, false); }
            for f in ES2024_FEATURES { map.insert(*f, false); }
            for f in ES2025_FEATURES { map.insert(*f, false); }
        }
        JsEngineKind::JSC => {
            for f in ES2020_FEATURES { map.insert(*f, true); }
            for f in ES2021_FEATURES { map.insert(*f, true); }
            for f in ES2022_FEATURES { map.insert(*f, true); }
            for f in ES2023_FEATURES { map.insert(*f, true); }
            for f in ES2024_FEATURES { map.insert(*f, true); }
            for f in ES2025_FEATURES { map.insert(*f, false); }
        }
    }
    map
}

pub struct FeatureDetector;

impl FeatureDetector {
    pub fn detect(engine: JsEngineKind) -> Vec<JsFeature> {
        let feature_map = engine_feature_map(engine);
        let all_features = ES2020_FEATURES.iter()
            .chain(ES2021_FEATURES.iter())
            .chain(ES2022_FEATURES.iter())
            .chain(ES2023_FEATURES.iter())
            .chain(ES2024_FEATURES.iter())
            .chain(ES2025_FEATURES.iter());

        let mut features = Vec::new();
        for f in all_features {
            let es_version = if ES2020_FEATURES.contains(f) { "ES2020" }
                else if ES2021_FEATURES.contains(f) { "ES2021" }
                else if ES2022_FEATURES.contains(f) { "ES2022" }
                else if ES2023_FEATURES.contains(f) { "ES2023" }
                else if ES2024_FEATURES.contains(f) { "ES2024" }
                else { "ES2025" };

            features.push(JsFeature {
                name: f,
                es_version,
                supported: *feature_map.get(f).unwrap_or(&false),
            });
        }
        features
    }

    pub fn unsupported_features(engine: JsEngineKind) -> Vec<&'static str> {
        Self::detect(engine)
            .into_iter()
            .filter(|f| !f.supported)
            .map(|f| f.name)
            .collect()
    }

    pub fn feature_matrix() -> HashMap<JsEngineKind, Vec<JsFeature>> {
        let mut matrix = HashMap::new();
        for engine in JsEngineKind::all() {
            matrix.insert(engine, Self::detect(engine));
        }
        matrix
    }

    pub fn auto_polyfill(code: &str, engine: JsEngineKind) -> String {
        let unsupported = Self::unsupported_features(engine);
        let mut result = String::new();

        for feature in &unsupported {
            match *feature {
                "optional_chaining" => {
                    result.push_str("// polyfill: optional chaining not supported, using manual checks\n");
                }
                "nullish_coalescing" => {
                    result.push_str("// polyfill: nullish coalescing not supported\n");
                }
                "bigint" => {
                    result.push_str("// polyfill: BigInt not supported, using number fallback\n");
                }
                "array_find_last" => {
                    result.push_str("// polyfill: Array.findLast not supported\n");
                }
                _ => {
                    result.push_str(&format!("// polyfill: {} not available, manual workaround needed\n", feature));
                }
            }
        }

        result.push_str(code);
        result
    }
}
