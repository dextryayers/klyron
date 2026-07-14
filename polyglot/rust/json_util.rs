//! JSON parsing and manipulation utilities.
//!
//! Provides a full JSON parser, stringifier, pretty-printer,
//! and a recursive merge for objects.

use crate::types::{JsonValue, KlyronError, Result};
use std::collections::BTreeMap;
use std::fmt::Write;

// ---------------------------------------------------------------------------
// Parser
// ---------------------------------------------------------------------------

struct Parser {
    chars: Vec<char>,
    pos: usize,
}

impl Parser {
    fn new(s: &str) -> Self {
        Parser {
            chars: s.chars().collect(),
            pos: 0,
        }
    }

    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    fn bump(&mut self) -> Option<char> {
        let c = self.chars.get(self.pos).copied();
        self.pos += 1;
        c
    }

    fn skip_ws(&mut self) {
        while let Some(c) = self.peek() {
            if c.is_ascii_whitespace() {
                self.pos += 1;
            } else {
                break;
            }
        }
    }

    fn done(&self) -> bool {
        self.pos >= self.chars.len()
    }

    fn parse_value(&mut self) -> Result<JsonValue> {
        self.skip_ws();
        match self.peek() {
            None => Err(KlyronError::Parse("unexpected end of input".into())),
            Some('"') => self.parse_string().map(JsonValue::String),
            Some('t') | Some('f') => self.parse_bool(),
            Some('n') => self.parse_null(),
            Some('{') => self.parse_object(),
            Some('[') => self.parse_array(),
            Some(c) if c == '-' || c.is_ascii_digit() => self.parse_number(),
            Some(c) => Err(KlyronError::Parse(format!("unexpected char '{}'", c))),
        }
    }

    fn parse_string(&mut self) -> Result<String> {
        self.skip_ws();
        if self.bump() != Some('"') {
            return Err(KlyronError::Parse("expected '\"'".into()));
        }
        let mut buf = String::new();
        loop {
            match self.bump() {
                None => return Err(KlyronError::Parse("unterminated string".into())),
                Some('"') => return Ok(buf),
                Some('\\') => match self.bump() {
                    Some('"') => buf.push('"'),
                    Some('\\') => buf.push('\\'),
                    Some('/') => buf.push('/'),
                    Some('n') => buf.push('\n'),
                    Some('r') => buf.push('\r'),
                    Some('t') => buf.push('\t'),
                    Some('b') => buf.push('\u{0008}'),
                    Some('f') => buf.push('\u{000c}'),
                    Some('u') => {
                        let hex: String = (0..4).filter_map(|_| self.bump()).collect();
                        if hex.len() != 4 {
                            return Err(KlyronError::Parse("invalid unicode escape".into()));
                        }
                        let cp = u32::from_str_radix(&hex, 16)
                            .map_err(|_| KlyronError::Parse("bad unicode hex".into()))?;
                        buf.push(char::from_u32(cp).ok_or_else(|| {
                            KlyronError::Parse("invalid unicode code point".into())
                        })?);
                    }
                    Some(c) => {
                        buf.push('\\');
                        buf.push(c);
                    }
                    None => return Err(KlyronError::Parse("unterminated escape".into())),
                },
                Some(c) => buf.push(c),
            }
        }
    }

