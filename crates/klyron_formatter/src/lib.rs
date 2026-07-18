pub mod config;
pub mod format;

pub use config::*;
pub use format::*;

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::{Path, PathBuf};

    fn test_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    }

    #[test]
    fn test_new() {
        let f = Formatter::new();
        let _ = f;
    }

    #[test]
    fn test_detect_rustfmt() {
        let dir = test_dir();
        assert_eq!(Formatter::detect(&dir), FormatBackend::Rustfmt);
    }

    #[test]
    fn test_detect_prettier_fallback() {
        let dir = Path::new("/tmp");
        assert_eq!(Formatter::detect(dir), FormatBackend::Prettier);
    }

    #[test]
    fn test_backend_name() {
        assert_eq!(FormatBackend::Prettier.name(), "Prettier");
        assert_eq!(FormatBackend::Rustfmt.name(), "rustfmt");
    }

    #[test]
    fn test_content_hash() {
        let h1 = Formatter::content_hash("hello");
        let h2 = Formatter::content_hash("hello");
        let h3 = Formatter::content_hash("world");
        assert_eq!(h1, h2);
        assert_ne!(h1, h3);
        assert_eq!(h1.len(), 64);
    }

    #[test]
    fn test_format_diff_same_content() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("Cargo.toml"), "[package]\nname = \"test\"\nversion = \"0.1.0\"\nedition = \"2021\"\n").unwrap();
        let test_file = dir.path().join("test.rs");
        std::fs::write(&test_file, "fn main() {}").unwrap();
        let config = FormatterConfig {
            write: false,
            incremental: false,
            use_cache: false,
            indent_size: 4,
            indent_style: IndentStyle::Spaces,
            line_width: 80,
        };
        let formatter = Formatter::with_config(config);
        if std::process::Command::new("rustfmt").arg("--version").output().is_err() {
            return;
        }
        let report = formatter.format_diff(dir.path()).unwrap();
        assert!(report.files_unchanged >= 1);
        assert_eq!(report.files_changed, 0);
    }

    #[test]
    fn test_format_report_struct() {
        let report = FormatReport {
            files_changed: 3,
            files_unchanged: 7,
            files_skipped: 0,
            diffs: vec![],
            output: "done".into(),
        };
        assert_eq!(report.files_changed, 3);
        assert_eq!(report.files_unchanged, 7);
    }

    #[test]
    fn test_config_defaults() {
        let cfg = FormatterConfig::default();
        assert_eq!(cfg.indent_size, 2);
        assert_eq!(cfg.indent_style, IndentStyle::Spaces);
        assert_eq!(cfg.line_width, 80);
    }

    #[test]
    fn test_set_indent() {
        let mut f = Formatter::new();
        f.set_indent(4, IndentStyle::Tabs);
        assert_eq!(f.config().indent_size, 4);
        assert_eq!(f.config().indent_style, IndentStyle::Tabs);
    }

    #[test]
    fn test_format_js_like_indent() {
        let f = Formatter::new();
        let input = "{\n\"a\": 1\n}";
        let result = f.format_content_inline(input, "json").unwrap();
        assert!(result.contains("  \"a\": 1"));
    }

    #[test]
    fn test_format_js_like_tabs() {
        let mut f = Formatter::new();
        f.set_indent(1, IndentStyle::Tabs);
        let input = "{\n\"a\": 1\n}";
        let result = f.format_content_inline(input, "json").unwrap();
        assert!(result.contains("\t\"a\": 1"));
    }

    #[test]
    fn test_backend_is_jsts() {
        assert!(FormatBackend::Prettier.is_jsts());
        assert!(FormatBackend::Biome.is_jsts());
        assert!(!FormatBackend::Rustfmt.is_jsts());
    }

    #[test]
    fn test_format_content_inline_unknown_language() {
        let f = Formatter::new();
        let input = "some text";
        let result = f.format_content_inline(input, "unknown").unwrap();
        assert_eq!(result, input);
    }
}
