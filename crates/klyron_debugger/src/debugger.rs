use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::client::DebugClient;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Breakpoint {
    pub id: u32,
    pub url: String,
    pub line: u32,
    pub column: Option<u32>,
    pub condition: Option<String>,
    pub hit_count: u64,
    pub enabled: bool,
}

#[derive(Debug, Clone)]
pub struct Debugger {
    pub clients: Vec<DebugClient>,
    pub breakpoints: Vec<Breakpoint>,
    pub watch_expressions: Vec<String>,
    enabled: bool,
    next_bp_id: u32,
}

impl Debugger {
    pub fn new() -> Self {
        Self {
            clients: Vec::new(),
            breakpoints: Vec::new(),
            watch_expressions: Vec::new(),
            enabled: false,
            next_bp_id: 1,
        }
    }

    pub async fn enable(&mut self) -> Result<()> {
        self.enabled = true;
        tracing::info!("Debugger enabled");
        Ok(())
    }

    pub fn set_breakpoint(&mut self, url: &str, line: u32) -> u32 {
        let id = self.next_bp_id;
        self.next_bp_id += 1;
        self.breakpoints.push(Breakpoint {
            id,
            url: url.to_string(),
            line,
            column: None,
            condition: None,
            hit_count: 0,
            enabled: true,
        });
        id
    }

    pub fn remove_breakpoint(&mut self, id: u32) -> bool {
        let len = self.breakpoints.len();
        self.breakpoints.retain(|bp| bp.id != id);
        self.breakpoints.len() < len
    }

    pub fn step_over(&mut self) {
        tracing::debug!("Step over");
    }

    pub fn step_into(&mut self) {
        tracing::debug!("Step into");
    }

    pub fn step_out(&mut self) {
        tracing::debug!("Step out");
    }

    pub fn continue_(&mut self) {
        tracing::debug!("Continue");
    }

    pub async fn evaluate(&self, expr: &str, frame_id: &str) -> Result<serde_json::Value> {
        Ok(serde_json::json!({
            "result": expr,
            "frameId": frame_id,
        }))
    }

    pub async fn pause(&mut self) -> Result<()> {
        tracing::info!("Pause requested");
        Ok(())
    }

    pub async fn resume(&mut self) -> Result<()> {
        tracing::info!("Resume requested");
        Ok(())
    }

    pub async fn get_source(&self, _url: &str) -> Result<String> {
        Ok(String::new())
    }

    pub fn hit_breakpoint(&mut self, url: &str, line: u32) -> Option<&Breakpoint> {
        let idx = self.breakpoints.iter().position(|bp| {
            bp.url == url && bp.line == line && bp.enabled
        })?;
        self.breakpoints[idx].hit_count += 1;
        Some(&self.breakpoints[idx])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_debugger_new() {
        let d = Debugger::new();
        assert!(d.clients.is_empty());
        assert!(d.breakpoints.is_empty());
        assert!(d.watch_expressions.is_empty());
    }

    #[test]
    fn test_set_breakpoint() {
        let mut d = Debugger::new();
        let id = d.set_breakpoint("test.js", 42);
        assert_eq!(id, 1);
        assert_eq!(d.breakpoints.len(), 1);
        assert_eq!(d.breakpoints[0].url, "test.js");
        assert_eq!(d.breakpoints[0].line, 42);
    }

    #[test]
    fn test_remove_breakpoint() {
        let mut d = Debugger::new();
        let id1 = d.set_breakpoint("a.js", 1);
        let id2 = d.set_breakpoint("b.js", 2);
        assert_eq!(d.breakpoints.len(), 2);
        assert!(d.remove_breakpoint(id1));
        assert_eq!(d.breakpoints.len(), 1);
        assert_eq!(d.breakpoints[0].id, id2);
        assert!(!d.remove_breakpoint(999));
    }

    #[test]
    fn test_hit_breakpoint() {
        let mut d = Debugger::new();
        let id = d.set_breakpoint("test.js", 10);
        let hit = d.hit_breakpoint("test.js", 10);
        assert!(hit.is_some());
        assert_eq!(hit.unwrap().hit_count, 1);
        let bp = d.breakpoints.iter().find(|b| b.id == id).unwrap();
        assert_eq!(bp.hit_count, 1);
    }

    #[test]
    fn test_hit_breakpoint_disabled() {
        let mut d = Debugger::new();
        d.set_breakpoint("test.js", 10);
        d.breakpoints[0].enabled = false;
        assert!(d.hit_breakpoint("test.js", 10).is_none());
    }

    #[tokio::test]
    async fn test_enable() {
        let mut d = Debugger::new();
        d.enable().await.unwrap();
    }

    #[tokio::test]
    async fn test_evaluate() {
        let d = Debugger::new();
        let val = d.evaluate("1+1", "frame-1").await.unwrap();
        assert_eq!(val["result"], "1+1");
        assert_eq!(val["frameId"], "frame-1");
    }
}
