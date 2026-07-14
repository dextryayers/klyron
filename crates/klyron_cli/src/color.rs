use std::fmt;
use std::time::Instant;

#[derive(Clone, Copy)]
pub struct Color {
    pub code: &'static str,
}

impl Color {
    pub const RESET: Color = Color { code: "\x1b[0m" };
    pub const BOLD: Color = Color { code: "\x1b[1m" };
    pub const DIM: Color = Color { code: "\x1b[2m" };
    pub const ITALIC: Color = Color { code: "\x1b[3m" };
    pub const UNDERLINE: Color = Color { code: "\x1b[4m" };

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
        format!("{}{}{}", self.code, text, Color::RESET.code)
    }

    pub fn bold(&self, text: impl fmt::Display) -> String {
        format!("\x1b[1m{}{}{}", self.code, text, Color::RESET.code)
    }

    pub fn paint_stderr(&self, text: impl fmt::Display) {
        let _ = Self::write_stderr(&format!("{}{}{}", self.code, text, Color::RESET.code));
    }

    fn write_stderr(msg: &str) -> std::io::Result<()> {
        use std::io::Write;
        let stderr = std::io::stderr();
        let mut handle = stderr.lock();
        handle.write_all(msg.as_bytes())?;
        handle.write_all(b"\n")?;
        handle.flush()
    }
}

pub struct ProgressBar {
    total: u64,
    current: u64,
    message: String,
    start: std::time::Instant,
    width: usize,
}

impl ProgressBar {
    pub fn new(total: u64, message: &str) -> Self {
        Self {
            total,
            current: 0,
            message: message.to_string(),
            start: Instant::now(),
            width: 30,
        }
    }

    pub fn tick(&mut self, n: u64) {
        self.current += n;
        self.render();
    }

    pub fn set_message(&mut self, msg: &str) {
        self.message = msg.to_string();
        self.render();
    }

    pub fn finish(&mut self) {
        self.current = self.total;
        self.render();
        eprintln!();
    }

    fn render(&self) {
        let pct = if self.total > 0 {
            (self.current as f64 / self.total as f64).min(1.0)
        } else {
            0.0
        };
        let filled = (pct * self.width as f64) as usize;
        let empty = self.width.saturating_sub(filled);

        let elapsed = self.start.elapsed();
        let bar = format!(
            "\r{msg} [{filled}{empty}] {pct:>3.0}% ({current}/{total}) {elapsed:.1}s",
            msg = self.message,
            filled = Color::GREEN.paint("\u{2588}".repeat(filled)),
            empty = Color::DIM.paint("\u{2591}".repeat(empty)),
            pct = pct * 100.0,
            current = self.current,
            total = self.total,
            elapsed = elapsed.as_secs_f64(),
        );
        let _ = Color::write_stderr(&bar);
    }
}

pub struct Spinner {
    message: String,
    chars: &'static [char],
    idx: usize,
    start: Instant,
}

impl Spinner {
    pub fn new(message: &str) -> Self {
        Self {
            message: message.to_string(),
            chars: &['\u{280B}', '\u{2819}', '\u{2839}', '\u{2838}', '\u{2830}', '\u{2870}', '\u{2860}', '\u{2864}'],
            idx: 0,
            start: Instant::now(),
        }
    }

    pub fn tick(&mut self) {
        let c = self.chars[self.idx % self.chars.len()];
        let secs = self.start.elapsed().as_secs_f64();
        let _ = Color::write_stderr(&format!("\r{c} {} ({secs:.1}s)", self.message));
        self.idx += 1;
    }

    pub fn done(&self) {
        let secs = self.start.elapsed().as_secs_f64();
        let _ = Color::write_stderr(&format!(
            "\r{} {} ({secs:.1}s)",
            Color::GREEN.paint("\u{2713}"),
            self.message
        ));
    }

    pub fn fail(&self, reason: &str) {
        let _ = Color::write_stderr(&format!(
            "\r{} {}: {reason}",
            Color::RED.paint("\u{2717}"),
            self.message
        ));
    }
}

pub fn style_success(msg: impl fmt::Display) -> String {
    format!("{} {}", Color::GREEN.paint("\u{2713}"), msg)
}

pub fn style_error(msg: impl fmt::Display) -> String {
    format!("{} {}", Color::RED.paint("\u{2717}"), msg)
}

pub fn style_warning(msg: impl fmt::Display) -> String {
    format!("{} {}", Color::YELLOW.paint("\u{26A0}"), msg)
}

pub fn style_info(msg: impl fmt::Display) -> String {
    format!("{} {}", Color::CYAN.paint("i"), msg)
}
