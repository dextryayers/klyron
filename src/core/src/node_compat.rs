/// Node.js compatibility polyfills, converted from ESM to CJS for require()
/// Each function takes (exports, module, require, __dirname, __filename) and populates exports.

/// Generate the JS code that defines all node:* module polyfills
pub fn generate_node_polyfills() -> String {
    let modules = [
        ("assert", include_str!("../../ext/node/js/assert.js")),
        ("buffer", include_str!("../../ext/node/js/buffer.js")),
        ("child_process", include_str!("../../ext/node/js/child_process.js")),
        ("crypto", include_str!("../../ext/node/js/crypto.js")),
        ("events", include_str!("../../ext/node/js/events.js")),
        ("fs", include_str!("../../ext/node/js/fs.js")),
        ("os", include_str!("../../ext/node/js/os.js")),
        ("path", include_str!("../../ext/node/js/path.js")),
        ("process", include_str!("../../ext/node/js/process.js")),
        ("querystring", include_str!("../../ext/node/js/querystring.js")),
        ("stream", include_str!("../../ext/node/js/stream.js")),
        ("string_decoder", include_str!("../../ext/node/js/string_decoder.js")),
        ("url", include_str!("../../ext/node/js/url.js")),
        ("util", include_str!("../../ext/node/js/util.js")),
    ];

    let mut js = String::from(
        "(function() {\n\
         if (typeof globalThis.__klyron_node_modules !== 'undefined') return;\n\
         const __klyron_node_modules = {};\n",
    );

    for (name, source) in &modules {
        let cjs = esm_to_cjs(source);
        js.push_str(&format!(
            "__klyron_node_modules['node:{}'] = function(exports, module, require, __dirname, __filename) {{\n",
            name
        ));
        js.push_str(&cjs);
        js.push_str("\n};\n");
    }

    js.push_str(
        "globalThis.__klyron_node_modules = __klyron_node_modules;\n\
         })();\n",
    );
    js
}

