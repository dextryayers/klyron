use std::collections::HashMap;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone)]
pub struct Cookie {
    pub name: String,
    pub value: String,
    pub expires: Option<SystemTime>,
    pub max_age: Option<Duration>,
    pub domain: Option<String>,
    pub path: Option<String>,
    pub secure: bool,
    pub http_only: bool,
    pub same_site: Option<SameSite>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SameSite {
    Strict,
    Lax,
    None,
}

impl Cookie {
    pub fn new(name: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            value: value.into(),
            expires: None,
            max_age: None,
            domain: None,
            path: Some("/".to_string()),
            secure: false,
            http_only: false,
            same_site: None,
        }
    }

    pub fn to_header(&self) -> String {
        let mut parts: Vec<String> = vec![format!("{}={}", self.name, self.value)];
        if let Some(expires) = self.expires {
            if let Ok(dur) = expires.duration_since(UNIX_EPOCH) {
                parts.push(format!(
                    "Expires={}",
                    http_date(dur.as_secs())
                ));
            }
        }
        if let Some(max_age) = self.max_age {
            parts.push(format!("Max-Age={}", max_age.as_secs()));
        }
        if let Some(ref domain) = self.domain {
            parts.push(format!("Domain={}", domain));
        }
        if let Some(ref path) = self.path {
            parts.push(format!("Path={}", path));
        }
        if self.secure {
            parts.push("Secure".to_string());
        }
        if self.http_only {
            parts.push("HttpOnly".to_string());
        }
        if let Some(same_site) = self.same_site {
            parts.push(format!(
                "SameSite={}",
                match same_site {
                    SameSite::Strict => "Strict",
                    SameSite::Lax => "Lax",
                    SameSite::None => "None",
                }
            ));
        }
        parts.join("; ")
    }
}

pub fn parse_cookie(header: &str) -> Vec<Cookie> {
    let mut cookies = Vec::new();
    for part in header.split(';') {
        let part = part.trim();
        if let Some((name, value)) = part.split_once('=') {
            let name = name.trim();
            let value = value.trim().trim_matches('"');
            if !name.is_empty() {
                cookies.push(Cookie::new(name, value));
            }
        }
    }
    cookies
}

pub fn parse_cookies(headers: &[String]) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for header in headers {
        for cookie in parse_cookie(header) {
            map.insert(cookie.name, cookie.value);
        }
    }
    map
}

pub struct CookieJar {
    cookies: HashMap<String, Cookie>,
}

impl CookieJar {
    pub fn new() -> Self {
        Self {
            cookies: HashMap::new(),
        }
    }

    pub fn insert(&mut self, cookie: Cookie) {
        self.cookies.insert(cookie.name.clone(), cookie);
    }

    pub fn get(&self, name: &str) -> Option<&Cookie> {
        self.cookies.get(name)
    }

    pub fn remove(&mut self, name: &str) {
        let mut cookie = Cookie::new(name, "");
        cookie.max_age = Some(Duration::ZERO);
        self.cookies.insert(name.to_string(), cookie);
    }

    pub fn iter(&self) -> impl Iterator<Item = &Cookie> {
        self.cookies.values()
    }

    pub fn to_set_cookie_headers(&self) -> Vec<String> {
        self.cookies.values().map(|c| c.to_header()).collect()
    }
}

impl Default for CookieJar {
    fn default() -> Self {
        Self::new()
    }
}

fn http_date(secs: u64) -> String {
    use std::fmt::Write;
    let secs = secs as i64;
    let days = secs / 86400;
    let time_secs = (secs % 86400) as u32;
    let time_hours = time_secs / 3600;
    let time_minutes = (time_secs % 3600) / 60;
    let time_seconds = time_secs % 60;
    let weekday = match days % 7 {
        0 => "Thu",
        1 => "Fri",
        2 => "Sat",
        3 => "Sun",
        4 => "Mon",
        5 => "Tue",
        6 => "Wed",
        _ => unreachable!(),
    };
    let month_idx = ((days % 365) as u32) / 31;
    let month = match month_idx.min(11) {
        0 => "Jan",
        1 => "Feb",
        2 => "Mar",
        3 => "Apr",
        4 => "May",
        5 => "Jun",
        6 => "Jul",
        7 => "Aug",
        8 => "Sep",
        9 => "Oct",
        10 => "Nov",
        11 => "Dec",
        _ => unreachable!(),
    };
    let day = (days % 31) as u32 + 1;
    let year = 1970 + (days / 365) as u32;
    let mut buf = String::with_capacity(29);
    write!(
        buf,
        "{}, {:02} {} {:04} {:02}:{:02}:{:02} GMT",
        weekday, day, month, year, time_hours, time_minutes, time_seconds
    )
    .ok();
    buf
}
