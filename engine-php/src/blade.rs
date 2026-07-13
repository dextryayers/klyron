//! Laravel Blade templating bridge.
//!
//! Renders Blade templates by delegating to PHP/Laravel's Blade compiler.
//! Communication happens via stdin/stdout JSON protocol: the PHP side
//! receives view name + data, renders the template, and returns HTML.

use crate::PhpEngine;

/// Renders Blade templates via the PHP engine
pub struct BladeRenderer {
  engine: Box<dyn PhpEngine>,
  view_paths: Vec<String>,
  cache_path: String,
}

impl BladeRenderer {
  pub fn new(engine: Box<dyn PhpEngine>, project_root: &str) -> Self {
    Self {
      engine,
      view_paths: vec![
        format!("{project_root}/resources/views"),
        format!("{project_root}/views"),
      ],
      cache_path: format!("{project_root}/storage/framework/views"),
    }
  }

  /// Render a Blade view with given data
  pub fn render(&self, view: &str, data: &std::collections::HashMap<String, serde_json::Value>) -> Result<String, String> {
    // 1. Set shared variables for data
    for (key, val) in data {
      self.engine.set_variable(key, val.clone())?;
    }

    // 2. Execute the Blade rendering PHP code
    let code = format!(
      r#"<?php
$__env = app(\Illuminate\View\Factory::class);
echo $__env->make('{view}', get_defined_vars())->render();
?>"#,
      view = view.replace("'", "\\'")
    );

    let result = self.engine.execute_code(&code)?;
    if result.exit_code != 0 {
      return Err(format!("Blade render error: {}", result.stderr));
    }
    Ok(result.stdout)
  }
}
