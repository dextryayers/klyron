use crate::error::BoaError;
use crate::runtime::BoaRuntime;

pub struct BoaIsolate {
    pub runtime: BoaRuntime,
    pub name: String,
}

impl BoaIsolate {
    pub fn new() -> Self {
        Self { runtime: BoaRuntime::new(), name: "default".to_string() }
    }

    pub fn new_with_limits(
        stack_limit: Option<usize>,
        recursion_limit: Option<usize>,
        loop_limit: Option<u64>,
    ) -> Self {
        Self {
            runtime: BoaRuntime::new_with_limits(stack_limit, recursion_limit, loop_limit),
            name: "isolated".to_string(),
        }
    }

    pub fn with_name(mut self, name: &str) -> Self {
        self.name = name.to_string();
        self
    }

    pub fn eval(&mut self, code: &str) -> Result<String, BoaError> {
        self.runtime.eval(code)
    }

    pub fn dispose(self) {}
}

impl Default for BoaIsolate {
    fn default() -> Self { Self::new() }
}
