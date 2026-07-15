use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ColorTheme {
    Light,
    Dark,
    HighContrast,
}

impl Default for ColorTheme {
    fn default() -> Self {
        if is_dark_terminal() {
            ColorTheme::Dark
        } else {
            ColorTheme::Light
        }
    }
}

fn is_dark_terminal() -> bool {
    std::env::var("COLORFGBG")
        .ok()
        .and_then(|v| v.split(';').last().map(|s| s.parse::<u8>().ok()))
        .flatten()
        .map(|bg| bg < 7)
        .unwrap_or(true)
}

#[derive(Clone, Copy)]
pub struct Style {
    pub fg: &'static str,
    pub bg: &'static str,
    pub bold: bool,
    pub dim: bool,
    pub italic: bool,
    pub underline: bool,
}

pub const STYLE_HEADER: Style = Style { fg: "\x1b[38;5;39m", bg: "", bold: true, dim: false, italic: false, underline: false };
pub const STYLE_SUCCESS: Style = Style { fg: "\x1b[38;5;46m", bg: "", bold: false, dim: false, italic: false, underline: false };
pub const STYLE_WARNING: Style = Style { fg: "\x1b[38;5;220m", bg: "", bold: false, dim: false, italic: false, underline: false };
pub const STYLE_ERROR: Style = Style { fg: "\x1b[38;5;196m", bg: "", bold: true, dim: false, italic: false, underline: false };
pub const STYLE_INFO: Style = Style { fg: "\x1b[38;5;45m", bg: "", bold: false, dim: false, italic: false, underline: false };
pub const STYLE_HIGHLIGHT: Style = Style { fg: "\x1b[38;5;213m", bg: "", bold: true, dim: false, italic: false, underline: false };
pub const STYLE_MUTED: Style = Style { fg: "\x1b[38;5;244m", bg: "", bold: false, dim: true, italic: false, underline: false };
pub const STYLE_LINK: Style = Style { fg: "\x1b[38;5;33m", bg: "", bold: false, dim: false, italic: false, underline: true };
pub const STYLE_CODE: Style = Style { fg: "\x1b[38;5;84m", bg: "", bold: false, dim: false, italic: false, underline: false };

pub static RESET: &str = "\x1b[0m";
pub static BOLD: &str = "\x1b[1m";
pub static DIM: &str = "\x1b[2m";
pub static ITALIC: &str = "\x1b[3m";
pub static UNDERLINE: &str = "\x1b[4m";

pub fn colorize(text: &str, style: Style) -> String {
    let mut out = String::new();
    if !style.fg.is_empty() { out.push_str(style.fg); }
    if !style.bg.is_empty() { out.push_str(style.bg); }
    if style.bold { out.push_str(BOLD); }
    if style.dim { out.push_str(DIM); }
    if style.italic { out.push_str(ITALIC); }
    if style.underline { out.push_str(UNDERLINE); }
    out.push_str(text);
    out.push_str(RESET);
    out
}

pub fn strip_ansi(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c == '\x1b' {
            if chars.next() == Some('[') {
                for ch in chars.by_ref() {
                    if ch == 'm' || ch == 'H' || ch == 'J' || ch == 'K' {
                        break;
                    }
                }
            }
        } else {
            out.push(c);
        }
    }
    out
}

pub fn gradient_text(text: &str, colors: &[&str]) -> String {
    let mut out = String::new();
    let len = text.len();
    for (i, c) in text.chars().enumerate() {
        let idx = if colors.is_empty() { 0 } else { (i * colors.len() / len.max(1)).min(colors.len() - 1) };
        out.push_str(colors[idx]);
        out.push(c);
        out.push_str(RESET);
    }
    out
}

pub fn rainbow(text: &str) -> String {
    let rainbow_colors = &["\x1b[31m", "\x1b[33m", "\x1b[32m", "\x1b[36m", "\x1b[34m", "\x1b[35m"];
    gradient_text(text, rainbow_colors)
}

