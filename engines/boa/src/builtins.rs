use crate::BoaError;
use boa_engine::{Context, JsValue, js_string, NativeFunction, JsString};
use boa_engine::property::Attribute;

pub struct Builtins;

impl Builtins {
    pub fn register_all(context: &mut Context) -> Result<(), BoaError> {
        Self::register_klyron_version(context)?;
        Self::register_global_helpers(context)?;
        Ok(())
    }

    fn register_klyron_version(context: &mut Context) -> Result<(), BoaError> {
        context.register_global_property(
            js_string!("__klyron_version"),
            JsValue::from(JsString::from(crate::VERSION)),
            Attribute::READONLY | Attribute::CONFIGURABLE,
        ).map_err(|e| BoaError::from_js_error(&e))
    }

    fn register_global_helpers(context: &mut Context) -> Result<(), BoaError> {
        let mut builder = boa_engine::object::ObjectInitializer::new(context);
        builder.function(
            NativeFunction::from_fn_ptr(|_this, args, ctx| {
                let val = args.first().map(|v| {
                    v.to_string(ctx)
                        .map(|s| s.to_std_string_escaped())
                        .unwrap_or_default()
                }).unwrap_or_default();
                println!("{}", val);
                Ok(boa_engine::JsValue::undefined())
            }),
            js_string!("print"),
            1usize,
        ).function(
            NativeFunction::from_fn_ptr(|_this, args, ctx| {
                let val = args.first().map(|v| {
                    v.to_string(ctx)
                        .map(|s| s.to_std_string_escaped())
                        .unwrap_or_default()
                }).unwrap_or_default();
                eprintln!("{}", val);
                Ok(boa_engine::JsValue::undefined())
            }),
            js_string!("printErr"),
            1usize,
        ).function(
            NativeFunction::from_fn_ptr(|_this, args, _ctx| {
                if let Some(code) = args.first().and_then(|v| v.as_number()) {
                    std::process::exit(code as i32);
                }
                std::process::exit(0);
            }),
            js_string!("exit"),
            1usize,
        );
        let obj = builder.build();

        context.register_global_property(
            js_string!("Klyron"),
            obj,
            Attribute::WRITABLE | Attribute::CONFIGURABLE | Attribute::ENUMERABLE,
        ).map_err(|e| BoaError::from_js_error(&e))?;

        Ok(())
    }
}
