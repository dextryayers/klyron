//! V8 isolate / context management

pub struct V8Isolate {
    pub context_created: bool,
}

impl V8Isolate {
    pub fn new() -> Self {
        Self { context_created: false }
    }

    pub fn create_context(&mut self) -> Result<(), crate::error::v8Error> {
        self.context_created = true;
        Ok(())
    }

    pub fn destroy_context(&mut self) {
        self.context_created = false;
    }
}
