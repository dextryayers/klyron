//! Boa isolate / context management

pub struct BoaIsolate {
    pub context_created: bool,
}

impl BoaIsolate {
    pub fn new() -> Self {
        Self { context_created: false }
    }

    pub fn create_context(&mut self) -> Result<(), crate::error::boaError> {
        self.context_created = true;
        Ok(())
    }

    pub fn destroy_context(&mut self) {
        self.context_created = false;
    }
}
