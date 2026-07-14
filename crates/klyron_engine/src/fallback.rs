use std::time::Instant;

use crate::engine::{JsEngineKind, EngineRuntime};

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

    pub fn resolve(&mut self) -> Result<EngineRuntime, String> {
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

        Err("No JavaScript engine available after fallback chain".to_string())
    }

    pub fn resolve_with_timeout(&mut self, timeout: std::time::Duration) -> Result<EngineRuntime, String> {
        let start = Instant::now();
        let candidates = self.candidates();
        for kind in candidates {
            if start.elapsed() > timeout {
                return Err("Fallback chain timed out".to_string());
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
        Err("No JavaScript engine available".to_string())
    }

    pub fn benchmark_and_select(&mut self) -> Result<JsEngineKind, String> {
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
            None => Err("No engine available for benchmarking".to_string()),
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
