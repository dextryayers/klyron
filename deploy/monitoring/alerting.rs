use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertRule {
    pub name: String,
    pub condition: AlertCondition,
    pub threshold: f64,
    pub duration_seconds: u64,
    pub severity: AlertSeverity,
    pub channel: AlertChannel,
    pub metric_name: String,
    pub description: String,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertCondition {
    GreaterThan,
    LessThan,
    Equals,
    Contains,
    Regex(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum AlertSeverity {
    Info,
    Warning,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertChannel {
    Email(Vec<String>),
    Slack(String),
    PagerDuty(String),
    Webhook(String),
    Sms(Vec<String>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertEvent {
    pub rule: AlertRule,
    pub value: f64,
    pub message: String,
    pub timestamp: DateTime<Utc>,
    pub resolved: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertManager {
    pub rules: Vec<AlertRule>,
    pub events: Vec<AlertEvent>,
    silenced_until: HashMap<String, DateTime<Utc>>,
}

impl AlertManager {
    pub fn new() -> Self {
        Self {
            rules: Vec::new(),
            events: Vec::new(),
            silenced_until: HashMap::new(),
        }
    }

    pub fn add_rule(&mut self, rule: AlertRule) {
        self.rules.push(rule);
    }

    pub fn remove_rule(&mut self, name: &str) {
        self.rules.retain(|r| r.name != name);
    }

    pub fn evaluate(&mut self, metric_name: &str, value: f64) -> Vec<AlertEvent> {
        let now = Utc::now();
        let mut triggered = Vec::new();
        let matching_rules: Vec<AlertRule> = self
            .rules
            .iter()
            .filter(|r| r.enabled && r.metric_name == metric_name)
            .cloned()
            .collect();
        for rule in matching_rules {
            if let Some(silenced_until) = self.silenced_until.get(&rule.name) {
                if &now < silenced_until {
                    continue;
                }
            }
            let condition_met = match rule.condition {
                AlertCondition::GreaterThan => value > rule.threshold,
                AlertCondition::LessThan => value < rule.threshold,
                AlertCondition::Equals => (value - rule.threshold).abs() < f64::EPSILON,
                AlertCondition::Contains => value.to_string().contains(&rule.threshold.to_string()),
                AlertCondition::Regex(ref pattern) => {
                    regex::Regex::new(pattern)
                        .ok()
                        .map(|re| re.is_match(&value.to_string()))
                        .unwrap_or(false)
                }
            };
            if condition_met {
                let event = AlertEvent {
                    rule: rule.clone(),
                    value,
                    message: format!(
                        "Alert '{}' triggered: {} {} {} (value: {})",
                        rule.name,
                        rule.metric_name,
                        match rule.condition {
                            AlertCondition::GreaterThan => ">",
                            AlertCondition::LessThan => "<",
                            AlertCondition::Equals => "==",
                            AlertCondition::Contains => "contains",
                            AlertCondition::Regex(_) => "matches",
                        },
                        rule.threshold,
                        value
                    ),
                    timestamp: now,
                    resolved: false,
                };
                triggered.push(event.clone());
                self.events.push(event);
            }
        }
        triggered
    }

    pub fn silence(&mut self, rule_name: &str, until: DateTime<Utc>) {
        self.silenced_until.insert(rule_name.to_string(), until);
    }

    pub fn unsilence(&mut self, rule_name: &str) {
        self.silenced_until.remove(rule_name);
    }

    pub fn resolve(&mut self, rule_name: &str) {
        let now = Utc::now();
        for event in self.events.iter_mut() {
            if event.rule.name == rule_name {
                event.resolved = true;
                event.timestamp = now;
            }
        }
    }

    pub fn send_alert(&self, event: &AlertEvent) -> Result<(), String> {
        match &event.rule.channel {
            AlertChannel::Email(recipients) => {
                for to in recipients {
                    self.send_email(to, &event.message)?;
                }
            }
            AlertChannel::Slack(webhook_url) => {
                self.send_slack(webhook_url, &event.message)?;
            }
            AlertChannel::PagerDuty(integration_key) => {
                self.send_pagerduty(integration_key, &event)?;
            }
            AlertChannel::Webhook(url) => {
                self.send_webhook(url, &event)?;
            }
            AlertChannel::Sms(phones) => {
                for phone in phones {
                    self.send_sms(phone, &event.message)?;
                }
            }
        }
        Ok(())
    }

    fn send_email(&self, to: &str, message: &str) -> Result<(), String> {
        println!("[EMAIL TO {to}] {message}");
        Ok(())
    }

    fn send_slack(&self, webhook_url: &str, message: &str) -> Result<(), String> {
        let payload = serde_json::json!({
            "text": message,
            "username": "Klyron AlertManager",
            "icon_emoji": ":warning:",
        });
        let client = reqwest::blocking::Client::new();
        client
            .post(webhook_url)
            .json(&payload)
            .send()
            .map(|_| ())
            .map_err(|e| format!("Slack alert failed: {e}"))
    }

    fn send_pagerduty(&self, integration_key: &str, event: &AlertEvent) -> Result<(), String> {
        let severity = match event.rule.severity {
            AlertSeverity::Info => "info",
            AlertSeverity::Warning => "warning",
            AlertSeverity::Critical => "critical",
        };
        let payload = serde_json::json!({
            "routing_key": integration_key,
            "event_action": "trigger",
            "payload": {
                "summary": event.message,
                "severity": severity,
                "source": "klyron-alertmanager",
                "custom_details": {
                    "metric": event.rule.metric_name,
                    "value": event.value,
                    "threshold": event.rule.threshold,
                }
            }
        });
        let client = reqwest::blocking::Client::new();
        client
            .post("https://events.pagerduty.com/v2/enqueue")
            .json(&payload)
            .send()
            .map(|_| ())
            .map_err(|e| format!("PagerDuty alert failed: {e}"))
    }

    fn send_webhook(&self, url: &str, event: &AlertEvent) -> Result<(), String> {
        let payload = serde_json::json!(event);
        let client = reqwest::blocking::Client::new();
        client
            .post(url)
            .json(&payload)
            .send()
            .map(|_| ())
            .map_err(|e| format!("Webhook alert failed: {e}"))
    }

    fn send_sms(&self, phone: &str, message: &str) -> Result<(), String> {
        println!("[SMS TO {phone}] {message}");
        Ok(())
    }

    pub fn get_recent_alerts(&self, count: usize, severity: Option<AlertSeverity>) -> Vec<&AlertEvent> {
        let mut events: Vec<&AlertEvent> = self
            .events
            .iter()
            .filter(|e| {
                severity
                    .as_ref()
                    .map(|s| e.rule.severity == *s)
                    .unwrap_or(true)
            })
            .collect();
        events.reverse();
        events.truncate(count);
        events
    }

    pub fn get_alerts_summary(&self) -> AlertSummary {
        let total = self.events.len();
        let open = self.events.iter().filter(|e| !e.resolved).count();
        let critical = self.events.iter().filter(|e| e.rule.severity == AlertSeverity::Critical && !e.resolved).count();
        let warning = self.events.iter().filter(|e| e.rule.severity == AlertSeverity::Warning && !e.resolved).count();
        let info = self.events.iter().filter(|e| e.rule.severity == AlertSeverity::Info && !e.resolved).count();
        AlertSummary {
            total_events: total,
            open_alerts: open,
            critical,
            warning,
            info,
        }
    }
}

impl Default for AlertManager {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertSummary {
    pub total_events: usize,
    pub open_alerts: usize,
    pub critical: usize,
    pub warning: usize,
    pub info: usize,
}
