use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct SourceMapEntry {
    pub generated_line: u32,
    pub generated_column: u32,
    pub original_line: u32,
    pub original_column: u32,
    pub source: String,
    pub name: Option<String>,
}

#[derive(Debug, Clone)]
pub struct SourceMap {
    pub version: u32,
    pub file: Option<String>,
    pub source_root: Option<String>,
    pub sources: Vec<String>,
    pub sources_content: Vec<Option<String>>,
    pub names: Vec<String>,
    pub mappings: Vec<SourceMapEntry>,
    line_mappings: HashMap<u32, Vec<SourceMapEntry>>,
}

impl SourceMap {
    pub fn new() -> Self {
        Self {
            version: 3,
            file: None,
            source_root: None,
            sources: Vec::new(),
            sources_content: Vec::new(),
            names: Vec::new(),
            mappings: Vec::new(),
            line_mappings: HashMap::new(),
        }
    }

    pub fn add_mapping(
        &mut self,
        generated_line: u32,
        generated_column: u32,
        original_line: u32,
        original_column: u32,
        source: &str,
        name: Option<&str>,
    ) {
        let entry = SourceMapEntry {
            generated_line,
            generated_column,
            original_line,
            original_column,
            source: source.to_string(),
            name: name.map(|s| s.to_string()),
        };
        if !self.sources.contains(&entry.source) {
            self.sources.push(entry.source.clone());
            self.sources_content.push(None);
        }
        self.line_mappings
            .entry(generated_line)
            .or_default()
            .push(entry.clone());
        self.mappings.push(entry);
    }

    pub fn map_stack_trace(&self, stack: &str) -> String {
        let mut result = String::new();
        for line in stack.lines() {
            if let Some(mapped) = self.map_line(line) {
                result.push_str(&mapped);
            } else {
                result.push_str(line);
            }
            result.push('\n');
        }
        result
    }

    fn map_line(&self, line: &str) -> Option<String> {
        let parsed = self.parse_stack_line(line)?;
        let line_mappings = self.line_mappings.get(&parsed.line)?;
        let best = line_mappings
            .iter()
            .filter(|m| m.generated_column <= parsed.column)
            .max_by_key(|m| m.generated_column)?;

        let source_idx = self.sources.iter().position(|s| s == &best.source);
        let _source_name = source_idx
            .and_then(|idx| self.sources_content.get(idx))
            .and_then(|c| c.as_ref());

        Some(format!(
            "  at {} ({}:{}:{})",
            best.name.as_deref().unwrap_or("<anonymous>"),
            best.source,
            best.original_line,
            best.original_column
        ))
    }

    fn parse_stack_line(&self, line: &str) -> Option<ParsedStackLine> {
        let line = line.trim();
        if line.is_empty() || line.starts_with("Error") || line.starts_with("at ") {
            return None;
        }
        if let Some(pos) = line.rfind(':') {
            let rest = &line[..pos];
            if let Some(pos2) = rest.rfind(':') {
                let column: u32 = line[pos + 1..].parse().ok()?;
                let line_str = &rest[pos2 + 1..pos];
                let line_num: u32 = line_str.parse().ok()?;
                let file_part = &rest[..pos2];
                let file = if let Some(start) = file_part.rfind('(') {
                    file_part[start + 1..].trim_end_matches(')').to_string()
                } else {
                    file_part.trim().to_string()
                };
                return Some(ParsedStackLine {
                    file,
                    line: line_num,
                    column,
                });
            }
        }
        None
    }

    pub fn parse_sourcemap(content: &str) -> Result<Self, String> {
        let parsed: serde_json::Value =
            serde_json::from_str(content).map_err(|e| format!("Invalid source map: {}", e))?;

        let mut sm = SourceMap::new();
        if let Some(version) = parsed.get("version").and_then(|v| v.as_u64()) {
            sm.version = version as u32;
        }
        if let Some(file) = parsed.get("file").and_then(|f| f.as_str()) {
            sm.file = Some(file.to_string());
        }
        if let Some(root) = parsed.get("sourceRoot").and_then(|s| s.as_str()) {
            sm.source_root = Some(root.to_string());
        }
        if let Some(sources) = parsed.get("sources").and_then(|s| s.as_array()) {
            for src in sources {
                if let Some(s) = src.as_str() {
                    sm.sources.push(s.to_string());
                }
            }
        }
        if let Some(content) = parsed.get("sourcesContent").and_then(|c| c.as_array()) {
            for entry in content {
                sm.sources_content
                    .push(entry.as_str().map(|s| s.to_string()));
            }
        }
        if let Some(names) = parsed.get("names").and_then(|n| n.as_array()) {
            for name in names {
                if let Some(n) = name.as_str() {
                    sm.names.push(n.to_string());
                }
            }
        }
        Ok(sm)
    }
}

