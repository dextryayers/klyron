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

#[op2]
#[string]
fn op_html_escape(#[string] text: String) -> String {
  text.replace('&', "&amp;")
    .replace('<', "&lt;")
    .replace('>', "&gt;")
    .replace('"', "&quot;")
    .replace('\'', "&#x27;")
}

#[op2]
#[string]
fn op_html_unescape(#[string] text: String) -> String {
  text.replace("&amp;", "&")
    .replace("&lt;", "<")
    .replace("&gt;", ">")
    .replace("&quot;", "\"")
    .replace("&#x27;", "'")
    .replace("&#39;", "'")
}

#[op2]
#[string]
fn op_html_strip_tags(#[string] html: String) -> String {
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
