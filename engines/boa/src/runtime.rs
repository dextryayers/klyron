use crate::error::BoaError;
use boa_engine::{Context, Source, JsValue, NativeFunction, js_string};
use boa_engine::property::Attribute;

pub struct BoaRuntime {
    context: Context,
}

impl BoaRuntime {
    pub fn new() -> Self {
        match Self::try_new() {
            Ok(r) => r,
            Err(e) => panic!("BoaRuntime::new() failed: {e}"),
        }
    }

    pub fn try_new() -> Result<Self, BoaError> {
        let context = Context::builder()
            .build()
            .map_err(|e| BoaError::InitFailed(e.to_string()))?;
        Ok(Self { context })
    }

    pub fn new_with_limits(
        stack_limit: Option<usize>,
        recursion_limit: Option<usize>,
        loop_iteration_limit: Option<u64>,
    ) -> Self {
        match Self::try_new_with_limits(stack_limit, recursion_limit, loop_iteration_limit) {
            Ok(r) => r,
            Err(e) => panic!("BoaRuntime::new_with_limits() failed: {e}"),
        }
    }

    pub fn try_new_with_limits(
        stack_limit: Option<usize>,
        recursion_limit: Option<usize>,
        loop_iteration_limit: Option<u64>,
    ) -> Result<Self, BoaError> {
        let mut context = Context::builder()
            .build()
            .map_err(|e| BoaError::InitFailed(e.to_string()))?;
        {
            let limits = context.runtime_limits_mut();
            if let Some(v) = stack_limit { limits.set_stack_size_limit(v); }
            if let Some(v) = recursion_limit { limits.set_recursion_limit(v); }
            if let Some(v) = loop_iteration_limit { limits.set_loop_iteration_limit(v); }
        }
        Ok(Self { context })
    }

    pub fn eval(&mut self, code: &str) -> Result<String, BoaError> {
        let result = self.context.eval(Source::from_bytes(code))
            .map_err(|e| BoaError::from_js_error_with_context(&e))?;
        Ok(result.to_string(&mut self.context)
            .map(|s| s.to_std_string_escaped())
            .unwrap_or_default())
    }

    pub fn execute_script(&mut self, filename: &str, source: &str) -> Result<String, BoaError> {
        let _ = filename;
        self.eval(source)
    }

    pub fn execute_module(&mut self, _filename: &str, source: &str) -> Result<String, BoaError> {
        self.eval(source)
    }

    pub fn context_mut(&mut self) -> &mut Context { &mut self.context }
    pub fn context(&self) -> &Context { &self.context }

    pub fn register_global_function(
        &mut self,
        name: &str,
        func: NativeFunction,
    ) -> Result<(), BoaError> {
        self.context.register_global_builtin_callable(
            js_string!(name),
            0,
            func,
        ).map_err(|e| BoaError::from_js_error(&e))
    }

    pub fn register_global_value(
        &mut self,
        name: &str,
        value: JsValue,
    ) -> Result<(), BoaError> {
        self.context.register_global_property(
            js_string!(name),
            value,
            Attribute::WRITABLE | Attribute::CONFIGURABLE | Attribute::ENUMERABLE,
        ).map_err(|e| BoaError::from_js_error(&e))
    }

    pub fn global_object(&self) -> boa_engine::JsObject {
        self.context.global_object()
    }

    pub fn stack_trace(&self) -> String {
        let code = "(function() { try { throw new Error(); } catch(e) { return e.stack || String(e); } })()";
        let mut ctx = Context::builder().build().ok();
        if let Some(ref mut c) = ctx {
            let result = c.eval(Source::from_bytes(code));
            if let Ok(val) = result {
                let s = val.to_string(c).map(|s| s.to_std_string_escaped()).unwrap_or_default();
                return s;
            }
        }
        "  (no stack trace)".to_string()
    }
}

impl Default for BoaRuntime {
    fn default() -> Self { Self::new() }
}
