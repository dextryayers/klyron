use std::path::PathBuf;

use oxc::allocator::Allocator;
use oxc::codegen::{Codegen, CodegenOptions};
use oxc::parser::Parser;
use oxc::span::SourceType;

#[derive(Clone, Default)]
pub struct Transpiler;

impl Transpiler {
  pub fn new() -> Self {
    Self
  }

  pub fn transpile(&self, code: &str) -> Result<String, String> {
    self.transpile_with_sourcemap("<input>", code).map(|(c, _)| c)
  }

  pub fn transpile_with_sourcemap(&self, filename: &str, code: &str) -> Result<(String, Option<String>), String> {
    let allocator = Allocator::default();
    let source_type = self.detect_source_type(filename);
    let ret = Parser::new(&allocator, code, source_type).parse();

    if !ret.diagnostics.is_empty() {
      let errors: Vec<String> = ret.diagnostics.iter().map(|d| d.to_string()).collect();
      return Err(format!("Parse error in {filename}:\n{}", errors.join("\n")));
    }

    let program = ret.program;

    let output = Codegen::new()
      .with_options(CodegenOptions {
        source_map_path: Some(PathBuf::from(filename)),
        ..Default::default()
      })
      .build(&program);

    let sm = output.map.and_then(|m| {
      let json = m.to_json_string();
      if json == "{}" { None } else { Some(json) }
    });
    Ok((output.code, sm))
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
    } else {
      // For <eval> and unknown extensions, try TS first (superset of JS)
      // Only use TS if the code contains TypeScript syntax
      if filename == "<eval>" || filename.contains("<eval") {
        SourceType::ts()
      } else {
        SourceType::mjs()
      }
    }
  }
}
