use rquickjs::{Context, Runtime as QuickRuntime};

pub struct QuickJSRuntime {
    runtime: QuickRuntime,
    context: Context,
}

impl QuickJSRuntime {
    pub fn new() -> Result<Self, String> {
        let runtime = QuickRuntime::new().map_err(|e| e.to_string())?;
        let context = Context::full(&runtime).map_err(|e| e.to_string())?;
        Ok(Self { runtime, context })
    }

    pub fn eval(&self, code: &str) -> Result<String, String> {
        self.context.with(|ctx| {
            let value: rquickjs::Value = ctx.eval(code).map_err(|e| format!("QuickJS eval error: {}", e))?;
            if value.is_string() {
                value.get::<String>().map_err(|e| e.to_string())
            } else if value.is_number() {
                if let Ok(n) = value.get::<f64>() {
                    if n.fract() == 0.0 && n.is_finite() {
                        Ok((n as i64).to_string())
                    } else {
                        Ok(n.to_string())
                    }
                } else {
                    Ok(value.get::<i64>().map(|n| n.to_string()).map_err(|e| e.to_string())?)
                }
            } else if value.is_bool() {
                value.get::<bool>().map(|b| b.to_string()).map_err(|e| e.to_string())
            } else if value.is_null() || value.is_undefined() {
                Ok("null".to_string())
            } else if value.is_object() || value.is_array() {
                let json_str: String = rquickjs::function::stringify(ctx, value.clone())
                    .map_err(|e| format!("JSON stringify error: {}", e))?
                    .unwrap_or_default();
                Ok(json_str)
            } else {
                Ok(value.get::<String>().unwrap_or_default())
            }
        })
    }

    pub fn execute_script(&self, _filename: &str, source: &str) -> Result<String, String> {
        self.eval(source)
    }
}