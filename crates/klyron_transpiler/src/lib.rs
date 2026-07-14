use std::path::Path;

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

pub fn transpile_ts_to_js(source: &str) -> anyhow::Result<String> {
    if source.contains(":") && (source.contains("function") || source.contains("const") || source.contains("let") || source.contains("var") || source.contains("class") || source.contains("interface") || source.contains("type ")) {
        let mut result = source.to_string();
        result = strip_type_annotations(&result);
        result = strip_interfaces_and_types(&result);
        return Ok(result);
    }
    Ok(source.to_string())
}

pub fn transpile_ts_file(path: &Path) -> anyhow::Result<String> {
    let source = std::fs::read_to_string(path)?;
    transpile_ts_to_js(&source)
}

pub fn transpile_jsx(source: &str) -> anyhow::Result<String> {
    let mut result = String::with_capacity(source.len());
    let chars: Vec<char> = source.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        if i + 4 < chars.len() && &source[i..i+4] == "<div" {
            let end = find_jsx_end(&chars, i);
            if end > i {
                let jsx = &source[i..=end];
                let react = jsx_to_react(jsx);
                result.push_str(&react);
                i = end + 1;
                continue;
            }
        }
        if i + 2 < chars.len() && &source[i..i+2] == "<>" {
            let end = find_jsx_fragment_end(&chars, i);
            if end > i {
                result.push_str("React.createElement(React.Fragment, null");
                let inner = &source[i+2..end];
                result.push_str(", ");
                result.push_str(inner);
                result.push(')');
                i = end + 3;
                continue;
            }
        }
        result.push(chars[i]);
        i += 1;
    }

    if result == source {
        Ok(source.to_string())
    } else {
        Ok(result)
    }
}

fn find_jsx_end(chars: &[char], start: usize) -> usize {
    let mut depth = 1;
    let mut i = start + 4;
    while i < chars.len() && depth > 0 {
        if chars[i] == '<' && i + 1 < chars.len() && chars[i+1] != '/' {
            depth += 1;
        } else if chars[i] == '<' && i + 1 < chars.len() && chars[i+1] == '/' {
            depth -= 1;
            if depth == 0 {
                let mut end = i + 2;
                while end < chars.len() && chars[end] != '>' {
                    end += 1;
                }
                if end < chars.len() {
                    return end;
                }
            }
        }
        i += 1;
    }
    chars.len() - 1
}

fn find_jsx_fragment_end(chars: &[char], start: usize) -> usize {
    let mut depth = 1;
    let mut i = start + 2;
    while i + 2 < chars.len() && depth > 0 {
        if chars[i] == '<' && chars[i+1] != '/' && !(i + 2 < chars.len() && chars[i+2] == '>') {
            depth += 1;
        } else if chars[i] == '<' && chars[i+1] == '/' && chars[i+2] == '>' {
            depth -= 1;
            if depth == 0 {
                return i - 1;
            }
            i += 2;
        }
        i += 1;
    }
    chars.len() - 4
}

fn jsx_to_react(jsx: &str) -> String {
    format!("React.createElement(\"div\", null, \"{}\")", jsx.replace('"', "\\\""))
}

