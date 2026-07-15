use std::time::Instant;
use std::io::Write;
use crate::colors::{Color, supports_color};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProgressStyle {
    Dots,
    Bar,
    Percentage,
    Eta,
    Speed,
}

pub struct ProgressBar {
    total: u64,
    current: u64,
    message: String,
    start: Instant,
    width: usize,
    style: ProgressStyle,
    finished: bool,
    start_value: u64,
}

impl ProgressBar {
    pub fn new(total: u64, message: &str) -> Self {
        Self {
            total,
            current: 0,
            message: message.to_string(),
            start: Instant::now(),
            width: 40,
            style: ProgressStyle::Bar,
            finished: false,
            start_value: 0,
        }
    }

    pub fn with_style(mut self, style: ProgressStyle) -> Self {
        self.style = style;
        self
    }

    pub fn tick(&mut self, n: u64) {
        self.current = self.current.saturating_add(n);
        self.render();
    }

    pub fn set_message(&mut self, msg: &str) {
        self.message = msg.to_string();
        self.render();
    }

    pub fn set_position(&mut self, pos: u64) {
        self.current = pos.min(self.total);
        self.render();
    }

    pub fn finish(&mut self) {
        if self.finished { return; }
        self.current = self.total;
        self.finished = true;
        self.render();
        let _ = writeln!(std::io::stderr());
    }

    pub fn is_finished(&self) -> bool { self.finished }

    fn elapsed_secs(&self) -> f64 { self.start.elapsed().as_secs_f64() }

    fn render(&self) {
        if !supports_color() { return; }
        let pct = if self.total > 0 {
            (self.current as f64 / self.total as f64).min(1.0)
        } else {
            0.0
        };
        let elapsed = self.elapsed_secs();

        let line = match self.style {
            ProgressStyle::Dots => {
                let dots = (pct * self.width as f64) as usize;
                let empty = self.width.saturating_sub(dots);
                format!("\r{msg} [{dots}{empty}] {pct:>3.0}%",
                    msg = self.message,
                    dots = "\u{25CF}".repeat(dots),
                    empty = "\u{25CB}".repeat(empty),
                    pct = pct * 100.0,
                )
            }
            ProgressStyle::Bar | ProgressStyle::Percentage => {
                let filled = (pct * self.width as f64) as usize;
                let empty = self.width.saturating_sub(filled);
                let bar = format!("\r{msg} [{filled}{empty}] {pct:>3.0}% ({current}/{total}) {elapsed:.1}s",
                    msg = self.message,
                    filled = Color::GREEN.paint("\u{2588}".repeat(filled)),
                    empty = Color::DIM.paint("\u{2591}".repeat(empty)),
                    pct = pct * 100.0,
                    current = self.current,
                    total = self.total,
                    elapsed = elapsed,
                );
                bar
            }
            ProgressStyle::Eta => {
                let eta = if self.current > self.start_value {
                    let rate = self.current as f64 / elapsed;
                    let remaining = (self.total.saturating_sub(self.current)) as f64 / rate;
                    format_eta(remaining)
                } else {
                    "?".to_string()
                };
                format!("\r{msg} ... {pct:>3.0}% ETA: {eta}",
                    msg = self.message,
                    pct = pct * 100.0,
                    eta = eta,
                )
            }
            ProgressStyle::Speed => {
                let rate = if elapsed > 0.0 { self.current as f64 / elapsed } else { 0.0 };
                let rate_str = if rate > 1000.0 { format!("{:.1}K/s", rate / 1000.0) } else { format!("{:.0}/s", rate) };
                format!("\r{msg} ... {pct:>3.0}% ({rate_str})",
                    msg = self.message,
                    pct = pct * 100.0,
                    rate_str = rate_str,
                )
            }
        };
        let _ = write!(std::io::stderr(), "{line}");
        let _ = std::io::stderr().flush();
    }
}

fn format_eta(secs: f64) -> String {
    if secs.is_infinite() || secs.is_nan() || secs < 0.0 { return "? ".to_string(); }
    let total_secs = secs as u64;
    let hours = total_secs / 3600;
    let mins = (total_secs % 3600) / 60;
    let secs = total_secs % 60;
    if hours > 0 { format!("{hours}h{mins:02}m") }
    else if mins > 0 { format!("{mins}m{secs:02}s") }
    else { format!("{secs}s") }
}

pub struct MultiProgressBar {
    bars: Vec<ProgressBar>,
}

impl MultiProgressBar {
    pub fn new() -> Self {
        Self { bars: Vec::new() }
    }

    pub fn add(&mut self, total: u64, message: &str) -> usize {
        let idx = self.bars.len();
        self.bars.push(ProgressBar::new(total, message));
        idx
    }

    pub fn tick(&mut self, idx: usize, n: u64) {
        if let Some(bar) = self.bars.get_mut(idx) {
            bar.tick(n);
        }
    }

    pub fn finish(&mut self, idx: usize) {
        if let Some(bar) = self.bars.get_mut(idx) {
            bar.finish();
        }
    }

    pub fn finish_all(&mut self) {
        for i in 0..self.bars.len() {
            self.finish(i);
        }
    }
}

pub struct Spinner {
    message: String,
    chars: &'static [char],
    idx: usize,
    start: Instant,
    done_msg: Option<String>,
}

impl Spinner {
    pub fn new(message: &str) -> Self {
        Self {
            message: message.to_string(),
            chars: &['\u{280B}', '\u{2819}', '\u{2839}', '\u{2838}', '\u{2830}', '\u{2870}', '\u{2860}', '\u{2864}'],
            idx: 0,
            start: Instant::now(),
            done_msg: None,
        }
    }

