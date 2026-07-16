use crate::process::EngineOutput;

pub trait EngineTrait {
    fn exec(&mut self, code: &str) -> anyhow::Result<EngineOutput>;
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockEngine;

    impl EngineTrait for MockEngine {
        fn exec(&mut self, code: &str) -> anyhow::Result<EngineOutput> {
            Ok(EngineOutput {
                stdout: code.to_string(),
                stderr: String::new(),
                exit_code: 0,
                result: "ok".to_string(),
            })
        }
    }

    #[test]
    fn test_engine_trait_trait_object() {
        let mut engine: Box<dyn EngineTrait> = Box::new(MockEngine);
        let output = engine.exec("1+1").unwrap();
        assert_eq!(output.stdout, "1+1");
        assert_eq!(output.exit_code, 0);
        assert_eq!(output.result, "ok");
    }

    #[test]
    fn test_engine_trait_empty_code() {
        let mut engine = MockEngine;
        let output = engine.exec("").unwrap();
        assert_eq!(output.stdout, "");
        assert!(output.stderr.is_empty());
    }

    #[test]
    fn test_engine_trait_output_roundtrip() {
        let mut engine = MockEngine;
        let output = engine.exec("test_code").unwrap();
        let json = serde_json::to_string(&output).unwrap();
        let deserialized: EngineOutput = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.stdout, "test_code");
        assert_eq!(deserialized.exit_code, 0);
    }
}
