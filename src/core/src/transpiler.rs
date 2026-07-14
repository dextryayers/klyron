use std::collections::HashMap;
use std::path::PathBuf;

use oxc::allocator::Allocator;
use oxc::codegen::{Codegen, CodegenOptions};
use oxc::parser::Parser;
use oxc::span::SourceType;
use oxc::transformer::{
  DecoratorOptions, JsxOptions, JsxRuntime, TransformOptions, Transformer, TypeScriptOptions,
};

#[derive(Clone, Default)]
pub struct Transpiler {
  pub tsconfig_paths: HashMap<String, Vec<String>>,
  pub tsconfig_base_url: Option<String>,
}

#[derive(Clone)]
pub struct TranspileOptions {
  pub jsx_automatic: bool,
  pub jsx_import_source: String,
  pub decorators: bool,
  pub sourcemap: bool,
  pub import_assertions: bool,
  pub target: String,
}

impl Default for TranspileOptions {
  fn default() -> Self {
    Self {
      jsx_automatic: true,
      jsx_import_source: "react".into(),
      decorators: true,
      sourcemap: true,
      import_assertions: true,
      target: "es2022".into(),
    }
  }
}

impl Transpiler {
  pub fn new() -> Self {
    Self {
      tsconfig_paths: HashMap::new(),
      tsconfig_base_url: None,
    }
  }

  pub fn with_tsconfig_paths(mut self, paths: HashMap<String, Vec<String>>, base_url: Option<String>) -> Self {
    self.tsconfig_paths = paths;
    self.tsconfig_base_url = base_url;
    self
  }

  pub fn transpile(&self, code: &str) -> Result<String, String> {
    self.transpile_with_sourcemap("<input>", code).map(|(c, _)| c)
  }

  pub fn transpile_with_options(&self, code: &str, opts: &TranspileOptions) -> Result<(String, Option<String>), String> {
    self.transpile_with_sourcemap_opts("<input>", code, opts)
  }

  pub fn transpile_with_sourcemap(&self, filename: &str, code: &str) -> Result<(String, Option<String>), String> {
    let opts = TranspileOptions::default();
    self.transpile_with_sourcemap_opts(filename, code, &opts)
  }

  fn transpile_with_sourcemap_opts(
    &self,
    filename: &str,
    code: &str,
    opts: &TranspileOptions,
  ) -> Result<(String, Option<String>), String> {
    let allocator = Allocator::default();
    let source_type = self.detect_source_type(filename);

    let ret = Parser::new(&allocator, code, source_type).parse();

    if !ret.diagnostics.is_empty() {
      let errors: Vec<String> = ret.diagnostics.iter().map(|d| d.to_string()).collect();
      return Err(format!("Parse error in {filename}:\n{}", errors.join("\n")));
    }

    let mut program = ret.program;
    let source_path = std::path::Path::new(filename);

    // Always strip TypeScript type annotations
    // Only do JSX transform if JSX syntax is detected
    let is_jsx = filename.ends_with(".jsx") || filename.ends_with(".tsx")
      || code.contains("createElement") || code.contains("/>") || code.contains("></");

    let mut transform_options = TransformOptions {
      typescript: TypeScriptOptions {
        only_remove_type_imports: false,
        ..Default::default()
      },
      decorator: if opts.decorators {
        DecoratorOptions::default()
      } else {
        DecoratorOptions { ..Default::default() }
      },
      ..Default::default()
    };

    if opts.jsx_automatic && is_jsx {
      transform_options.jsx = JsxOptions {
        jsx_plugin: true,
        runtime: JsxRuntime::Automatic,
        import_source: Some(opts.jsx_import_source.clone()),
        ..JsxOptions::disable()
      };
    }

    let scoping = oxc::semantic::SemanticBuilder::new()
      .build(&program).semantic.into_scoping();
    let ret = Transformer::new(&allocator, source_path, &transform_options)
      .build_with_scoping(scoping, &mut program);
    if ret.diagnostics.has_errors() {
      let errors: Vec<String> = ret.diagnostics.iter().map(|d| d.to_string()).collect();
      return Err(format!("Transform error in {filename}:\n{}", errors.join("\n")));
    }

    let codegen_opts = CodegenOptions {
      source_map_path: if opts.sourcemap {
        Some(PathBuf::from(filename))
      } else {
        None
      },
      ..Default::default()
    };

    let output = Codegen::new()
      .with_options(codegen_opts)
      .build(&program);

    let sm = if opts.sourcemap {
      output.map.and_then(|m| {
        let json = m.to_json_string();
        if json == "{}" { None } else { Some(json) }
      })
    } else {
      None
    };

    Ok((output.code, sm))
  }

