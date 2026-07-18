pub mod cache;
pub mod resolve;

pub use cache::*;
pub use resolve::*;

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;
    use std::collections::HashMap;
    use std::path::Path;
    use url::Url;

    #[test]
    fn test_module_kind_from_extension() {
        assert_eq!(ModuleKind::from_extension(".mjs"), ModuleKind::Mjs);
        assert_eq!(ModuleKind::from_extension(".cjs"), ModuleKind::Cjs);
        assert_eq!(ModuleKind::from_extension(".json"), ModuleKind::Json);
        assert_eq!(ModuleKind::from_extension(".wasm"), ModuleKind::Wasm);
        assert_eq!(ModuleKind::from_extension(".node"), ModuleKind::Node);
        assert_eq!(ModuleKind::from_extension(".jsx"), ModuleKind::Jsx);
        assert_eq!(ModuleKind::from_extension(".tsx"), ModuleKind::Tsx);
        assert_eq!(ModuleKind::from_extension(".ts"), ModuleKind::TypeScript);
        assert_eq!(ModuleKind::from_extension(".js"), ModuleKind::JavaScript);
    }

    #[test]
    fn test_module_kind_is_esm() {
        assert!(ModuleKind::Mjs.is_esm());
        assert!(ModuleKind::Wasm.is_esm());
        assert!(!ModuleKind::Cjs.is_esm());
        assert!(!ModuleKind::Json.is_esm());
    }

    #[test]
    fn test_import_map_exact() {
        let mut map = ImportMap::new();
        map.imports
            .insert("preact".into(), "https://esm.sh/preact".into());
        assert_eq!(
            map.resolve("preact", None),
            Some("https://esm.sh/preact".into())
        );
    }

    #[test]
    fn test_import_map_prefix() {
        let mut map = ImportMap::new();
        map.imports
            .insert("std/".into(), "https://deno.land/std@0.224.0/".into());
        assert_eq!(
            map.resolve("std/fs/mod.ts", None),
            Some("https://deno.land/std@0.224.0/fs/mod.ts".into())
        );
    }

    #[test]
    fn test_import_map_wildcard() {
        let mut map = ImportMap::new();
        map.imports
            .insert("std/*".into(), "https://deno.land/std@0.224.0/*".into());
        assert_eq!(
            map.resolve("std/fs", None),
            Some("https://deno.land/std@0.224.0/fs".into())
        );
    }

    #[test]
    fn test_import_map_scopes() {
        let mut map = ImportMap::new();
        map.imports
            .insert("lodash".into(), "https://esm.sh/lodash".into());
        let mut scope = HashMap::new();
        scope.insert(
            "lodash".into(),
            "https://cdn.skypack.dev/lodash".into(),
        );
        map.scopes.insert("/app/".into(), scope);

        assert_eq!(
            map.resolve("lodash", Some("/app/index.ts")),
            Some("https://cdn.skypack.dev/lodash".into())
        );
        assert_eq!(
            map.resolve("lodash", Some("/other/index.ts")),
            Some("https://esm.sh/lodash".into())
        );
    }

    #[test]
    fn test_import_map_from_json() {
        let json: Value = serde_json::from_str(
            r#"{
      "imports": {
        "preact": "https://esm.sh/preact",
        "std/": "https://deno.land/std@0.224.0/"
      }
    }"#,
        )
        .unwrap();
        let map = ImportMap::from_json(&json).unwrap();
        assert_eq!(
            map.resolve("preact", None),
            Some("https://esm.sh/preact".into())
        );
        assert_eq!(
            map.resolve("std/fs/mod.ts", None),
            Some("https://deno.land/std@0.224.0/fs/mod.ts".into())
        );
    }

    #[test]
    fn test_resolve_bare_specifier() {
        let dir = std::env::temp_dir().join("_klyron_loader_bare");
        let _ = std::fs::create_dir_all(&dir.join("node_modules").join("test-pkg"));
        std::fs::write(
            dir.join("node_modules/test-pkg/package.json"),
            r#"{"main":"index.js"}"#,
        )
        .unwrap();
        std::fs::write(dir.join("node_modules/test-pkg/index.js"), "").unwrap();
        let result = ModuleResolver::resolve_bare_specifier("test-pkg", &dir);
        assert!(result.is_ok());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_resolve_bare_specifier_not_found() {
        let result =
            ModuleResolver::resolve_bare_specifier("nonexistent-pkg-xyz", Path::new("/tmp"));
        assert!(result.is_err());
    }

    #[test]
    fn test_detect_format_mjs() {
        assert_eq!(
            ModuleResolver::detect_format(Path::new("/test/file.mjs")),
            ModuleFormat::ESM
        );
    }

    #[test]
    fn test_detect_format_cjs() {
        assert_eq!(
            ModuleResolver::detect_format(Path::new("/test/file.cjs")),
            ModuleFormat::CommonJS
        );
    }

    #[test]
    fn test_cjs_to_esm() {
        let cjs = r#"const path = require("path");
module.exports = { foo: 1 };
exports.bar = 2;"#;
        let esm = cjs_to_esm(cjs);
        assert!(esm.contains("export default"));
        assert!(!esm.contains("module.exports"));
    }

    #[test]
    fn test_esm_to_cjs() {
        let esm = r#"import { readFile } from "fs";
export default { foo: 1 };
export const bar = 2;"#;
        let cjs = esm_to_cjs(esm);
        assert!(cjs.contains("module.exports"));
        assert!(!cjs.contains("export default"));
    }

    #[test]
    fn test_resolve_specifier_https() {
        let url = Url::parse("file:///app/index.js").unwrap();
        let result = ModuleResolver::resolve_specifier("https://esm.sh/preact", &url);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().scheme(), "https");
    }

    #[test]
    fn test_resolve_specifier_node() {
        let url = Url::parse("file:///app/index.js").unwrap();
        let result = ModuleResolver::resolve_specifier("node:fs", &url);
        assert!(result.is_ok());
    }

    #[test]
    fn test_detect_kind() {
        assert_eq!(
            ModuleResolver::detect_kind(Path::new("f.mjs")),
            ModuleKind::Mjs
        );
        assert_eq!(
            ModuleResolver::detect_kind(Path::new("f.cjs")),
            ModuleKind::Cjs
        );
        assert_eq!(
            ModuleResolver::detect_kind(Path::new("f.tsx")),
            ModuleKind::Tsx
        );
        assert_eq!(
            ModuleResolver::detect_kind(Path::new("f.jsx")),
            ModuleKind::Jsx
        );
        assert_eq!(
            ModuleResolver::detect_kind(Path::new("f.node")),
            ModuleKind::Node
        );
        assert_eq!(
            ModuleResolver::detect_kind(Path::new("f.wasm")),
            ModuleKind::Wasm
        );
    }

    #[test]
    fn test_resolver_import_map_path() {
        let resolver = ModuleResolver::new();
        let dir = std::env::temp_dir().join("_klyron_loader_map");
        let _ = std::fs::create_dir_all(&dir);
        let entry = dir.join("index.js");
        std::fs::write(&entry, "").unwrap();
        let result = resolver.resolve("./index.js", &dir);
        assert!(result.is_ok());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_resolver_not_found() {
        let resolver = ModuleResolver::new();
        let result = resolver.resolve("./nonexistent_file_xyz.js", Path::new("/tmp"));
        assert!(result.is_err());
    }

    #[test]
    fn test_package_json_exports() {
        let dir = std::env::temp_dir().join("_klyron_pkg_exports");
        let _ = std::fs::create_dir_all(&dir);
        let pkg: Value = serde_json::from_str(
            r#"{
      "exports": {
        ".": "./dist/index.js",
        "./utils": "./dist/utils.js"
      }
    }"#,
        )
        .unwrap();
        std::fs::write(
            dir.join("package.json"),
            serde_json::to_string(&pkg).unwrap(),
        )
        .unwrap();
        std::fs::create_dir_all(dir.join("dist")).unwrap();
        std::fs::write(dir.join("dist/index.js"), "").unwrap();
        let entry = ModuleResolver::resolve_package_json_entry(&dir, &pkg);
        assert!(entry.is_some());
        assert!(entry.unwrap().ends_with("dist/index.js"));
        let _ = std::fs::remove_dir_all(&dir);
    }
}
