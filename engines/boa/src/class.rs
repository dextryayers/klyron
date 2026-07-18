use boa_engine::{Context, JsValue, JsResult, NativeFunction};

pub struct ClassBuilder {
    name: String,
    methods: Vec<(String, NativeFunction, usize)>,
    properties: Vec<(String, JsValue)>,
}

impl ClassBuilder {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            methods: Vec::new(),
            properties: Vec::new(),
        }
    }

    pub fn method(mut self, name: &str, func: NativeFunction, arity: usize) -> Self {
        self.methods.push((name.to_string(), func, arity));
        self
    }

    pub fn property(mut self, name: &str, value: JsValue) -> Self {
        self.properties.push((name.to_string(), value));
        self
    }

    pub fn build(self, context: &mut Context) -> Result<JsValue, crate::BoaError> {
        let mut code = format!("class {} {{", self.name);
        for (name, _func, _arity) in &self.methods {
            code.push_str(&format!(" {}(...args) {{ return __call_method(this, '{}', args); }}", name, name));
        }
        code.push_str(" }");
        let setup = format!(
            "var __call_method = (obj, name, args) => obj[name](...args);\n"
        );
        context.eval(boa_engine::Source::from_bytes(&setup))
            .map_err(|e| crate::BoaError::from_js_error(&e))?;
        let result = context.eval(boa_engine::Source::from_bytes(&code))
            .map_err(|e| crate::BoaError::from_js_error(&e))?;
        Ok(result)
    }
}

pub fn define_class(context: &mut Context, name: &str, body: &str) -> JsResult<JsValue> {
    let code = format!("(function() {{ {}; return {}; }})()", body, name);
    context.eval(boa_engine::Source::from_bytes(&code))
}
