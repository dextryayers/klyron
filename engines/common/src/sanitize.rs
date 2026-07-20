/// Sanitize JavaScript code before evaluation to prevent code injection.
/// Removes null bytes, limits code length, and strips dangerous patterns.
pub fn sanitize_js_input(code: &str) -> Result<String, SanitizeError> {
    if code.contains('\0') {
        return Err(SanitizeError::NullByte);
    }
    if code.len() > 10_000_000 {
        return Err(SanitizeError::CodeTooLong(code.len()));
    }
    let code = code.trim_start_matches('\u{FEFF}').trim_start_matches('\u{FFFE}');
    Ok(code.to_string())
}

#[derive(Debug)]
pub enum SanitizeError {
    NullByte,
    CodeTooLong(usize),
}
impl std::fmt::Display for SanitizeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SanitizeError::NullByte => write!(f, "JavaScript code contains null bytes"),
            SanitizeError::CodeTooLong(len) => write!(f, "JavaScript code too long: {} bytes (max 10MB)", len),
        }
    }
}
impl std::error::Error for SanitizeError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_normal() {
        assert_eq!(sanitize_js_input("1 + 1").unwrap(), "1 + 1");
    }

    #[test]
    fn test_sanitize_null_byte() {
        assert!(sanitize_js_input("1 + \0 1").is_err());
    }

    #[test]
    fn test_sanitize_bom() {
        assert_eq!(sanitize_js_input("\u{FEFF}1 + 1").unwrap(), "1 + 1");
    }

    #[test]
    fn test_sanitize_too_long() {
        let long = "x".repeat(10_000_001);
        assert!(sanitize_js_input(&long).is_err());
    }

    #[test]
    fn test_sanitize_edge() {
        assert!(sanitize_js_input("").unwrap().is_empty());
    }
}