pub fn banner() -> String {
    let c = "\x1b[38;5;39m";
    let r = RESET;
    format!(
        "{c}  _  __           {r}\n\
         {c} | |/ /_ _ _ _ _ {r}\n\
         {c} | ' <| '_| '_| {r}\n\
         {c} |_|\\_\\_| |_|   {r}\n\
         {c}  Universal Polyglot Runtime{r}\n"
    )
}

pub fn supports_color() -> bool {
    if std::env::var("NO_COLOR").is_ok() || std::env::var("CI").is_ok() && std::env::var("TERM").is_err() {
        return false;
    }
    if let Ok(term) = std::env::var("TERM") {
        if term == "dumb" || term == "xterm-mono" {
            return false;
        }
    }
    true
}

pub fn has_rainbow_flag() -> bool {
    std::env::args().any(|a| a == "--rainbow")
}

pub struct Color {
    pub code: &'static str,
}

impl Color {
    pub const RESET: Color = Color { code: "\x1b[0m" };
    pub const BOLD: Color = Color { code: "\x1b[1m" };
    pub const DIM: Color = Color { code: "\x1b[2m" };
    pub const RED: Color = Color { code: "\x1b[31m" };
    pub const GREEN: Color = Color { code: "\x1b[32m" };
    pub const YELLOW: Color = Color { code: "\x1b[33m" };
    pub const BLUE: Color = Color { code: "\x1b[34m" };
    pub const MAGENTA: Color = Color { code: "\x1b[35m" };
    pub const CYAN: Color = Color { code: "\x1b[36m" };
    pub const WHITE: Color = Color { code: "\x1b[37m" };
    pub const BRIGHT_RED: Color = Color { code: "\x1b[91m" };
    pub const BRIGHT_GREEN: Color = Color { code: "\x1b[92m" };
    pub const BRIGHT_YELLOW: Color = Color { code: "\x1b[93m" };
    pub const BRIGHT_BLUE: Color = Color { code: "\x1b[94m" };
    pub const BRIGHT_MAGENTA: Color = Color { code: "\x1b[95m" };
    pub const BRIGHT_CYAN: Color = Color { code: "\x1b[96m" };

    pub fn paint(&self, text: impl fmt::Display) -> String {
        if !supports_color() {
            return text.to_string();
        }
        format!("{}{}{}", self.code, text, Color::RESET.code)
    }

    pub fn bold(&self, text: impl fmt::Display) -> String {
        if !supports_color() {
            return text.to_string();
        }
        format!("\x1b[1m{}{}{}", self.code, text, Color::RESET.code)
    }
}

pub fn style_success(msg: impl fmt::Display) -> String {
    Color::GREEN.paint(format!("\u{2713} {}", msg))
}

pub fn style_error(msg: impl fmt::Display) -> String {
    Color::RED.paint(format!("\u{2717} {}", msg))
}

pub fn style_warning(msg: impl fmt::Display) -> String {
    Color::YELLOW.paint(format!("\u{26A0} {}", msg))
}

pub fn style_info(msg: impl fmt::Display) -> String {
    Color::CYAN.paint(format!("i {}", msg))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_ansi() {
        let colored = format!("\x1b[31mhello\x1b[0m");
        assert_eq!(strip_ansi(&colored), "hello");
    }

    #[test]
    fn test_colorize() {
        let result = colorize("test", STYLE_SUCCESS);
        assert!(result.contains("test"));
    }

    #[test]
    fn test_gradient_text() {
        let result = gradient_text("hello", &["\x1b[31m", "\x1b[32m"]);
        assert!(result.contains("hello"));
    }

    #[test]
    fn test_rainbow() {
        let result = rainbow("klyron");
        assert!(result.contains("klyron"));
    }

    #[test]
    fn test_banner() {
        let b = banner();
        assert!(b.contains("Polyglot"));
    }

    #[test]
    fn test_supports_color_env() {
        let _ = supports_color();
    }

    #[test]
    fn test_style_success() {
        let s = style_success("done");
        assert!(s.contains("done"));
    }

    #[test]
    fn test_paint_no_color() {
        std::env::set_var("NO_COLOR", "1");
        let c = Color::RED.paint("text");
        assert_eq!(c, "text");
        std::env::remove_var("NO_COLOR");
    }
}
