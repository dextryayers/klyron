use boa_engine::{Context, JsValue, NativeFunction, js_string, JsString};
use boa_engine::property::Attribute;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct DateExtensions;

impl DateExtensions {
    pub fn now_millis() -> f64 {
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis() as f64
    }

    pub fn now_seconds() -> f64 {
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs_f64()
    }

    pub fn format_date(format: &str, timestamp: Option<f64>) -> String {
        let ts = timestamp.unwrap_or_else(Self::now_millis);
        let secs = (ts / 1000.0) as u64;
        let nanos = ((ts % 1000.0) * 1_000_000.0) as u32;
        let since_epoch = std::time::Duration::new(secs, nanos);
        let datetime = match UNIX_EPOCH.checked_add(since_epoch) {
            Some(t) => t,
            None => return "invalid-date".to_string(),
        };
        let d = datetime.duration_since(UNIX_EPOCH).unwrap_or_default();
        let total_secs = d.as_secs();
        let days = total_secs / 86400;
        let time_secs = total_secs % 86400;
        let hours = time_secs / 3600;
        let minutes = (time_secs % 3600) / 60;
        let seconds = time_secs % 60;

        let mut year = 1970i64;
        let mut remaining_days = days as i64;
        loop {
            let days_in_year = if is_leap_year(year) { 366 } else { 365 };
            if remaining_days < days_in_year { break; }
            remaining_days -= days_in_year;
            year += 1;
        }
        let month_days = if is_leap_year(year) {
            [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
        } else {
            [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
        };
        let mut month = 0usize;
        for (i, &md) in month_days.iter().enumerate() {
            if remaining_days < md { month = i; break; }
            remaining_days -= md;
        }
        let day = remaining_days + 1;

        match format {
            "iso" => format!("{:04}-{:02}-{:02}T{:02}:{:02}:{:02}.{:03}Z", year, month + 1, day, hours, minutes, seconds, nanos / 1_000_000),
            "date" => format!("{:04}-{:02}-{:02}", year, month + 1, day),
            "time" => format!("{:02}:{:02}:{:02}", hours, minutes, seconds),
            "datetime" => format!("{:04}-{:02}-{:02} {:02}:{:02}:{:02}", year, month + 1, day, hours, minutes, seconds),
            _ => format!("{:04}-{:02}-{:02}T{:02}:{:02}:{:02}.{:03}Z", year, month + 1, day, hours, minutes, seconds, nanos / 1_000_000),
        }
    }

    pub fn register(context: &mut Context) -> Result<(), crate::BoaError> {
        let mut builder = boa_engine::object::ObjectInitializer::new(context);
        builder.function(
            NativeFunction::from_fn_ptr(|_this, _args, _ctx| Ok(JsValue::from(Self::now_millis()))),
            js_string!("now"), 0usize,
        ).function(
            NativeFunction::from_fn_ptr(|_this, _args, _ctx| Ok(JsValue::from(Self::now_seconds()))),
            js_string!("nowSeconds"), 0usize,
        ).function(
            NativeFunction::from_fn_ptr(|_this, args, _ctx| {
                let ts = args.first().and_then(|v| v.as_number()).unwrap_or_else(Self::now_millis);
                Ok(JsValue::from(JsString::from(Self::format_date("iso", Some(ts)))))
            }),
            js_string!("toISO"), 1usize,
        ).function(
            NativeFunction::from_fn_ptr(|_this, args, _ctx| {
                let ts = args.first().and_then(|v| v.as_number()).unwrap_or_else(Self::now_millis);
                let fmt = args.get(1).and_then(|v| v.as_string())
                    .map(|s| s.to_std_string_escaped())
                    .unwrap_or_else(|| "iso".to_string());
                Ok(JsValue::from(JsString::from(Self::format_date(&fmt, Some(ts)))))
            }),
            js_string!("format"), 2usize,
        );
        let obj = builder.build();
        context.register_global_property(
            js_string!("DateUtils"),
            obj,
            Attribute::WRITABLE | Attribute::CONFIGURABLE | Attribute::ENUMERABLE,
        ).map_err(|e| crate::BoaError::from_js_error(&e))
    }
}

fn is_leap_year(year: i64) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}