    fn parse_number(&mut self) -> Result<JsonValue> {
        self.skip_ws();
        let start = self.pos;
        if self.peek() == Some('-') {
            self.pos += 1;
        }
        while self.peek().map_or(false, |c| c.is_ascii_digit()) {
            self.pos += 1;
        }
        let mut is_float = false;
        if self.peek() == Some('.') {
            is_float = true;
            self.pos += 1;
            while self.peek().map_or(false, |c| c.is_ascii_digit()) {
                self.pos += 1;
            }
        }
        if matches!(self.peek(), Some('e') | Some('E')) {
            is_float = true;
            self.pos += 1;
            if matches!(self.peek(), Some('+') | Some('-')) {
                self.pos += 1;
            }
            while self.peek().map_or(false, |c| c.is_ascii_digit()) {
                self.pos += 1;
            }
        }
        let num: String = self.chars[start..self.pos].iter().collect();
        if is_float {
            num
                .parse::<f64>()
                .map(JsonValue::Number)
                .map_err(|e| KlyronError::Parse(format!("bad number '{}': {}", num, e)))
        } else {
            num
                .parse::<i64>()
                .map(|n| JsonValue::Number(n as f64))
                .or_else(|_| num.parse::<f64>().map(JsonValue::Number))
                .map_err(|e| KlyronError::Parse(format!("bad number '{}': {}", num, e)))
        }
    }

    fn parse_bool(&mut self) -> Result<JsonValue> {
        self.skip_ws();
        if self.chars[self.pos..].starts_with(&['t', 'r', 'u', 'e']) {
            self.pos += 4;
            Ok(JsonValue::Bool(true))
        } else if self.chars[self.pos..].starts_with(&['f', 'a', 'l', 's', 'e']) {
            self.pos += 5;
            Ok(JsonValue::Bool(false))
        } else {
            Err(KlyronError::Parse("invalid boolean".into()))
        }
    }

    fn parse_null(&mut self) -> Result<JsonValue> {
        self.skip_ws();
        if self.chars[self.pos..].starts_with(&['n', 'u', 'l', 'l']) {
            self.pos += 4;
            Ok(JsonValue::Null)
        } else {
            Err(KlyronError::Parse("invalid null".into()))
        }
    }

    fn parse_object(&mut self) -> Result<JsonValue> {
        self.skip_ws();
        if self.bump() != Some('{') {
            return Err(KlyronError::Parse("expected '{{'".into()));
        }
        let mut map = BTreeMap::new();
        loop {
            self.skip_ws();
            if self.peek() == Some('}') {
                self.pos += 1;
                return Ok(JsonValue::Object(map));
            }
            if !map.is_empty() {
                if self.bump() != Some(',') {
                    return Err(KlyronError::Parse("expected ',' or '}}'".into()));
                }
                self.skip_ws();
            }
            let key = self.parse_string()?;
            self.skip_ws();
            if self.bump() != Some(':') {
                return Err(KlyronError::Parse("expected ':'".into()));
            }
            let val = self.parse_value()?;
            map.insert(key, val);
        }
    }

    fn parse_array(&mut self) -> Result<JsonValue> {
        self.skip_ws();
        if self.bump() != Some('[') {
            return Err(KlyronError::Parse("expected '['".into()));
        }
        let mut arr = Vec::new();
        loop {
            self.skip_ws();
            if self.peek() == Some(']') {
                self.pos += 1;
                return Ok(JsonValue::Array(arr));
            }
            if !arr.is_empty() {
                if self.bump() != Some(',') {
                    return Err(KlyronError::Parse("expected ',' or ']'".into()));
                }
                self.skip_ws();
            }
            arr.push(self.parse_value()?);
        }
    }
}

/// Parse a JSON string into a `JsonValue`.
pub fn parse(input: &str) -> Result<JsonValue> {
    let mut p = Parser::new(input);
    let val = p.parse_value()?;
    p.skip_ws();
    if !p.done() {
        return Err(KlyronError::Parse("trailing characters after JSON value".into()));
    }
    Ok(val)
}

// ---------------------------------------------------------------------------
// Stringify
// ---------------------------------------------------------------------------

