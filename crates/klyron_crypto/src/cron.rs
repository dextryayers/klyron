use chrono::{DateTime, Datelike, Timelike, Utc};

/// Describe a cron expression in human-readable English.
///
/// Supports standard 5-field cron: `minute hour day-of-month month day-of-week`
pub fn describe_cron(expr: &str) -> String {
    let parts: Vec<&str> = expr.split_whitespace().collect();
    if parts.len() != 5 {
        return format!("invalid cron expression: {expr}");
    }
    let (min, hour, dom, mon, dow) = (parts[0], parts[1], parts[2], parts[3], parts[4]);

    let mut desc = String::new();

    // Day of week
    match dow {
        "*" => {}
        "1" => desc.push_str("on Monday"),
        "2" => desc.push_str("on Tuesday"),
        "3" => desc.push_str("on Wednesday"),
        "4" => desc.push_str("on Thursday"),
        "5" => desc.push_str("on Friday"),
        "6" => desc.push_str("on Saturday"),
        "0" | "7" => desc.push_str("on Sunday"),
        range if range.contains('-') => {
            let dows = expand_range(range, 0, 7);
            let names: Vec<&str> = dows.iter().map(|&d| day_name(d)).collect();
            desc.push_str(&format!("on {}", names.join(", ")));
        }
        _ => {}
    }

    // Month
    match mon {
        "*" => {}
        m => {
            if !desc.is_empty() { desc.insert_str(0, " "); }
            let month_name = match m {
                "1" => "January", "2" => "February", "3" => "March",
                "4" => "April", "5" => "May", "6" => "June",
                "7" => "July", "8" => "August", "9" => "September",
                "10" => "October", "11" => "November", "12" => "December",
                _ => m,
            };
            desc = format!("in {month_name}{desc}");
        }
    }

    // Day of month
    match dom {
        "*" => {}
        d => {
            let prefix = if desc.contains("on ") { "and " } else { "" };
            desc = format!("{prefix}on day {d} {desc}");
        }
    }

    // Hour and minute
    match (min, hour) {
        ("0", "0") => desc = format!("At midnight{}", prefix_space(&desc)),
        ("0", h) => desc = format!("At {}:00{}", hour12(h), prefix_space(&desc)),
        (m, "*") => desc = format!("Every hour at minute {m}{}", prefix_space(&desc)),
        (m, h) => desc = format!("At {}:{:02}{}", hour12(h), m.parse::<u32>().unwrap_or(0), prefix_space(&desc)),
    }

    if desc.is_empty() {
        desc = "Every minute".to_string();
    }

    // Handle special cases like "*/5"
    if let Some(step) = min.strip_prefix("*/") {
        desc = format!("Every {step} minutes");
    }
    if let Some(step) = hour.strip_prefix("*/") {
        if desc.starts_with("Every") {
            // already handled
        } else {
            desc = format!("Every {step} hours");
        }
    }

    desc
        .replace("  ", " ")
        .trim()
        .to_string()
}

/// Compute the next N occurrences of a cron expression from a given time.
pub fn next_run(expr: &str, from: Option<DateTime<Utc>>, count: usize) -> Vec<DateTime<Utc>> {
    let parts: Vec<&str> = expr.split_whitespace().collect();
    if parts.len() != 5 || count == 0 {
        return Vec::new();
    }

    let parsed = CronExpr::parse(parts);
    let parsed = match parsed {
        Some(p) => p,
        None => return Vec::new(),
    };

    let mut current = from.unwrap_or_else(Utc::now);
    let mut results = Vec::with_capacity(count);

    while results.len() < count {
        current = current + chrono::Duration::minutes(1);
        current = current.with_second(0).unwrap();

        if parsed.matches(&current) {
            results.push(current);
        }

        // safety valve: prevent infinite loop
        if results.len() < count && current > Utc::now() + chrono::Duration::days(365 * 5) {
            break;
        }
    }

    results
}

fn prefix_space(s: &str) -> String {
    if s.is_empty() { String::new() } else { format!(" {s}") }
}

