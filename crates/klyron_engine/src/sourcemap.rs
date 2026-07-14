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

struct ParsedStackLine {
    file: String,
    line: u32,
    column: u32,
}