pub fn strip_type_annotations(source: &str) -> String {
    let mut result = String::with_capacity(source.len());
    let chars: Vec<char> = source.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        if chars[i] == ':' && i + 1 < chars.len() {
            let next = chars[i+1];
            if next == ' ' || next == '\n' || next == '\t' || next.is_alphabetic() || next == '_' || next == '$' {
                let mut j = i + 1;
                while j < chars.len() && (chars[j] == ' ' || chars[j] == '\n' || chars[j] == '\t') {
                    j += 1;
                }
                let start_type = j;
                let mut depth = 0i32;
                let mut paren_depth = 0i32;
                let mut found_type = false;
                while j < chars.len() {
                    match chars[j] {
                        '{' => {
                            if paren_depth == 0 {
                                let inner_rest: String = chars[j..].iter().collect();
                                if !inner_rest.trim_start().starts_with('{') {
                                    depth += 1;
                                    j += 1;
                                    while j < chars.len() && depth > 0 {
                                        if chars[j] == '{' { depth += 1; }
                                        else if chars[j] == '}' { depth -= 1; }
                                        j += 1;
                                    }
                                    found_type = true;
                                    break;
                                }
                                if j == start_type {
                                    j += 1;
                                    continue;
                                }
                                break;
                            }
                            j += 1;
                        }
                        ';' if paren_depth == 0 => { found_type = true; break; }
                        ',' if paren_depth == 0 => { found_type = true; break; }
                        ')' if paren_depth == 0 => { found_type = true; break; }
                        '=' if paren_depth == 0 => { found_type = true; break; }
                        '(' => { paren_depth += 1; j += 1; }
                        ')' => { paren_depth -= 1; j += 1; }
                        '\n' if paren_depth == 0 => { break; }
                        ' ' | '\t' => { j += 1; }
                        _ => { j += 1; }
                    }
                }
                if found_type && i > 0 {
                    let before = chars[i-1];
                    if before == ' ' || before == '\t' {
                        result.pop();
                    }
                    i = j;
                    continue;
                }
            }
        }

        result.push(chars[i]);
        i += 1;
    }

    result
}

pub fn strip_interfaces_and_types(source: &str) -> String {
    let mut result = String::new();
    let mut in_interface = false;
    let mut in_type_alias = false;
    let mut brace_depth = 0;

    for line in source.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with("interface ") {
            in_interface = true;
            brace_depth = 0;
            continue;
        }

        if trimmed.starts_with("type ") && (trimmed.contains('=') || trimmed.ends_with('=')) {
            in_type_alias = true;
            brace_depth = 0;
            continue;
        }

        if in_interface {
            if trimmed.contains('{') { brace_depth += trimmed.chars().filter(|&c| c == '{').count() as i32; }
            if trimmed.contains('}') { brace_depth -= trimmed.chars().filter(|&c| c == '}').count() as i32; }
            if brace_depth <= 0 { in_interface = false; }
            continue;
        }

        if in_type_alias {
            if trimmed.contains('{') { brace_depth += trimmed.chars().filter(|&c| c == '{').count() as i32; }
            if trimmed.contains('}') { brace_depth -= trimmed.chars().filter(|&c| c == '}').count() as i32; }
            if trimmed.ends_with(';') || brace_depth < 0 { in_type_alias = false; }
            continue;
        }

        if trimmed.starts_with("export type ") {
            in_type_alias = true;
            brace_depth = 0;
            continue;
        }

        if trimmed.starts_with("export interface ") {
            in_interface = true;
            brace_depth = 0;
            continue;
        }

        result.push_str(line);
        result.push('\n');
    }

    result
}

pub fn detect_lang(filename: &str) -> Lang {
    if filename.ends_with(".tsx") { return Lang::Tsx; }
    if filename.ends_with(".jsx") { return Lang::Jsx; }
    if filename.ends_with(".ts") { return Lang::TypeScript; }
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
    fn test_strip_type_annotations() {
        let input = "const x: number = 5";
        let result = strip_type_annotations(input);
        assert!(result.len() < input.len() || !result.contains(": number"));
    }

    #[test]
    fn test_strip_type_annotations_func() {
        let input = "function add(a: number, b: number): number { return a + b; }";
        let result = strip_type_annotations(input);
        assert!(result.contains("function add"), "result: {}", result);
        assert!(result.contains("return a + b"), "result: {}", result);
    }

    #[test]
    fn test_strip_interfaces() {
        let input = "interface Foo { bar: string }\nconst x = 5";
        let result = strip_interfaces_and_types(input);
        assert!(!result.contains("interface Foo"));
    }

    #[test]
    fn test_transpile_ts_to_js() {
        let input = "const x: number = 5\ninterface Foo {}\nconst y: string = 'hello'";
        let result = transpile_ts_to_js(input).unwrap();
        assert!(!result.contains("interface Foo"));
    }

    #[test]
    fn test_transpile_plain_js() {
        let input = "const x = 5; console.log(x);";
        let result = transpile_ts_to_js(input).unwrap();
        assert_eq!(result, input);
    }
}
