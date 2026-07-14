//! Quickjs isolate / context management

pub struct QuickjsIsolate {
    pub context_created: bool,
}

impl QuickjsIsolate {
    pub fn new() -> Self {
        Self { context_created: false }
    }

    pub fn create_context(&mut self) -> Result<(), crate::error::quickjsError> {
        self.context_created = true;
        Ok(())
    }

    pub fn destroy_context(&mut self) {
        self.context_created = false;
    }
}
