use std::path::Path;

use oxc::allocator::Allocator;
use oxc::codegen::Codegen;
use oxc::parser::Parser;
use oxc::span::SourceType;
use oxc::transformer::{TransformOptions, Transformer};

#[derive(Clone, Default)]
pub struct Transpiler;

impl Transpiler {
  pub fn new() -> Self {
    Self
  }

  pub fn transpile(&self, code: &str) -> Result<String, String> {
    self.transpile_with_sourcemap(code).map(|(c, _)| c)
  }

  pub fn is_ts(&self, specifier: &str) -> bool {
    specifier.ends_with(".ts") || specifier.ends_with(".tsx") || specifier.ends_with(".mts")
  }

  pub fn transpile_with_sourcemap(&self, code: &str) -> Result<(String, Option<String>), String> {
    let allocator = Allocator::default();
    let source_type = SourceType::mjs();
    let ret = Parser::new(&allocator, code, source_type).parse();
    if !ret.diagnostics.is_empty() {
      let errors: Vec<String> = ret.diagnostics.iter().map(|d| d.to_string()).collect();
      return Err(errors.join("\n"));
    }
    let mut program = ret.program;
    let scoping = oxc::semantic::Scoping::default();
    let transform_result = Transformer::new(&allocator, Path::new("."), &TransformOptions::default())
      .build_with_scoping(scoping, &mut program);
    if !transform_result.diagnostics.is_empty() {
      let errors: Vec<String> = transform_result.diagnostics.iter().map(|d| d.to_string()).collect();
      return Err(errors.join("\n"));
    }
    let output = Codegen::new().build(&program);
    Ok((output.code, None))
  }
}