/// Convert ESM source to CJS-compatible code:
/// - Remove `import` lines
/// - Convert `export function` -> `exports.X = function`
/// - Convert `export class` -> `exports.X = class`
/// - Convert `export default` -> `exports.default =`
/// - Convert `export { X, Y }` -> `exports.X = X; exports.Y = Y;`
/// - Convert `export const/let/var` -> `exports.X =`
/// - Convert `export * as` -> removed (handled by module map)
fn esm_to_cjs(source: &str) -> String {
    let mut out = String::new();
    let mut pending_exports: Vec<String> = Vec::new();

    for line in source.lines() {
        let trimmed = line.trim();

        // Handle import statements
        if trimmed.starts_with("import ") {
            // import { ... } from "ext:core/ops" -> const { ... } = Deno.core.ops;
            if let Some(braces) = trimmed.strip_prefix("import {") {
                if let Some(from_pos) = braces.find("from") {
                    let from_part = braces[from_pos + 4..].trim().trim_matches(&['"', ';', '\''][..]).trim();
                    let names_part = braces[..from_pos].trim().trim_end_matches('}').trim();
                    if from_part == "ext:core/ops" {
                        out.push_str(&format!("const {{ {} }} = Deno.core.ops;\n", names_part));
                    } else if from_part.starts_with("./") || from_part.starts_with("../") {
                        // Rewrite relative imports to require() with node module name
                        let mod_name = from_part.trim_end_matches(".js").trim_end_matches(".mjs");
                        let node_name = mod_name.trim_start_matches("./").trim_start_matches("../");
                        out.push_str(&format!("const {{ {} }} = require(\"{node_name}\");\n", names_part));
                    }
                    // Other imports: skip
                    continue;
                }
            }
            // import default or side-effect imports: skip
            continue;
        }

        // Handle `export default function` / `export default class` / `export default {`
        if trimmed.starts_with("export default ") || trimmed == "export default {" {
                let rest = &trimmed[15..]; // after "export default " (15 chars)
            if rest.starts_with("{") || rest.starts_with("[") || rest.starts_with("function") || rest.starts_with("class") {
                out.push_str(&line.replace("export default ", "exports.default = "));
                out.push('\n');
            } else {
                // `export default Foo;` or `export default 42;`
                out.push_str("exports.default = ");
                out.push_str(rest);
                out.push('\n');
            }
            continue;
        }

        // Convert `export function foo(` -> `foo(` with pending export
        if let Some(rest) = trimmed.strip_prefix("export function ") {
            let func_name = rest.split('(').next().unwrap_or("").trim();
            // output the function def without 'export '
            out.push_str(&line.replace("export function ", "function "));
            out.push('\n');
            if !func_name.is_empty() {
                pending_exports.push(format!("exports.{} = {};", func_name, func_name));
            }
            continue;
        }

        // Convert `export class Foo` -> `Foo`
        if let Some(rest) = trimmed.strip_prefix("export class ") {
            let class_name = rest.split(' ').next().unwrap_or("").trim();
            out.push_str(&line.replace("export class ", "class "));
            out.push('\n');
            if !class_name.is_empty() && class_name != "extends" {
                pending_exports.push(format!("exports.{} = {};", class_name, class_name));
            }
            continue;
        }

        // Convert `export const`, `export let`, `export var`
        if trimmed.starts_with("export const ") || trimmed.starts_with("export let ") || trimmed.starts_with("export var ") {
            let decl_type = if trimmed.starts_with("export const ") { "const" }
                else if trimmed.starts_with("export let ") { "let" }
                else { "var" };
            let after_export = trimmed.trim_start_matches("export ");
            // Extract variable name(s)
            let after_decl = after_export.trim_start_matches(decl_type).trim();
            let var_names: Vec<&str> = after_decl.split('=').next().unwrap_or("").split(',').map(|s| s.trim()).filter(|s| !s.is_empty()).collect();
            out.push_str(&line.replace("export ", ""));
            out.push('\n');
            for vn in var_names {
                pending_exports.push(format!("exports.{} = {};", vn, vn));
            }
            continue;
        }

        // Handle export { ... } lines
        if trimmed.starts_with("export {") && trimmed.ends_with(';') {
            let inner = &trimmed[8..trimmed.len()-1].trim(); // between "export {" and "};"
            let inner = inner.trim_end_matches('}').trim();
            for item in inner.split(',') {
                let item = item.trim();
                if item.is_empty() { continue; }
                if let Some(pos) = item.find(" as ") {
                    let local = item[..pos].trim();
                    let exported = item[pos + 4..].trim();
                    pending_exports.push(format!("exports.{} = {};", exported, local));
                } else {
                    pending_exports.push(format!("exports.{} = {};", item, item));
                }
            }
            continue;
        }

        // Handle `export * as name from` — remove entirely
        if trimmed.starts_with("export * as ") && trimmed.contains(" from ") {
            continue;
        }

        // Handle `export * from` — forward all exports (simplified: skip)
        if trimmed.starts_with("export * from") {
            continue;
        }

        // Normal line
        if trimmed.starts_with("export ") {
            // Catch-all: remove 'export ' prefix and try to add to exports
            out.push_str(&line.replacen("export ", "", 1));
            out.push('\n');
            continue;
        }

        out.push_str(line);
        out.push('\n');
    }

    // Append pending exports
    for pe in &pending_exports {
        out.push_str(pe);
        out.push('\n');
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_esm_to_cjs_export_function() {
        let result = esm_to_cjs("export function foo() { return 1; }");
        assert!(result.contains("function foo()"));
        assert!(result.contains("exports.foo = foo;"));
    }

    #[test]
    fn test_esm_to_cjs_export_class() {
        let result = esm_to_cjs("export class Foo { constructor() {} }");
        assert!(result.contains("class Foo {"));
        assert!(result.contains("exports.Foo = Foo;"));
    }

    #[test]
    fn test_esm_to_cjs_export_default() {
        let result = esm_to_cjs("export default 42;");
        assert!(result.contains("exports.default = 42;"));
    }

    #[test]
    fn test_esm_to_cjs_export_const() {
        let result = esm_to_cjs("export const foo = 1;");
        assert!(result.contains("const foo = 1;"));
        assert!(result.contains("exports.foo = foo;"));
    }

    #[test]
    fn test_esm_to_cjs_import_removed() {
        let result = esm_to_cjs("import { op_fs_read_file } from \"ext:core/ops\";\nexport function foo() {}");
        assert!(!result.contains("import"));
        assert!(result.contains("function foo()"));
    }

    #[test]
    fn test_generate_node_polyfills_includes_all() {
        let js = generate_node_polyfills();
        assert!(js.contains("node:fs"));
        assert!(js.contains("node:path"));
        assert!(js.contains("node:buffer"));
        assert!(js.contains("node:os"));
        assert!(js.contains("node:crypto"));
        assert!(js.contains("node:events"));
        assert!(js.contains("node:stream"));
        assert!(js.contains("node:util"));
        assert!(js.contains("node:url"));
        assert!(js.contains("node:querystring"));
        assert!(js.contains("node:assert"));
        assert!(js.contains("node:child_process"));
        assert!(js.contains("node:string_decoder"));
        assert!(js.contains("node:process"));
    }
}