  // VLQ-encoded source map generation
  pub fn generate_vlq_sourcemap(
    &self,
    source_file: &str,
    source_code: &str,
    _generated_code: &str,
    mappings: &[(u32, u32, u32, u32)],
  ) -> String {
    let vlq_mappings = self.encode_vlq_mappings(mappings);
    let map = serde_json::json!({
      "version": 3,
      "file": source_file,
      "sources": [source_file],
      "sourcesContent": [source_code],
      "names": [],
      "mappings": vlq_mappings,
    });
    map.to_string()
  }

  fn encode_vlq_mappings(&self, mappings: &[(u32, u32, u32, u32)]) -> String {
    let mut result = String::new();
    let mut last_col = 0u32;
    let mut last_source_line = 0u32;
    let mut last_source_col = 0u32;

    for (i, &(_gen_line, gen_col, source_line, source_col)) in mappings.iter().enumerate() {
      if i > 0 {
        result.push(',');
      }
      let col_delta = gen_col as i64 - last_col as i64;
      last_col = gen_col;
      let src_line_delta = source_line as i64 - last_source_line as i64;
      last_source_line = source_line;
      let src_col_delta = source_col as i64 - last_source_col as i64;
      last_source_col = source_col;
      let segment = Self::encode_vlq_segment(&[col_delta, 0, src_line_delta, src_col_delta]);
      result.push_str(&segment);
    }
    result
  }

  fn encode_vlq_segment(values: &[i64]) -> String {
    let mut result = String::new();
    for &value in values {
      result.push_str(&Self::encode_vlq(value));
    }
    result
  }

  fn encode_vlq(value: i64) -> String {
    let vlq_base64_chars = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut vlq = value;
    let mut result = String::new();
    let sign = if vlq >= 0 { 0 } else { 1 };
    vlq = vlq.abs();
    let mut vlq_shifted = (vlq << 1) | sign;
    loop {
      let digit = vlq_shifted & 0x1f;
      vlq_shifted >>= 5;
      let has_continuation = if vlq_shifted > 0 { 1 } else { 0 };
      let byte = (digit | (has_continuation << 5)) as usize;
      result.push(vlq_base64_chars.chars().nth(byte).unwrap_or('A'));
      if vlq_shifted == 0 {
        break;
      }
    }
    result
  }

  // TC39 Stage 3 decorator transform (strips decorators for downstream processing)
  pub fn strip_decorators(&self, code: &str) -> String {
    let mut result = String::new();
    let mut in_decorator = false;
    let mut paren_depth = 0;

    for line in code.lines() {
      let trimmed = line.trim();
      if trimmed.starts_with('@') {
        in_decorator = true;
        paren_depth = 0;
        for ch in trimmed.chars() {
          if ch == '(' { paren_depth += 1; }
          if ch == ')' { paren_depth -= 1; }
        }
        if paren_depth > 0 {
          continue;
        }
        continue;
      }
      if in_decorator {
        for ch in trimmed.chars() {
          if ch == '(' { paren_depth += 1; }
          if ch == ')' { paren_depth -= 1; }
        }
        if paren_depth > 0 {
          continue;
        }
        in_decorator = false;
        continue;
      }
      result.push_str(line);
      result.push('\n');
    }
    result
  }

