//! Jsc isolate / context management

pub struct JscIsolate {
    pub context_created: bool,
}

impl JscIsolate {
    pub fn new() -> Self {
        Self { context_created: false }
    }

    pub fn create_context(&mut self) -> Result<(), crate::error::jscError> {
        self.context_created = true;
        Ok(())
    }

    pub fn destroy_context(&mut self) {
        self.context_created = false;
    }
}
