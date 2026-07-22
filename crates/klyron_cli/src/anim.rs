use std::io::Write;
use std::time::{Duration, Instant};

pub fn rgb(r: u8, g: u8, b: u8, text: &str) -> String {
    format!("\x1b[38;2;{};{};{}m{}\x1b[0m", r, g, b, text)
}

fn ansi_rgb(r: u8, g: u8, b: u8) -> String {
    format!("\x1b[38;2;{};{};{}m", r, g, b)
}

fn hide_cursor() { print!("\x1b[?25l"); }
fn show_cursor() { print!("\x1b[?25h"); }
pub fn clear_line() { print!("\r\x1b[K"); }

const COLORS: [(u8, u8, u8); 6] = [
    (255, 56, 168),
    (200, 40, 255),
    (140, 80, 255),
    (80, 120, 255),
    (0, 200, 255),
    (0, 230, 180),
];

fn gradient_color(t: f64) -> (u8, u8, u8) {
    let t = t.clamp(0.0, 1.0);
    let idx = (t * (COLORS.len() - 1) as f64) as usize;
    let next = (idx + 1).min(COLORS.len() - 1);
    let frac = t * (COLORS.len() - 1) as f64 - idx as f64;
    let (r1, g1, b1) = COLORS[idx];
    let (r2, g2, b2) = COLORS[next];
    (
        (r1 as f64 + (r2 as f64 - r1 as f64) * frac) as u8,
        (g1 as f64 + (g2 as f64 - g1 as f64) * frac) as u8,
        (b1 as f64 + (b2 as f64 - b1 as f64) * frac) as u8,
    )
}

pub struct GradientBar {
    message: String,
    total: u64,
    current: u64,
    width: usize,
    start: Instant,
    finished: bool,
}

impl GradientBar {
    pub fn new(total: u64, message: &str) -> Self {
        let mut s = Self {
            message: message.to_string(),
            total,
            current: 0,
            width: 30,
            start: Instant::now(),
            finished: false,
        };
        s.render();
        s
    }

    pub fn tick(&mut self, n: u64) {
        self.current = self.current.saturating_add(n).min(self.total);
        self.render();
    }

    pub fn set_message(&mut self, msg: &str) {
        self.message = msg.to_string();
        self.render();
    }

    pub fn finish_with(&mut self, msg: &str) {
        self.finished = true;
        let elapsed = self.start.elapsed();
        clear_line();
        let check = rgb(0, 230, 180, "✓");
        let _ = writeln!(std::io::stderr(), "  {}  {} ({:.1}s)", check, msg, elapsed.as_secs_f64());
    }

    fn render(&self) {
        if self.finished { return; }
        let progress = if self.total > 0 {
            (self.current as f64 / self.total as f64).min(1.0)
        } else {
            0.0
        };
        let filled = (progress * self.width as f64) as usize;
        let elapsed = self.start.elapsed().as_secs_f64();

        let bar: String = (0..self.width)
            .map(|j| {
                if j < filled {
                    let t = j as f64 / self.width.max(1) as f64;
                    let (r, g, b) = gradient_color(t);
                    let pulse = ((elapsed * 3.0 + j as f64 * 0.3).sin() * 0.15 + 0.85)
                        * (1.0 - (j as f64 / self.width.max(1) as f64) * 0.3);
                    let (r2, g2, b2) = (
                        (r as f64 * pulse) as u8,
                        (g as f64 * pulse) as u8,
                        (b as f64 * pulse) as u8,
                    );
                    let blocks = ["█", "▓", "▒"];
                    let block = blocks[(j + (elapsed * 4.0) as usize) % 3];
                    rgb(r2, g2, b2, block)
                } else {
                    let trail = if j < filled + 3 && j >= filled {
                        let dist = j - filled;
                        let brightness = 40 + (20 - dist as u8 * 5).max(0).min(20);
                        rgb(brightness, brightness, brightness + 10, "░")
                    } else {
                        rgb(60, 60, 80, "░")
                    };
                    trail
                }
            })
            .collect();

        let pct = format!("{:>3}%", (progress * 100.0) as u8);
        let pct_colored = rgb(140, 80, 255, &pct);
        let elapsed_str = format!("{:.1}s", elapsed);
        let elapsed_colored = rgb(100, 100, 120, &elapsed_str);
        let msg = rgb(180, 180, 200, &self.message);

        let _ = write!(std::io::stderr(), "\r  {}  {}  {}  {}", bar, pct_colored, msg, elapsed_colored);
        let _ = std::io::stderr().flush();
    }
}

pub struct PulseSpinner {
    message: String,
    idx: usize,
    start: Instant,
    finished: bool,
}

impl PulseSpinner {
    pub fn new(message: &str) -> Self {
        let mut s = Self {
            message: message.to_string(),
            idx: 0,
            start: Instant::now(),
            finished: false,
        };
        s.render();
        s
    }

    pub fn set_message(&mut self, msg: &str) {
        self.message = msg.to_string();
        self.render();
    }

    pub fn tick(&mut self) {
        self.idx += 1;
        self.render();
    }

    pub fn done(&mut self, msg: &str) {
        self.finished = true;
        clear_line();
        let elapsed = self.start.elapsed();
        let check = rgb(0, 230, 180, "✓");
        let _ = writeln!(std::io::stderr(), "  {}  {} ({:.1}s)", check, msg, elapsed.as_secs_f64());
    }

    pub fn fail(&mut self, msg: &str) {
        self.finished = true;
        clear_line();
        let cross = rgb(255, 56, 56, "✗");
        let _ = writeln!(std::io::stderr(), "  {}  {}", cross, msg);
    }

