use boa_engine::{Context, JsValue, NativeFunction, js_string};
use boa_engine::property::Attribute;

pub struct MathExtensions;

impl MathExtensions {
    pub fn clamp(value: f64, min: f64, max: f64) -> f64 {
        value.max(min).min(max)
    }

    pub fn lerp(a: f64, b: f64, t: f64) -> f64 {
        a + (b - a) * t
    }

    pub fn map_range(value: f64, in_min: f64, in_max: f64, out_min: f64, out_max: f64) -> f64 {
        if (in_max - in_min).abs() < f64::EPSILON { return out_min; }
        out_min + (value - in_min) * (out_max - out_min) / (in_max - in_min)
    }

    pub fn degrees_to_radians(degrees: f64) -> f64 {
        degrees * std::f64::consts::PI / 180.0
    }

    pub fn radians_to_degrees(radians: f64) -> f64 {
        radians * 180.0 / std::f64::consts::PI
    }

    pub fn gcd(mut a: i64, mut b: i64) -> i64 {
        while b != 0 { let t = b; b = a % b; a = t; }
        a.abs()
    }

    pub fn lcm(a: i64, b: i64) -> i64 {
        if a == 0 || b == 0 { return 0; }
        (a.abs() / Self::gcd(a, b)) * b.abs()
    }

    pub fn register(context: &mut Context) -> Result<(), crate::BoaError> {
        let mut builder = boa_engine::object::ObjectInitializer::new(context);
        builder.function(
            NativeFunction::from_fn_ptr(|_this, args, _ctx| {
                let val = args.first().and_then(|v| v.as_number()).unwrap_or(0.0);
                let min = args.get(1).and_then(|v| v.as_number()).unwrap_or(0.0);
                let max = args.get(2).and_then(|v| v.as_number()).unwrap_or(1.0);
                Ok(JsValue::from(Self::clamp(val, min, max)))
            }), js_string!("clamp"), 3usize,
        ).function(
            NativeFunction::from_fn_ptr(|_this, args, _ctx| {
                let a = args.first().and_then(|v| v.as_number()).unwrap_or(0.0);
                let b = args.get(1).and_then(|v| v.as_number()).unwrap_or(1.0);
                let t = args.get(2).and_then(|v| v.as_number()).unwrap_or(0.5);
                Ok(JsValue::from(Self::lerp(a, b, t)))
            }), js_string!("lerp"), 3usize,
        ).function(
            NativeFunction::from_fn_ptr(|_this, args, _ctx| {
                let val = args.first().and_then(|v| v.as_number()).unwrap_or(0.0);
                let in_min = args.get(1).and_then(|v| v.as_number()).unwrap_or(0.0);
                let in_max = args.get(2).and_then(|v| v.as_number()).unwrap_or(1.0);
                let out_min = args.get(3).and_then(|v| v.as_number()).unwrap_or(0.0);
                let out_max = args.get(4).and_then(|v| v.as_number()).unwrap_or(1.0);
                Ok(JsValue::from(Self::map_range(val, in_min, in_max, out_min, out_max)))
            }), js_string!("mapRange"), 5usize,
        ).function(
            NativeFunction::from_fn_ptr(|_this, args, _ctx| {
                let deg = args.first().and_then(|v| v.as_number()).unwrap_or(0.0);
                Ok(JsValue::from(Self::degrees_to_radians(deg)))
            }), js_string!("degToRad"), 1usize,
        ).function(
            NativeFunction::from_fn_ptr(|_this, args, _ctx| {
                let rad = args.first().and_then(|v| v.as_number()).unwrap_or(0.0);
                Ok(JsValue::from(Self::radians_to_degrees(rad)))
            }), js_string!("radToDeg"), 1usize,
        ).function(
            NativeFunction::from_fn_ptr(|_this, args, _ctx| {
                let a = args.first().and_then(|v| v.as_number()).unwrap_or(0.0) as i64;
                let b = args.get(1).and_then(|v| v.as_number()).unwrap_or(0.0) as i64;
                Ok(JsValue::from(Self::gcd(a, b) as f64))
            }), js_string!("gcd"), 2usize,
        ).function(
            NativeFunction::from_fn_ptr(|_this, args, _ctx| {
                let a = args.first().and_then(|v| v.as_number()).unwrap_or(0.0) as i64;
                let b = args.get(1).and_then(|v| v.as_number()).unwrap_or(0.0) as i64;
                Ok(JsValue::from(Self::lcm(a, b) as f64))
            }), js_string!("lcm"), 2usize,
        );
        let obj = builder.build();
        context.register_global_property(
            js_string!("MathX"),
            obj,
            Attribute::WRITABLE | Attribute::CONFIGURABLE | Attribute::ENUMERABLE,
        ).map_err(|e| crate::BoaError::from_js_error(&e))
    }
}