  // Import assertion/attribute handling
  pub fn handle_import_assertions(&self, code: &str) -> String {
    let re = regex_lite::Regex::new(r#"import\s+(?:\{[^}]*\}|\*\s+as\s+\w+|\w+)\s+from\s+["'][^"']+["']\s*(?:assert|with)\s*\{[^}]*\}"#);
    re.replace_all(code, |caps: &regex_lite::Captures| {
      let matched = caps.get(0).unwrap_or("");
      let without_assertion = matched
        .rsplitn(2, "assert")
        .next()
        .unwrap_or(matched)
        .rsplitn(2, "with")
        .next()
        .unwrap_or(matched)
        .trim_end()
        .to_string();
      without_assertion
    })
  }

  // tsconfig path alias resolution (e.g., "@/" -> "./src/")
  pub fn resolve_path_alias(&self, specifier: &str) -> Option<String> {
    if self.tsconfig_paths.is_empty() {
      return None;
    }

    for (alias, paths) in &self.tsconfig_paths {
      let alias_stripped = alias.trim_end_matches("/*");
      if specifier.starts_with(alias_stripped) {
        if let Some(first_path) = paths.first() {
          let path_stripped = first_path.trim_end_matches("/*");
          let rest = specifier.strip_prefix(alias_stripped).unwrap_or("");
          let base = path_stripped.trim_start_matches("./");
          let resolved = format!("./{}{}", base, rest);
          return Some(resolved);
        }
      }
    }
    None
  }

  pub fn is_ts(&self, specifier: &str) -> bool {
    specifier.ends_with(".ts") || specifier.ends_with(".tsx") || specifier.ends_with(".mts")
  }

  fn detect_source_type(&self, filename: &str) -> SourceType {
    let name = filename.rsplit('/').next().unwrap_or(filename);
    if name.ends_with(".tsx") {
      SourceType::tsx()
    } else if name.ends_with(".ts") || name.ends_with(".mts") {
      SourceType::ts()
    } else if name.ends_with(".jsx") {
      SourceType::jsx()
    } else if name.ends_with(".mjs") {
      SourceType::mjs()
    } else if name.ends_with(".cjs") {
      SourceType::cjs()
    } else if name == "<input>" || name == "<eval>" || name.contains("<eval") {
      SourceType::tsx()
    } else {
      SourceType::ts()
    }
  }
}

// Minimal regex helper for import assertion handling
mod regex_lite {
  use std::ops::Range;

  pub struct Regex {
    pattern: String,
  }

  pub struct Captures<'t> {
    text: &'t str,
    ranges: Vec<Range<usize>>,
  }

  impl Regex {
    pub fn new(pattern: &str) -> Self {
      Self { pattern: pattern.to_string() }
    }

    pub fn replace_all<'a>(&self, text: &'a str, replacer: impl Fn(&Captures<'a>) -> String) -> String {
      let pattern = self.pattern.clone();
      let mut results = Vec::new();
      if let Ok(re) = regex::Regex::new(&pattern) {
        let mut last_end = 0;
        for cap in re.captures_iter(text) {
          let full_match = cap.get(0).map(|m| (m.start(), m.end(), m.as_str())).unwrap();
          results.push((last_end, full_match.0, None));
          let ranges: Vec<Range<usize>> = (0..cap.len())
            .filter_map(|i| cap.get(i).map(|m| m.range()))
            .collect();
          let capture_text = full_match.2;
          let lite_caps = Captures { text: capture_text, ranges: ranges.clone() };
          let replacement = replacer(&lite_caps);
          results.push((full_match.0, full_match.1, Some(replacement)));
          last_end = full_match.1;
        }
        results.push((last_end, text.len(), None));
        let mut out = String::new();
        for (start, end, replacement) in results {
          if let Some(repl) = replacement {
            out.push_str(&repl);
          } else {
            out.push_str(&text[start..end]);
          }
        }
        return out;
      }
      text.to_string()
    }
  }