    fn render(&self) {
        if self.finished { return; }
        let elapsed = self.start.elapsed().as_secs_f64();
        let phase = (elapsed * 3.0).sin() * 0.5 + 0.5;
        let (r, g, b) = gradient_color(phase);

        let dots = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
        let dot_idx = (elapsed * 8.0) as usize % dots.len();
        let dot = rgb(r, g, b, dots[dot_idx]);
        let msg = rgb(180, 180, 200, &self.message);
        let _ = write!(std::io::stderr(), "\r  {}  {}  {:.1}s", dot, msg, elapsed);
        let _ = std::io::stderr().flush();
    }
}

pub struct StepAnim {
    steps: Vec<String>,
    current: usize,
    start: Instant,
}

impl StepAnim {
    pub fn new(steps: Vec<String>) -> Self {
        Self { steps, current: 0, start: Instant::now() }
    }

    pub fn begin(&mut self, label: &str) {
        if self.current > 0 { println!(); }
        let arrow = rgb(140, 80, 255, "▶");
        let msg = rgb(200, 200, 220, label);
        let _ = writeln!(std::io::stderr(), "\n  {}  {}", arrow, msg);
    }

    pub fn step_begin(&mut self, detail: &str) {
        let dot = rgb(80, 120, 255, "◌");
        let msg = rgb(180, 180, 200, detail);
        let _ = write!(std::io::stderr(), "    {}  {}", dot, msg);
        let _ = std::io::stderr().flush();
    }

    pub fn step_ok(&mut self, detail: &str) {
        clear_line();
        let check = rgb(0, 230, 180, "✓");
        let msg = rgb(200, 200, 220, detail);
        let _ = writeln!(std::io::stderr(), "    {}  {}", check, msg);
    }

    pub fn step_done(&mut self) {
        if self.current < self.steps.len() {
            clear_line();
            let check = rgb(0, 230, 180, "✓");
            let msg = rgb(200, 200, 220, &self.steps[self.current]);
            let _ = writeln!(std::io::stderr(), "    {}  {}", check, msg);
        }
        self.current += 1;
    }

    pub fn step_fail(&mut self, reason: &str) {
        clear_line();
        let cross = rgb(255, 56, 56, "✗");
        let _ = writeln!(std::io::stderr(), "    {}  {}", cross, reason);
        self.current += 1;
    }

    pub fn done(&self) {
        let elapsed = self.start.elapsed();
        println!();
        let check = rgb(0, 230, 180, "✓");
        let msg = rgb(180, 180, 200, &format!("Done in {:.1}s", elapsed.as_secs_f64()));
        let _ = writeln!(std::io::stderr(), "  {}  {}", check, msg);
    }
}

pub fn pulsing_dots(msg: &str, duration_ms: u64) {
    let frames = ['●', '◔', '◐', '◕', '○', '◕', '◐', '◔'];
    let steps = duration_ms / 60;
    for i in 0..steps {
        let f = frames[(i as usize) % frames.len()];
        let t = i as f64 / steps.max(1) as f64;
        let (r, g, b) = gradient_color(t);
        let dot = rgb(r, g, b, &f.to_string());
        let _ = write!(std::io::stderr(), "\r  {}  {}", dot, msg);
        let _ = std::io::stderr().flush();
        std::thread::sleep(Duration::from_millis(60));
    }
    clear_line();
}

pub fn success_banner(msg: &str) {
    let line = "─".repeat(msg.len() + 8);
    let (r, g, b) = COLORS[5];
    let border = rgb(r, g, b, &format!("  ┌{}┐", line));
    let check = rgb(0, 230, 180, "✓");
    let text = rgb(r, g, b, &format!("  │  {}  {}  │", check, msg));
    let border2 = rgb(r, g, b, &format!("  └{}┘", line));
    println!("\n{}\n{}\n{}", border, text, border2);
}

pub fn cmd_header(cmd: &str, desc: &str) {
    let (r, g, b) = gradient_color(0.8);
    let cmd_colored = rgb(r, g, b, cmd);
    let desc_colored = rgb(180, 180, 200, desc);
    let _ = writeln!(std::io::stderr(), "\n  {}  {}", cmd_colored, desc_colored);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gradient_bar() {
        let mut bar = GradientBar::new(100, "test");
        bar.tick(50);
        bar.finish_with("done");
        assert!(bar.finished);
    }

    #[test]
    fn test_pulse_spinner() {
        let mut s = PulseSpinner::new("loading");
        s.tick();
        s.done("done");
        assert!(s.finished);
    }

    #[test]
    fn test_pulse_spinner_fail() {
        let mut s = PulseSpinner::new("loading");
        s.fail("error");
        assert!(s.finished);
    }

    #[test]
    fn test_step_anim() {
        let mut sa = StepAnim::new(vec!["step1".into(), "step2".into()]);
        sa.begin("Phase 1");
        sa.step_begin("subtask");
        sa.step_done();
        sa.step_done();
        sa.done();
    }

    #[test]
    fn test_step_fail() {
        let mut sa = StepAnim::new(vec!["step".into()]);
        sa.begin("Phase");
        sa.step_fail("error");
    }

    #[test]
    fn test_gradient_color_range() {
        for i in 0..10 {
            let t = i as f64 / 9.0;
            let (r, g, b) = gradient_color(t);
            assert!(r <= 255 && g <= 255 && b <= 255);
        }
    }

    #[test]
    fn test_success_banner() {
        success_banner("All tests passed");
    }

    #[test]
    fn test_cmd_header() {
        cmd_header("install", "install packages");
    }
}
