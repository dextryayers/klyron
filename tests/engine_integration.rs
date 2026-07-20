/// Integration tests for klyron engines.

#[cfg(test)]
mod tests {
    use klyron_engine::JsEngineKind;

    #[test]
    fn test_engine_kind_all() {
        let kinds = JsEngineKind::all();
        assert!(!kinds.is_empty(), "At least one engine should be available");
        assert!(kinds.contains(&JsEngineKind::Boa), "Boa should always be available");
    }

    #[test]
    fn test_engine_kind_display() {
        assert_eq!(JsEngineKind::Boa.to_string(), "boa");
        assert_eq!(JsEngineKind::QuickJS.to_string(), "quickjs");
    }
}
