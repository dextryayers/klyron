use deno_core::{extension, op2, Extension};

extension!(
  klyron_html,
  ops = [op_html_escape, op_html_unescape, op_html_strip_tags],
  esm_entry_point = "ext:klyron_html/html.js",
  esm = [dir "js", "html.js"],
);

pub fn init() -> Extension {
  klyron_html::init()
}

fn op_html_escape_impl(text: String) -> String {
  text.replace('&', "&amp;")
    .replace('<', "&lt;")
    .replace('>', "&gt;")
    .replace('"', "&quot;")
    .replace('\'', "&#x27;")
}

#[op2]
#[string]
fn op_html_escape(#[string] text: String) -> String {
  op_html_escape_impl(text)
}

fn op_html_unescape_impl(text: String) -> String {
  text.replace("&amp;", "&")
    .replace("&lt;", "<")
    .replace("&gt;", ">")
    .replace("&quot;", "\"")
    .replace("&#x27;", "'")
    .replace("&#39;", "'")
}

#[op2]
#[string]
fn op_html_unescape(#[string] text: String) -> String {
  op_html_unescape_impl(text)
}

fn op_html_strip_tags_impl(html: String) -> String {
  let mut out = String::with_capacity(html.len());
  let mut in_tag = false;
  for c in html.chars() {
    match c {
      '<' => in_tag = true,
      '>' => in_tag = false,
      _ if !in_tag => out.push(c),
      _ => {}
    }
  }
  out
}

#[op2]
#[string]
fn op_html_strip_tags(#[string] html: String) -> String {
  op_html_strip_tags_impl(html)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init_returns_extension() {
        let ext = init();
        assert_eq!(ext.name, "klyron_html");
    }

    #[test]
    fn test_html_escape_basic() {
        let result = op_html_escape_impl("<script>alert('xss')</script>".to_string());
        assert_eq!(result, "&lt;script&gt;alert(&#x27;xss&#x27;)&lt;/script&gt;");
    }

    #[test]
    fn test_html_escape_ampersand() {
        let result = op_html_escape_impl("AT&T".to_string());
        assert_eq!(result, "AT&amp;T");
    }

    #[test]
    fn test_html_escape_quotes() {
        let result = op_html_escape_impl(r#"He said "hello""#.to_string());
        assert_eq!(result, "He said &quot;hello&quot;");
    }

    #[test]
    fn test_html_escape_empty() {
        let result = op_html_escape_impl("".to_string());
        assert_eq!(result, "");
    }

    #[test]
    fn test_html_escape_no_special() {
        let result = op_html_escape_impl("plain text".to_string());
        assert_eq!(result, "plain text");
    }

    #[test]
    fn test_html_unescape_basic() {
        let result = op_html_unescape_impl("&lt;div&gt;".to_string());
        assert_eq!(result, "<div>");
    }

    #[test]
    fn test_html_unescape_ampersand() {
        let result = op_html_unescape_impl("AT&amp;T".to_string());
        assert_eq!(result, "AT&T");
    }

    #[test]
    fn test_html_unescape_quotes() {
        let result = op_html_unescape_impl("&quot;hello&quot;".to_string());
        assert_eq!(result, "\"hello\"");
    }

    #[test]
    fn test_html_unescape_apos() {
        let result = op_html_unescape_impl("&#x27;test&#39;".to_string());
        assert_eq!(result, "'test'");
    }

    #[test]
    fn test_html_unescape_empty() {
        let result = op_html_unescape_impl("".to_string());
        assert_eq!(result, "");
    }

    #[test]
    fn test_html_strip_tags() {
        let result = op_html_strip_tags_impl("<h1>Title</h1><p>Paragraph</p>".to_string());
        assert_eq!(result, "TitleParagraph");
    }

    #[test]
    fn test_html_strip_tags_nested() {
        let result = op_html_strip_tags_impl("<div><span>text</span></div>".to_string());
        assert_eq!(result, "text");
    }

    #[test]
    fn test_html_strip_tags_no_tags() {
        let result = op_html_strip_tags_impl("plain text".to_string());
        assert_eq!(result, "plain text");
    }

    #[test]
    fn test_html_strip_tags_empty() {
        let result = op_html_strip_tags_impl("".to_string());
        assert_eq!(result, "");
    }

    #[test]
    fn test_html_strip_tags_self_closing() {
        let result = op_html_strip_tags_impl("Hello<br/>World".to_string());
        assert_eq!(result, "HelloWorld");
    }

    #[test]
    fn test_html_roundtrip() {
        let original = "a < b && b > c";
        let escaped = op_html_escape_impl(original.to_string());
        let unescaped = op_html_unescape_impl(escaped);
        assert_eq!(original, unescaped);
    }
}