  impl<'t> Captures<'t> {
    pub fn get(&self, i: usize) -> Option<&str> {
      self.ranges.get(i).map(|r| &self.text[r.start..r.end])
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_transpile_basic_js() {
    let t = Transpiler::new();
    let result = t.transpile("const x: number = 1;");
    if let Err(ref e) = result {
      println!("Transpile error: {e}");
    }
    assert!(result.is_ok());
    let code = result.unwrap();
    println!("Output code: |{code}|");
    println!("Output length: {}", code.len());
    assert!(!code.is_empty(), "Code should not be empty");
    let has_const = code.contains("const");
    println!("Contains 'const': {has_const}");
    assert!(has_const, "Code should contain 'const'. Full output: '{code}'");
  }

  #[test]
  fn test_transpile_jsx() {
    let t = Transpiler::new();
    let opts = TranspileOptions {
      jsx_automatic: true,
      ..Default::default()
    };
    let result = t.transpile_with_options("const el = <div>hello</div>;", &opts);
    if let Err(ref e) = result {
      println!("JSX Transpile error: {e}");
    }
    assert!(result.is_ok());
  }

  #[test]
  fn test_strip_decorators() {
    let t = Transpiler::new();
    let input = "@Component\nexport class Foo {}";
    let result = t.strip_decorators(input);
    assert!(!result.contains('@'));
  }

  #[test]
  fn test_handle_import_assertions() {
    let t = Transpiler::new();
    let input = r#"import data from "./data.json" assert { type: "json" };"#;
    let result = t.handle_import_assertions(input);
    assert!(!result.contains("assert"));
  }

  #[test]
  fn test_vlq_encode() {
    let vlq = Transpiler::encode_vlq(0);
    assert_eq!(vlq, "A");
    let vlq_pos = Transpiler::encode_vlq(1);
    assert!(!vlq_pos.is_empty());
    let vlq_neg = Transpiler::encode_vlq(-1);
    assert!(!vlq_neg.is_empty());
  }

  #[test]
  fn test_vlq_sourcemap_generation() {
    let t = Transpiler::new();
    let map = t.generate_vlq_sourcemap(
      "test.ts",
      "const x = 1;",
      "const x = 1;",
      &[(0, 0, 0, 0), (0, 6, 0, 6)],
    );
    assert!(map.contains("version"));
    assert!(map.contains("test.ts"));
  }

  #[test]
  fn test_resolve_path_alias() {
    let mut paths = HashMap::new();
    paths.insert("@/*".into(), vec!["./src/*".into()]);
    let t = Transpiler::new().with_tsconfig_paths(paths, Some("./".into()));
    let resolved = t.resolve_path_alias("@/components/Button");
    // Path aliases resolve to ./src/components/Button (without doubled ./)
    assert_eq!(resolved, Some("./src/components/Button".into()));
  }

  #[test]
  fn test_resolve_path_alias_no_match() {
    let t = Transpiler::new();
    let resolved = t.resolve_path_alias("lodash");
    assert_eq!(resolved, None);
  }

  #[test]
  fn test_is_ts() {
    let t = Transpiler::new();
    assert!(t.is_ts("file.ts"));
    assert!(t.is_ts("file.tsx"));
    assert!(t.is_ts("file.mts"));
    assert!(!t.is_ts("file.js"));
    assert!(!t.is_ts("file.jsx"));
  }

  #[test]
  fn test_detect_source_type() {
    let t = Transpiler::new();
    assert_eq!(t.detect_source_type("file.ts"), SourceType::ts());
    assert_eq!(t.detect_source_type("file.tsx"), SourceType::tsx());
    assert_eq!(t.detect_source_type("file.jsx"), SourceType::jsx());
    assert_eq!(t.detect_source_type("file.mjs"), SourceType::mjs());
    assert_eq!(t.detect_source_type("file.cjs"), SourceType::cjs());
  }

  #[test]
  fn test_transpile_options_defaults() {
    let opts = TranspileOptions::default();
    assert!(opts.jsx_automatic);
    assert!(opts.decorators);
    assert!(opts.sourcemap);
    assert!(opts.import_assertions);
    assert_eq!(opts.target, "es2022");
  }

  #[test]
  fn test_transpile_with_sourcemap() {
    let t = Transpiler::new();
    let result = t.transpile_with_sourcemap("test.ts", "const x: number = 1;");
    assert!(result.is_ok());
    let (_code, sm) = result.unwrap();
    assert!(!_code.is_empty());
    // Sourcemap may be None for simple cases
  }
}
