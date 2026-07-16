use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CdpMessage {
    pub id: u64,
    pub method: String,
    #[serde(default)]
    pub params: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebuggerDomain {
    pub methods: Vec<String>,
    pub events: Vec<String>,
}

impl Default for DebuggerDomain {
    fn default() -> Self {
        Self {
            methods: vec![
                "enable".into(),
                "disable".into(),
                "setBreakpoint".into(),
                "removeBreakpoint".into(),
                "stepOver".into(),
                "stepInto".into(),
                "stepOut".into(),
                "continue".into(),
                "pause".into(),
                "resume".into(),
                "evaluate".into(),
                "getScriptSource".into(),
            ],
            events: vec![
                "paused".into(),
                "resumed".into(),
                "scriptParsed".into(),
                "breakpointResolved".into(),
            ],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PausedEvent {
    pub call_frames: Vec<CallFrame>,
    pub reason: String,
    pub hit_breakpoints: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallFrame {
    pub call_frame_id: String,
    pub function_name: String,
    pub location: Location,
    pub scope_chain: Vec<Scope>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Location {
    pub script_id: String,
    pub line_number: u32,
    pub column_number: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scope {
    pub scope_type: String,
    pub name: Option<String>,
    pub object: serde_json::Value,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cdp_message() {
        let msg = CdpMessage {
            id: 1,
            method: "Debugger.enable".into(),
            params: serde_json::json!({}),
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("Debugger.enable"));
        let deserialized: CdpMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.id, 1);
    }

    #[test]
    fn test_paused_event() {
        let event = PausedEvent {
            call_frames: vec![CallFrame {
                call_frame_id: "frame-0".into(),
                function_name: "testFunc".into(),
                location: Location {
                    script_id: "1".into(),
                    line_number: 10,
                    column_number: Some(5),
                },
                scope_chain: vec![Scope {
                    scope_type: "local".into(),
                    name: Some("test".into()),
                    object: serde_json::json!({"x": 42}),
                }],
            }],
            reason: "breakpoint".into(),
            hit_breakpoints: vec!["1".into()],
        };
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("breakpoint"));
        assert!(json.contains("testFunc"));
        assert!(json.contains("frame-0"));

        let deserialized: PausedEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.reason, "breakpoint");
        assert_eq!(deserialized.call_frames.len(), 1);
        assert_eq!(deserialized.call_frames[0].function_name, "testFunc");
        assert_eq!(deserialized.hit_breakpoints, vec!["1"]);
    }

    #[test]
    fn test_location() {
        let loc = Location {
            script_id: "script-1".into(),
            line_number: 42,
            column_number: None,
        };
        let json = serde_json::to_string(&loc).unwrap();
        assert!(json.contains("script-1"));
        let deserialized: Location = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.script_id, "script-1");
        assert_eq!(deserialized.line_number, 42);
        assert!(deserialized.column_number.is_none());
    }

    #[test]
    fn test_scope() {
        let scope = Scope {
            scope_type: "global".into(),
            name: None,
            object: serde_json::json!({"window": {}}),
        };
        let json = serde_json::to_string(&scope).unwrap();
        assert!(json.contains("global"));
        let deserialized: Scope = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.scope_type, "global");
        assert!(deserialized.name.is_none());
    }

    #[test]
    fn test_debugger_domain_default() {
        let domain = DebuggerDomain::default();
        assert!(domain.methods.contains(&"enable".to_string()));
        assert!(domain.methods.contains(&"setBreakpoint".to_string()));
        assert!(domain.events.contains(&"paused".to_string()));
        assert!(domain.events.contains(&"resumed".to_string()));
    }
}
