use deno_core::Extension;

pub struct ExtensionRegistry {
  extensions: Vec<Extension>,
}

impl ExtensionRegistry {
  pub fn new() -> Self {
    Self { extensions: vec![] }
  }

  pub fn register(&mut self, ext: Extension) {
    self.extensions.push(ext);
  }

  pub fn all(&self) -> &[Extension] {
    &self.extensions
  }

  pub fn into_vec(self) -> Vec<Extension> {
    self.extensions
  }
}