fn hour12(h: &str) -> String {
    match h.parse::<u32>() {
        Ok(0) => "12 AM".into(),
        Ok(1..=11) => format!("{h} AM"),
        Ok(12) => "12 PM".into(),
        Ok(h24 @ 13..=23) => format!("{} PM", h24 - 12),
        _ => h.to_string(),
    }
}

fn day_name(d: u32) -> &'static str {
    match d {
        0 | 7 => "Sunday",
        1 => "Monday",
        2 => "Tuesday",
        3 => "Wednesday",
        4 => "Thursday",
        5 => "Friday",
        6 => "Saturday",
        _ => "Unknown",
    }
}

fn expand_range(s: &str, min: u32, max: u32) -> Vec<u32> {
    let parts: Vec<&str> = s.split('-').collect();
    if parts.len() != 2 { return Vec::new(); }
    let start: u32 = parts[0].parse().unwrap_or(min);
    let end: u32 = parts[1].parse().unwrap_or(max);
    (start..=end).collect()
}

struct CronExpr {
    minute: CronField,
    hour: CronField,
    dom: CronField,
    month: CronField,
    dow: CronField,
}

impl CronExpr {
    fn parse(parts: Vec<&str>) -> Option<Self> {
        Some(Self {
            minute: CronField::parse(parts[0], 0, 59)?,
            hour: CronField::parse(parts[1], 0, 23)?,
            dom: CronField::parse(parts[2], 1, 31)?,
            month: CronField::parse(parts[3], 1, 12)?,
            dow: CronField::parse(parts[4], 0, 7)?,
        })
    }

    fn matches(&self, dt: &DateTime<Utc>) -> bool {
        self.minute.matches(dt.minute())
            && self.hour.matches(dt.hour())
            && self.dom.matches(dt.day())
            && self.month.matches(dt.month())
            && self.dow.matches(dt.weekday().num_days_from_sunday())
    }
}

enum CronField {
    All,
    Value(u32),
    Step { base: u32, step: u32 },
    Range { start: u32, end: u32 },
    List(Vec<u32>),
}

impl CronField {
    fn parse(s: &str, _min: u32, _max: u32) -> Option<Self> {
        if s == "*" {
            return Some(Self::All);
        }
        if let Some(rest) = s.strip_prefix("*/") {
            let step: u32 = rest.parse().ok()?;
            return Some(Self::Step { base: 0, step });
        }
        if s.contains(',') {
            let vals: Vec<u32> = s.split(',').filter_map(|v| v.parse().ok()).collect();
            return Some(Self::List(vals));
        }
        if s.contains('-') {
            let parts: Vec<&str> = s.split('-').collect();
            if parts.len() == 2 {
                let start: u32 = parts[0].parse().ok()?;
                let end: u32 = parts[1].parse().ok()?;
                return Some(Self::Range { start, end });
            }
        }
        let val: u32 = s.parse().ok()?;
        Some(Self::Value(val))
    }

    fn matches(&self, v: u32) -> bool {
        match self {
            Self::All => true,
            Self::Value(x) => *x == v,
            Self::Step { base, step } => v >= *base && (v - base) % step == 0,
            Self::Range { start, end } => v >= *start && v <= *end,
            Self::List(vals) => vals.contains(&v),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_describe_midnight() {
        let desc = describe_cron("0 0 * * *");
        assert_eq!(desc, "At midnight");
    }

    #[test]
    fn test_describe_every_5_minutes() {
        let desc = describe_cron("*/5 * * * *");
        assert_eq!(desc, "Every 5 minutes");
    }

    #[test]
    fn test_describe_workday_morning() {
        let desc = describe_cron("0 9 * * 1-5");
        assert!(desc.contains("9 AM") || desc.contains("Monday"));
    }

    #[test]
    fn test_next_run_basic() {
        let from = Utc::now();
        let next = next_run("0 * * * *", Some(from), 3);
        assert_eq!(next.len(), 3);
        for w in next.windows(2) {
            assert!(w[0] < w[1]);
        }
    }

    #[test]
    fn test_next_run_empty_for_invalid() {
        let next = next_run("invalid", None, 5);
        assert!(next.is_empty());
    }
}