/// Convert a `JsonValue` into a compact JSON string (no extra whitespace).
pub fn stringify(value: &JsonValue) -> String {
    match value {
        JsonValue::Null => "null".into(),
        JsonValue::Bool(b) => b.to_string(),
        JsonValue::Number(n) => {
            if n.is_finite() && n.fract() == 0.0 && n.abs() <= (i64::MAX as f64) {
                format!("{}", *n as i64)
            } else {
                format!("{}", n)
            }
        }
        JsonValue::String(s) => {
            let mut out = String::with_capacity(s.len() + 2);
            out.push('"');
            for c in s.chars() {
                match c {
                    '"' => out.push_str("\\\""),
                    '\\' => out.push_str("\\\\"),
                    '\n' => out.push_str("\\n"),
                    '\r' => out.push_str("\\r"),
                    '\t' => out.push_str("\\t"),
                    '\u{0008}' => out.push_str("\\b"),
                    '\u{000c}' => out.push_str("\\f"),
                    c if (c as u32) < 0x20 => write!(out, "\\u{:04x}", c as u32).unwrap(),
                    c => out.push(c),
                }
            }
            out.push('"');
            out
        }
        JsonValue::Array(arr) => {
            let mut out = String::from('[');
            for (i, v) in arr.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                out.push_str(&stringify(v));
            }
            out.push(']');
            out
        }
        JsonValue::Object(map) => {
            let mut out = String::from('{');
            for (i, (k, v)) in map.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                out.push('"');
                out.push_str(k);
                out.push_str("\":");
                out.push_str(&stringify(v));
            }
            out.push('}');
            out
        }
    }
}

// ---------------------------------------------------------------------------
// Pretty-print
// ---------------------------------------------------------------------------

fn pretty_into(value: &JsonValue, indent: usize, out: &mut String) {
    match value {
        JsonValue::Null => out.push_str("null"),
        JsonValue::Bool(b) => {
            let _ = write!(out, "{}", b);
        }
        JsonValue::Number(n) => {
            if n.is_finite() && n.fract() == 0.0 && n.abs() <= (i64::MAX as f64) {
                let _ = write!(out, "{}", *n as i64);
            } else {
                let _ = write!(out, "{}", n);
            }
        }
        JsonValue::String(_) => {
            out.push_str(&stringify(value));
        }
        JsonValue::Array(arr) => {
            if arr.is_empty() {
                out.push_str("[]");
                return;
            }
            out.push_str("[\n");
            for (i, v) in arr.iter().enumerate() {
                if i > 0 {
                    out.push_str(",\n");
                }
                out.push_str(&"  ".repeat(indent + 1));
                pretty_into(v, indent + 1, out);
            }
            out.push('\n');
            out.push_str(&"  ".repeat(indent));
            out.push(']');
        }
        JsonValue::Object(map) => {
            if map.is_empty() {
                out.push_str("{}");
                return;
            }
            out.push_str("{\n");
            for (i, (k, v)) in map.iter().enumerate() {
                if i > 0 {
                    out.push_str(",\n");
                }
                out.push_str(&"  ".repeat(indent + 1));
                out.push('"');
                out.push_str(k);
                out.push('"');
                out.push_str(": ");
                pretty_into(v, indent + 1, out);
            }
            out.push('\n');
            out.push_str(&"  ".repeat(indent));
            out.push('}');
        }
    }
}

/// Pretty-print a `JsonValue` with 2-space indentation.
pub fn pretty_print(value: &JsonValue) -> String {
    let mut out = String::new();
    pretty_into(value, 0, &mut out);
    out
}

// ---------------------------------------------------------------------------
// Merge
// ---------------------------------------------------------------------------

/// Recursively merge two JSON values.
///
/// When both values are objects, keys from `b` overwrite keys from `a`,
/// and nested objects are merged recursively. For all other types,
/// the value from `b` wins.
pub fn merge(a: &JsonValue, b: &JsonValue) -> JsonValue {
    match (a, b) {
        (JsonValue::Object(a_map), JsonValue::Object(b_map)) => {
            let mut result = a_map.clone();
            for (key, val) in b_map {
                if let Some(existing) = result.remove(key) {
                    result.insert(key.clone(), merge(&existing, val));
                } else {
                    result.insert(key.clone(), val.clone());
                }
            }
            JsonValue::Object(result)
        }
        _ => b.clone(),
    }
}
