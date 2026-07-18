use boa_engine::{Context, JsValue, js_string, NativeFunction};

pub struct ArrayOps;

impl ArrayOps {
    pub fn new_array(context: &mut Context) -> boa_engine::object::builtins::JsArray {
        boa_engine::object::builtins::JsArray::new(context)
    }

    pub fn from_values(values: &[JsValue], context: &mut Context) -> boa_engine::object::builtins::JsArray {
        let arr = boa_engine::object::builtins::JsArray::new(context);
        for (i, v) in values.iter().enumerate() {
            let _ = arr.set(i as u32, v.clone(), false, context);
        }
        arr
    }

    pub fn to_vec(arr: &boa_engine::object::builtins::JsArray, context: &mut Context) -> Vec<JsValue> {
        let len = arr.length(context).unwrap_or(0) as usize;
        let mut vec = Vec::with_capacity(len);
        for i in 0..len {
            if let Ok(val) = arr.get(i as u32, context) {
                vec.push(val);
            }
        }
        vec
    }

    pub fn register(context: &mut Context) -> Result<(), crate::BoaError> {
        let mut builder = boa_engine::object::ObjectInitializer::new(context);
        builder.function(
            NativeFunction::from_fn_ptr(|_this, args, ctx| {
                let _len = args.first().and_then(|v| v.as_number()).unwrap_or(0.0) as u32;
                let arr = boa_engine::object::builtins::JsArray::new(ctx);
                let _ = arr.length(ctx);
                Ok(JsValue::from(arr))
            }),
            js_string!("ofSize"),
            1usize,
        );
        let obj = builder.build();
        context.register_global_property(
            js_string!("ArrayOps"),
            obj,
            boa_engine::property::Attribute::WRITABLE | boa_engine::property::Attribute::CONFIGURABLE | boa_engine::property::Attribute::ENUMERABLE,
        ).map_err(|e| crate::BoaError::from_js_error(&e))
    }
}
