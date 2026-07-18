use crate::error::BoaError;
use crate::runtime::BoaRuntime;
use boa_engine::{Context, JsValue, js_string};

pub struct ContextManager {
    runtime: BoaRuntime,
    sealed: bool,
}

impl ContextManager {
    pub fn new() -> Self {
        Self {
            runtime: BoaRuntime::new(),
            sealed: false,
        }
    }

    pub fn new_with_limits(
        stack_limit: Option<usize>,
        recursion_limit: Option<usize>,
        loop_limit: Option<u64>,
    ) -> Self {
        Self {
            runtime: BoaRuntime::new_with_limits(stack_limit, recursion_limit, loop_limit),
            sealed: false,
        }
    }

    pub fn context(&self) -> &Context {
        self.runtime.context()
    }

    pub fn context_mut(&mut self) -> &mut Context {
        self.runtime.context_mut()
    }

    pub fn runtime(&self) -> &BoaRuntime {
        &self.runtime
    }

    pub fn runtime_mut(&mut self) -> &mut BoaRuntime {
        &mut self.runtime
    }

    pub fn seal(&mut self) {
        self.sealed = true;
    }

    pub fn is_sealed(&self) -> bool {
        self.sealed
    }

    pub fn reset(&mut self) -> Result<(), BoaError> {
        let limits = {
            let ctx = self.runtime.context();
            let lim = ctx.runtime_limits();
            (lim.stack_size_limit(), lim.recursion_limit(), lim.loop_iteration_limit())
        };
        self.runtime = BoaRuntime::new_with_limits(
            Some(limits.0),
            Some(limits.1),
            Some(limits.2 as u64),
        );
        self.sealed = false;
        Ok(())
    }

    pub fn eval(&mut self, code: &str) -> Result<String, BoaError> {
        self.runtime.eval(code)
    }

    pub fn set_global(&mut self, name: &str, value: JsValue) -> Result<(), BoaError> {
        let global = self.runtime.context().global_object();
        global.set(js_string!(name), value, false, self.runtime.context_mut())
            .map_err(|e| BoaError::from_js_error(&e))?;
        Ok(())
    }

    pub fn get_global(&mut self, name: &str) -> Result<JsValue, BoaError> {
        self.runtime.context().global_object()
            .get(js_string!(name), self.runtime.context_mut())
            .map_err(|e| BoaError::from_js_error(&e))
    }

    pub fn has_global(&mut self, name: &str) -> bool {
        self.runtime.context().global_object()
            .get(js_string!(name), self.runtime.context_mut())
            .is_ok()
    }

    pub fn delete_global(&mut self, name: &str) -> Result<(), BoaError> {
        self.runtime.context().global_object()
            .delete_property_or_throw(js_string!(name), self.runtime.context_mut())
            .map_err(|e| BoaError::from_js_error(&e))?;
        Ok(())
    }

    pub fn register_function(&mut self, name: &str, func: boa_engine::NativeFunction) -> Result<(), BoaError> {
        self.runtime.register_global_function(name, func)
    }

    pub fn dispose(self) {
        drop(self.runtime);
    }
}

impl Default for ContextManager {
    fn default() -> Self { Self::new() }
}
