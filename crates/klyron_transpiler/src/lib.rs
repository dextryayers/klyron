pub mod polyfill;
pub mod transform;

pub use transform::*;

pub use polyfill::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Lang {
    TypeScript,
    Jsx,
    Tsx,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Target {
    EsNext,
    Es2022,
    Es2021,
    Es2020,
    Es5,
}

pub struct TranspileOptions {
    pub lang: Lang,
    pub target: Target,
    pub minify: bool,
    pub sourcemap: bool,
}

pub fn transpile_js(source: &str, options: &TranspileOptions) -> anyhow::Result<String> {
    let _ = options;
    Ok(source.to_string())
}

pub fn transpile_ts_file(path: &std::path::Path) -> anyhow::Result<String> {
    let source = std::fs::read_to_string(path)?;
    crate::transform::transpile_ts_to_js(&source)
}

pub fn detect_lang(filename: &str) -> Lang {
    if filename.ends_with(".tsx") {
        return Lang::Tsx;
    }
    if filename.ends_with(".jsx") {
        return Lang::Jsx;
    }
    if filename.ends_with(".ts") {
        return Lang::TypeScript;
    }
    Lang::TypeScript
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_lang() {
        assert_eq!(detect_lang("file.ts"), Lang::TypeScript);
        assert_eq!(detect_lang("file.tsx"), Lang::Tsx);
        assert_eq!(detect_lang("file.jsx"), Lang::Jsx);
    }

    #[test]
    fn test_transpile_plain_js() {
        let input = "const x = 5; console.log(x);";
        let result = transpile_ts_to_js(input).unwrap();
        assert_eq!(result, input);
    }
}
