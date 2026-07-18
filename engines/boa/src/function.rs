use boa_engine::{Context, NativeFunction, js_string};
use boa_engine::property::Attribute;

pub struct FunctionRegistrar;

impl FunctionRegistrar {
    pub fn register_fn(context: &mut Context, name: &str, func: NativeFunction) -> Result<(), crate::BoaError> {
        context.register_global_builtin_callable(
            js_string!(name),
            0usize,
            func,
        ).map_err(|e| crate::BoaError::from_js_error(&e))
    }

    pub fn register_fn_with_arity(context: &mut Context, name: &str, arity: usize, func: NativeFunction) -> Result<(), crate::BoaError> {
        context.register_global_builtin_callable(
            js_string!(name),
            arity,
            func,
        ).map_err(|e| crate::BoaError::from_js_error(&e))
    }

    pub fn register_object_methods(
        context: &mut Context,
        obj_name: &str,
        methods: &[(String, NativeFunction, usize)],
    ) -> Result<(), crate::BoaError> {
        let mut builder = boa_engine::object::ObjectInitializer::new(context);
        for (name, func, arity) in methods {
            builder.function(func.clone(), js_string!(name.as_str()), *arity);
        }
        let obj = builder.build();
        context.register_global_property(
            js_string!(obj_name),
            obj,
            Attribute::WRITABLE | Attribute::CONFIGURABLE | Attribute::ENUMERABLE,
        ).map_err(|e| crate::BoaError::from_js_error(&e))?;
        Ok(())
    }
}
