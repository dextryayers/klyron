use std::path::Path;

use oxc::allocator::Allocator;
use oxc::codegen::Codegen;
use oxc::parser::Parser;
use oxc::span::SourceType;
use oxc::transformer::{TransformOptions, Transformer};

#[derive(Clone)]
pub struct Transpiler;

impl Transpiler {
  pub fn new() -> Self {
    Self
  }

  pub fn transpile(&self, code: &str) -> Result<String, String> {
    let allocator = Allocator::default();
    let source_type = SourceType::mjs();
    let ret = Parser::new(&allocator, code, source_type).parse();
    if !ret.diagnostics.is_empty() {
      let msg = format!("Parse errors ({} diagnostics)", ret.diagnostics.len());
      return Err(msg);
    }
    let mut program = ret.program;
    let scoping = oxc::semantic::Scoping::default();
    let _ = Transformer::new(&allocator, Path::new("."), &TransformOptions::default())
      .build_with_scoping(scoping, &mut program);
    let output = Codegen::new().build(&program);
    Ok(output.code)
  }

  pub fn is_ts(&self, specifier: &str) -> bool {
    specifier.ends_with(".ts") || specifier.ends_with(".tsx") || specifier.ends_with(".mts")
  }

  pub fn transpile_with_sourcemap(&self, code: &str) -> Result<(String, Option<String>), String> {
    let allocator = Allocator::default();
    let source_type = SourceType::mjs();
    let ret = Parser::new(&allocator, code, source_type).parse();
    if !ret.diagnostics.is_empty() {
      let msg = format!("Parse errors ({} diagnostics)", ret.diagnostics.len());
      return Err(msg);
    }
    let mut program = ret.program;
    let scoping = oxc::semantic::Scoping::default();
    let _ = Transformer::new(&allocator, Path::new("."), &TransformOptions::default())
      .build_with_scoping(scoping, &mut program);
    let output = Codegen::new().build(&program);
    Ok((output.code, None))
  }
}

impl Default for Transpiler {
  fn default() -> Self {
    Self::new()
  }
}
