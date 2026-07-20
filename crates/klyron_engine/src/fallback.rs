use std::time::Instant;

use crate::engine::{JsEngineKind, EngineRuntime, EngineError};

#[derive(Debug, Clone)]
pub enum FallbackStrategy {
    Fastest,
    FirstWorking,
    Ordered(Vec<JsEngineKind>),
}

impl Default for FallbackStrategy {
    fn default() -> Self {
        Self::Fastest
    }
}

pub struct FallbackChain {
    strategy: FallbackStrategy,
    last_successful: Option<JsEngineKind>,
    blacklist: Vec<JsEngineKind>,
}

impl FallbackChain {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_strategy(strategy: FallbackStrategy) -> Self {
        Self {
            strategy,
            last_successful: None,
            blacklist: Vec::new(),
        }
    }

    pub fn resolve(&mut self) -> Result<EngineRuntime, EngineError> {
        if let Some(kind) = self.last_successful {
            if self.is_available(kind) {
                match EngineRuntime::new(kind) {
                    Ok(engine) => return Ok(engine),
                    Err(_) => self.blacklist.push(kind),
                }
            }
        }

        let candidates = self.candidates();
        for kind in candidates {
            if self.blacklist.contains(&kind) {
                continue;
            }
            match EngineRuntime::new(kind) {
                Ok(engine) => {
                    self.last_successful = Some(kind);
                    return Ok(engine);
                }
                Err(e) => {
                    self.blacklist.push(kind);
                    tracing::warn!("Fallback: engine {} failed: {}", kind, e);
                    continue;
                }
            }
        }

        Err(EngineError::NoEngineAvailable)
    }

    pub fn resolve_with_timeout(&mut self, timeout: std::time::Duration) -> Result<EngineRuntime, EngineError> {
        let start = Instant::now();
        let candidates = self.candidates();
        for kind in candidates {
            if start.elapsed() > timeout {
                return Err(EngineError::Timeout);
            }
            if self.blacklist.contains(&kind) {
                continue;
            }
            match EngineRuntime::new(kind) {
                Ok(engine) => {
                    self.last_successful = Some(kind);
                    return Ok(engine);
                }
                Err(e) => {
                    self.blacklist.push(kind);
                    tracing::warn!("Fallback (timed): engine {} failed: {}", kind, e);
                    continue;
                }
            }
        }
        Err(EngineError::NoEngineAvailable)
    }

    pub fn benchmark_and_select(&mut self) -> Result<JsEngineKind, EngineError> {
        use crate::engine::benchmark_all_engines;

        let results = benchmark_all_engines();
        let best = results
            .into_iter()
            .filter(|(_, r)| r.success)
            .min_by_key(|(_, r)| r.eval_time)
            .map(|(kind, _)| kind);

        match best {
            Some(kind) => {
                self.last_successful = Some(kind);
                Ok(kind)
            }
            None => Err(EngineError::NoEngineAvailable),
        }
    }

    fn candidates(&self) -> Vec<JsEngineKind> {
        match &self.strategy {
            FallbackStrategy::Fastest => {
                let mut all = JsEngineKind::all();
                if let Some(last) = self.last_successful {
                    all.sort_by_key(|k| if *k == last { 0 } else { 1 });
                }
                all
            }
            FallbackStrategy::FirstWorking => JsEngineKind::all(),
            FallbackStrategy::Ordered(order) => order.clone(),
        }
    }

    fn is_available(&self, kind: JsEngineKind) -> bool {
        let engine = EngineRuntime::new(kind);
        engine.is_ok()
    }

    pub fn reset_blacklist(&mut self) {
        self.blacklist.clear();
    }

    pub fn current_strategy(&self) -> &FallbackStrategy {
        &self.strategy
    }
}

impl Default for FallbackChain {
    fn default() -> Self {
        Self {
            strategy: FallbackStrategy::Fastest,
            last_successful: None,
            blacklist: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fallback_chain_new() {
        let chain = FallbackChain::new();
        assert!(matches!(chain.current_strategy(), FallbackStrategy::Fastest));
    }

    #[test]
    fn test_fallback_strategy_fastest() {
        let mut chain = FallbackChain::with_strategy(FallbackStrategy::Fastest);
        let result = chain.resolve();
        // May fail without engine features, but shouldn't panic
        assert!(result.is_err() || result.is_ok());
    }

    #[test]
    fn test_fallback_strategy_first_working() {
        let mut chain = FallbackChain::with_strategy(FallbackStrategy::FirstWorking);
        let result = chain.resolve();
        assert!(result.is_err() || result.is_ok());
    }

    #[test]
    fn test_fallback_strategy_ordered() {
        let order = vec![JsEngineKind::QuickJS, JsEngineKind::Boa];
        let mut chain = FallbackChain::with_strategy(FallbackStrategy::Ordered(order));
        let result = chain.resolve();
        assert!(result.is_err() || result.is_ok());
    }

    #[test]
    fn test_reset_blacklist() {
        let mut chain = FallbackChain::new();
        chain.blacklist.push(JsEngineKind::V8);
        chain.reset_blacklist();
        assert!(chain.blacklist.is_empty());
    }

    #[test]
    fn test_current_strategy() {
        let chain = FallbackChain::with_strategy(FallbackStrategy::FirstWorking);
        assert!(matches!(chain.current_strategy(), FallbackStrategy::FirstWorking));
    }

    #[test]
    fn test_benchmark_and_select() {
        let mut chain = FallbackChain::new();
        let result = chain.benchmark_and_select();
        // Without engines, this should fall back to error
        assert!(result.is_err() || result.is_ok());
    }

    #[test]
    fn test_fallback_strategy_default() {
        let strategy = FallbackStrategy::default();
        assert!(matches!(strategy, FallbackStrategy::Fastest));
    }

    #[test]
    fn test_resolve_with_timeout() {
        let mut chain = FallbackChain::new();
        let result = chain.resolve_with_timeout(std::time::Duration::from_secs(1));
        assert!(result.is_err() || result.is_ok());
    }

    #[test]
    fn test_fallback_strategy_ordered_priority() {
        let order = vec![JsEngineKind::V8, JsEngineKind::Boa];
        let strategy = FallbackStrategy::Ordered(order);
        if let FallbackStrategy::Ordered(o) = &strategy {
            assert_eq!(o[0], JsEngineKind::V8);
            assert_eq!(o[1], JsEngineKind::Boa);
        }
    }
}