    pub fn with_chars(mut self, chars: &'static [char]) -> Self {
        self.chars = chars;
        self
    }

    pub fn tick(&mut self) {
        if !supports_color() { return; }
        let c = self.chars[self.idx % self.chars.len()];
        let secs = self.start.elapsed().as_secs_f64();
        let _ = write!(std::io::stderr(), "\r{c} {} ({secs:.1}s)", self.message);
        let _ = std::io::stderr().flush();
        self.idx += 1;
    }

    pub fn done(&self) {
        let secs = self.start.elapsed().as_secs_f64();
        let msg = self.done_msg.as_deref().unwrap_or(&self.message);
        let _ = writeln!(std::io::stderr(), "\r{} {} ({secs:.1}s)", Color::GREEN.paint("\u{2713}"), msg);
    }

    pub fn fail(&self, reason: &str) {
        let _ = writeln!(std::io::stderr(), "\r{} {}: {reason}", Color::RED.paint("\u{2717}"), self.message);
    }

    pub fn set_message(&mut self, msg: &str) {
        self.message = msg.to_string();
    }

    pub fn elapsed(&self) -> std::time::Duration { self.start.elapsed() }
}

pub struct BytesProgress {
    bar: ProgressBar,
    downloaded: u64,
}

impl BytesProgress {
    pub fn new(total: u64, message: &str) -> Self {
        Self {
            bar: ProgressBar::new(total, message),
            downloaded: 0,
        }
    }

    pub fn tick(&mut self, bytes: u64) {
        self.downloaded = self.downloaded.saturating_add(bytes);
        self.bar.set_position(self.downloaded);
    }

    pub fn finish(&mut self) {
        self.bar.finish();
    }
}

pub struct StepProgress {
    steps: Vec<String>,
    current: usize,
    start: Instant,
}

impl StepProgress {
    pub fn new(steps: Vec<String>) -> Self {
        Self { steps, current: 0, start: Instant::now() }
    }

    pub fn begin_step(&mut self, step: &str) {
        if self.current < self.steps.len() {
            self.steps[self.current] = step.to_string();
        }
        let _ = writeln!(std::io::stderr(), "  {} {step} ...", Color::CYAN.paint("\u{25B6}"));
    }

    pub fn finish_step(&mut self) {
        if self.current < self.steps.len() {
            let step = &self.steps[self.current];
            let _ = writeln!(std::io::stderr(), "  {} {step}", Color::GREEN.paint("\u{2713}"));
        }
        self.current += 1;
    }

    pub fn fail_step(&mut self, reason: &str) {
        if self.current < self.steps.len() {
            let step = &self.steps[self.current];
            let _ = writeln!(std::io::stderr(), "  {} {step}: {reason}", Color::RED.paint("\u{2717}"));
        }
        self.current += 1;
    }

    pub fn elapsed(&self) -> std::time::Duration { self.start.elapsed() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_progress_bar() {
        let mut bar = ProgressBar::new(100, "test");
        bar.tick(50);
        assert!(!bar.is_finished());
        bar.finish();
        assert!(bar.is_finished());
    }

    #[test]
    fn test_progress_bar_overflow() {
        let mut bar = ProgressBar::new(10, "test");
        bar.tick(100);
        bar.finish();
        assert!(bar.is_finished());
    }

    #[test]
    fn test_spinner() {
        let mut spinner = Spinner::new("loading");
        spinner.tick();
        spinner.done();
    }

    #[test]
    fn test_spinner_fail() {
        let spinner = Spinner::new("loading");
        spinner.fail("error");
    }

    #[test]
    fn test_multi_progress() {
        let mut multi = MultiProgressBar::new();
        let idx = multi.add(100, "task1");
        multi.tick(idx, 50);
        multi.finish(idx);
    }

    #[test]
    fn test_bytes_progress() {
        let mut bp = BytesProgress::new(1000, "downloading");
        bp.tick(500);
        bp.finish();
    }

    #[test]
    fn test_step_progress() {
        let mut sp = StepProgress::new(vec!["step1".into(), "step2".into()]);
        sp.begin_step("step1");
        sp.finish_step();
        sp.begin_step("step2");
        sp.finish_step();
    }

    #[test]
    fn test_step_progress_fail() {
        let mut sp = StepProgress::new(vec!["step1".into()]);
        sp.begin_step("step1");
        sp.fail_step("something went wrong");
    }

    #[test]
    fn test_format_eta() {
        assert_eq!(format_eta(5.0), "5s");
        assert_eq!(format_eta(65.0), "1m05s");
        assert_eq!(format_eta(3665.0), "1h01m");
        assert_eq!(format_eta(-1.0), "? ");
        assert_eq!(format_eta(f64::INFINITY), "? ");
    }

    #[test]
    fn test_progress_with_style() {
        let mut bar = ProgressBar::new(100, "test").with_style(ProgressStyle::Dots);
        bar.tick(50);
        bar.finish();
    }

    #[test]
    fn test_progress_eta_style() {
        let mut bar = ProgressBar::new(100, "test").with_style(ProgressStyle::Eta);
        bar.tick(50);
        bar.finish();
    }

    #[test]
    fn test_progress_speed_style() {
        let mut bar = ProgressBar::new(100, "test").with_style(ProgressStyle::Speed);
        bar.tick(50);
        bar.finish();
    }
}