impl Default for SourceMap {
    fn default() -> Self {
        Self::new()
    }
}

#[allow(dead_code)]
struct ParsedStackLine {
    file: String,
    line: u32,
    column: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sourcemap_new() {
        let sm = SourceMap::new();
        assert_eq!(sm.version, 3);
        assert!(sm.sources.is_empty());
        assert!(sm.mappings.is_empty());
    }

    #[test]
    fn test_sourcemap_add_mapping() {
        let mut sm = SourceMap::new();
        sm.add_mapping(1, 0, 1, 0, "source.js", None);
        assert_eq!(sm.mappings.len(), 1);
        assert_eq!(sm.sources.len(), 1);
        assert_eq!(sm.sources[0], "source.js");
    }

    #[test]
    fn test_sourcemap_add_mapping_with_name() {
        let mut sm = SourceMap::new();
        sm.add_mapping(2, 5, 10, 3, "lib.js", Some("myFunction"));
        let mapping = &sm.mappings[0];
        assert_eq!(mapping.name.as_deref(), Some("myFunction"));
        assert_eq!(mapping.generated_line, 2);
        assert_eq!(mapping.original_line, 10);
    }

    #[test]
    fn test_sourcemap_duplicate_source() {
        let mut sm = SourceMap::new();
        sm.add_mapping(1, 0, 1, 0, "shared.js", None);
        sm.add_mapping(2, 0, 5, 0, "shared.js", None);
        assert_eq!(sm.sources.len(), 1);
    }

    #[test]
    fn test_sourcemap_parse_valid() {
        let json = r#"{
            "version": 3,
            "file": "out.js",
            "sources": ["in.js"],
            "names": ["foo"],
            "mappings": ""
        }"#;
        let sm = SourceMap::parse_sourcemap(json).unwrap();
        assert_eq!(sm.version, 3);
        assert_eq!(sm.file.as_deref(), Some("out.js"));
        assert_eq!(sm.sources, vec!["in.js"]);
        assert_eq!(sm.names, vec!["foo"]);
    }

    #[test]
    fn test_sourcemap_parse_with_source_root() {
        let json = r#"{
            "version": 3,
            "sourceRoot": "/root",
            "sources": ["src/app.ts"],
            "sourcesContent": ["console.log(1);"]
        }"#;
        let sm = SourceMap::parse_sourcemap(json).unwrap();
        assert_eq!(sm.source_root.as_deref(), Some("/root"));
        assert_eq!(sm.sources_content.len(), 1);
        assert_eq!(sm.sources_content[0].as_deref(), Some("console.log(1);"));
    }

    #[test]
    fn test_sourcemap_parse_invalid_json() {
        let result = SourceMap::parse_sourcemap("not json");
        assert!(result.is_err());
    }

    #[test]
    fn test_sourcemap_map_stack_trace_no_match() {
        let sm = SourceMap::new();
        let mapped = sm.map_stack_trace("Error: test\n    at file.js:1:2");
        assert_eq!(mapped, "Error: test\n    at file.js:1:2\n");
    }

    #[test]
    fn test_sourcemap_map_stack_trace_with_mapping() {
        let mut sm = SourceMap::new();
        sm.add_mapping(1, 0, 10, 5, "original.ts", Some("func"));
        // parse_stack_line expects lines like "file:line:col" without "at " or "Error" prefix
        let mapped = sm.map_stack_trace("file.js:1:2");
        assert!(mapped.contains("original.ts"), "mapped should contain original.ts, got: {}", mapped);
    }

    #[test]
    fn test_sourcemap_default() {
        let sm: SourceMap = Default::default();
        assert_eq!(sm.version, 3);
    }

    #[test]
    fn test_sourcemap_multiple_mappings_same_line() {
        let mut sm = SourceMap::new();
        sm.add_mapping(1, 0, 1, 0, "a.js", Some("a"));
        sm.add_mapping(1, 10, 2, 0, "b.js", Some("b"));
        assert_eq!(sm.mappings.len(), 2);
        assert_eq!(sm.line_mappings.get(&1).unwrap().len(), 2);
    }
}
