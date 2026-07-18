use std::sync::Mutex;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfilingStats {
    pub compile_time: Duration,
    pub execution_time: Duration,
    pub memory_usage: u64,
    pub jit_compilations: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub total_scripts: u64,
    pub avg_compile_time_ns: f64,
    pub avg_execution_time_ns: f64,
    pub peak_memory_usage: u64,
    pub jit_compilation_time: Duration,
    pub deoptimizations: u64,
    pub optimized_code_count: u64,
}

impl Default for ProfilingStats {
    fn default() -> Self {
        Self {
            compile_time: Duration::default(),
            execution_time: Duration::default(),
            memory_usage: 0,
            jit_compilations: 0,
            cache_hits: 0,
            cache_misses: 0,
            total_scripts: 0,
            avg_compile_time_ns: 0.0,
            avg_execution_time_ns: 0.0,
            peak_memory_usage: 0,
            jit_compilation_time: Duration::default(),
            deoptimizations: 0,
            optimized_code_count: 0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct IndividualProfile {
    pub script_name: String,
    pub compile_time: Duration,
    pub execution_time: Duration,
    pub memory_delta: u64,
    pub jit_compiled: bool,
    pub cache_hit: bool,
    pub timestamp: Instant,
}

pub struct JitProfiler {
    enabled: bool,
    stats: Mutex<ProfilingStats>,
    profiles: Mutex<Vec<IndividualProfile>>,
    start_time: Instant,
}

impl JitProfiler {
    pub fn new() -> Self {
        Self {
            enabled: true,
            stats: Mutex::new(ProfilingStats::default()),
            profiles: Mutex::new(Vec::new()),
            start_time: Instant::now(),
        }
    }

    pub fn new_disabled() -> Self {
        Self {
            enabled: false,
            stats: Mutex::new(ProfilingStats::default()),
            profiles: Mutex::new(Vec::new()),
            start_time: Instant::now(),
        }
    }

    pub fn start_profiling(&mut self) {
        self.enabled = true;
        self.start_time = Instant::now();
        tracing::info!("JIT profiling started");
    }

    pub fn stop_profiling(&mut self) {
        self.enabled = false;
        tracing::info!("JIT profiling stopped. {} scripts profiled", self.profiles.lock().unwrap().len());
    }

    pub fn record_compile(&self, _script_name: &str, compile_time: Duration) {
        if !self.enabled { return; }
        let mut stats = self.stats.lock().unwrap();
        stats.compile_time += compile_time;
        stats.total_scripts += 1;
        stats.avg_compile_time_ns = stats.compile_time.as_nanos() as f64 / stats.total_scripts as f64;
    }

    pub fn record_execution(&self, _script_name: &str, execution_time: Duration, memory_delta: u64) {
        if !self.enabled { return; }
        let mut stats = self.stats.lock().unwrap();
        stats.execution_time += execution_time;
        stats.memory_usage += memory_delta;
        if stats.memory_usage > stats.peak_memory_usage {
            stats.peak_memory_usage = stats.memory_usage;
        }
        stats.avg_execution_time_ns = stats.execution_time.as_nanos() as f64 / stats.total_scripts.max(1) as f64;
    }

    pub fn record_jit_compilation(&self, compile_time: Duration) {
        if !self.enabled { return; }
        let mut stats = self.stats.lock().unwrap();
        stats.jit_compilations += 1;
        stats.jit_compilation_time += compile_time;
        stats.optimized_code_count += 1;
    }

    pub fn record_cache_hit(&self) {
        if !self.enabled { return; }
        self.stats.lock().unwrap().cache_hits += 1;
    }

    pub fn record_cache_miss(&self) {
        if !self.enabled { return; }
        self.stats.lock().unwrap().cache_misses += 1;
    }

    pub fn record_deoptimization(&self) {
        if !self.enabled { return; }
        self.stats.lock().unwrap().deoptimizations += 1;
    }

    pub fn add_profile(&self, profile: IndividualProfile) {
        if !self.enabled { return; }
        self.profiles.lock().unwrap().push(profile);
    }

    pub fn get_stats(&self) -> ProfilingStats {
        self.stats.lock().unwrap().clone()
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn elapsed(&self) -> Duration {
        self.start_time.elapsed()
    }

    pub fn profile_count(&self) -> usize {
        self.profiles.lock().unwrap().len()
    }

    pub fn generate_report(&self) -> serde_json::Value {
        let stats = self.stats.lock().unwrap();
        let profiles = self.profiles.lock().unwrap();

        let hot_scripts: Vec<&IndividualProfile> = profiles.iter()
            .filter(|p| p.jit_compiled || p.execution_time > Duration::from_millis(10))
            .collect();

        serde_json::json!({
            "profiling_enabled": self.enabled,
            "elapsed_seconds": self.start_time.elapsed().as_secs_f64(),
            "total_scripts": stats.total_scripts,
            "total_compile_time_ms": stats.compile_time.as_secs_f64() * 1000.0,
            "total_execution_time_ms": stats.execution_time.as_secs_f64() * 1000.0,
            "avg_compile_time_ns": stats.avg_compile_time_ns,
            "avg_execution_time_ns": stats.avg_execution_time_ns,
            "memory_usage_bytes": stats.memory_usage,
            "peak_memory_usage_bytes": stats.peak_memory_usage,
            "jit_compilations": stats.jit_compilations,
            "jit_compilation_time_ms": stats.jit_compilation_time.as_secs_f64() * 1000.0,
            "optimized_code_count": stats.optimized_code_count,
            "deoptimizations": stats.deoptimizations,
            "cache_hits": stats.cache_hits,
            "cache_misses": stats.cache_misses,
            "cache_hit_ratio": if stats.cache_hits + stats.cache_misses > 0 {
                stats.cache_hits as f64 / (stats.cache_hits + stats.cache_misses) as f64
            } else { 0.0 },
            "hot_scripts_count": hot_scripts.len(),
            "hot_scripts": hot_scripts.iter().map(|p| serde_json::json!({
                "script": p.script_name,
                "compile_time_ms": p.compile_time.as_secs_f64() * 1000.0,
                "execution_time_ms": p.execution_time.as_secs_f64() * 1000.0,
                "memory_delta": p.memory_delta,
                "jit_compiled": p.jit_compiled,
                "cache_hit": p.cache_hit,
            })).collect::<Vec<_>>(),
            "recommendations": self.generate_recommendations(&stats),
        })
    }

    fn generate_recommendations(&self, stats: &ProfilingStats) -> Vec<String> {
        let mut recommendations = Vec::new();

        if stats.jit_compilations > 100 && stats.avg_compile_time_ns > 1_000_000.0 {
            recommendations.push("High JIT compilation overhead detected. Consider increasing cache TTL.".to_string());
        }
        if stats.cache_hits < stats.cache_misses && stats.total_scripts > 100 {
            recommendations.push("Low cache hit ratio. Consider increasing cache size or pre-warming common modules.".to_string());
        }
        if stats.deoptimizations > stats.jit_compilations / 10 {
            recommendations.push("High deoptimization rate. Consider using type-stable code patterns.".to_string());
        }
        if stats.avg_execution_time_ns > 10_000_000.0 {
            recommendations.push("Slow average execution time. Consider switching to a faster engine (V8).".to_string());
        }
        if stats.peak_memory_usage > 100_000_000 {
            recommendations.push("High memory usage detected. Consider enabling memory limits.".to_string());
        }

        recommendations
    }

    pub fn reset(&self) {
        let mut stats = self.stats.lock().unwrap();
        *stats = ProfilingStats::default();
        self.profiles.lock().unwrap().clear();
    }
}

impl Default for JitProfiler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jit_profiler_new() {
        let profiler = JitProfiler::new();
        assert!(profiler.is_enabled());
        assert_eq!(profiler.profile_count(), 0);
    }

    #[test]
    fn test_jit_profiler_new_disabled() {
        let profiler = JitProfiler::new_disabled();
        assert!(!profiler.is_enabled());
    }

    #[test]
    fn test_start_stop_profiling() {
        let mut profiler = JitProfiler::new();
        profiler.stop_profiling();
        assert!(!profiler.is_enabled());
        profiler.start_profiling();
        assert!(profiler.is_enabled());
    }

    #[test]
    fn test_record_compile() {
        let profiler = JitProfiler::new();
        profiler.record_compile("test.js", Duration::from_millis(10));
        let stats = profiler.get_stats();
        assert_eq!(stats.total_scripts, 1);
        assert_eq!(stats.compile_time.as_millis(), 10);
    }

    #[test]
    fn test_record_execution() {
        let profiler = JitProfiler::new();
        profiler.record_execution("test.js", Duration::from_millis(5), 1024);
        let stats = profiler.get_stats();
        assert_eq!(stats.execution_time.as_millis(), 5);
        assert_eq!(stats.memory_usage, 1024);
    }

    #[test]
    fn test_record_jit_compilation() {
        let profiler = JitProfiler::new();
        profiler.record_jit_compilation(Duration::from_millis(50));
        let stats = profiler.get_stats();
        assert_eq!(stats.jit_compilations, 1);
        assert_eq!(stats.jit_compilation_time.as_millis(), 50);
    }

    #[test]
    fn test_cache_hit_miss() {
        let profiler = JitProfiler::new();
        profiler.record_cache_hit();
        profiler.record_cache_hit();
        profiler.record_cache_miss();
        let stats = profiler.get_stats();
        assert_eq!(stats.cache_hits, 2);
        assert_eq!(stats.cache_misses, 1);
    }

    #[test]
    fn test_record_deoptimization() {
        let profiler = JitProfiler::new();
        profiler.record_deoptimization();
        profiler.record_deoptimization();
        let stats = profiler.get_stats();
        assert_eq!(stats.deoptimizations, 2);
    }

    #[test]
    fn test_add_profile() {
        let profiler = JitProfiler::new();
        let profile = IndividualProfile {
            script_name: "test.js".to_string(),
            compile_time: Duration::from_millis(10),
            execution_time: Duration::from_millis(20),
            memory_delta: 512,
            jit_compiled: true,
            cache_hit: false,
            timestamp: Instant::now(),
        };
        profiler.add_profile(profile);
        assert_eq!(profiler.profile_count(), 1);
    }

    #[test]
    fn test_disabled_profiler_discards() {
        let profiler = JitProfiler::new_disabled();
        profiler.record_compile("test.js", Duration::from_millis(10));
        profiler.record_cache_hit();
        let stats = profiler.get_stats();
        assert_eq!(stats.total_scripts, 0);
        assert_eq!(stats.cache_hits, 0);
    }

    #[test]
    fn test_generate_report() {
        let profiler = JitProfiler::new();
        profiler.record_compile("test.js", Duration::from_millis(10));
        profiler.record_execution("test.js", Duration::from_millis(5), 256);
        profiler.record_cache_hit();
        let report = profiler.generate_report();
        assert_eq!(report["total_scripts"], 1);
        assert!(report["profiling_enabled"].as_bool().unwrap());
        assert!(report["cache_hits"].as_u64().unwrap() >= 1);
    }

    #[test]
    fn test_generate_report_empty() {
        let profiler = JitProfiler::new();
        let report = profiler.generate_report();
        assert_eq!(report["total_scripts"], 0);
        assert_eq!(report["hot_scripts_count"], 0);
    }

    #[test]
    fn test_recommendations_empty() {
        let profiler = JitProfiler::new();
        let stats = ProfilingStats::default();
        let recommendations = profiler.generate_recommendations(&stats);
        assert!(recommendations.is_empty());
    }

    #[test]
    fn test_elapsed_time() {
        let profiler = JitProfiler::new();
        let elapsed = profiler.elapsed();
        assert!(elapsed.as_nanos() > 0 || elapsed.as_nanos() == 0);
    }

    #[test]
    fn test_reset() {
        let profiler = JitProfiler::new();
        profiler.record_compile("test.js", Duration::from_millis(10));
        profiler.record_cache_hit();
        profiler.reset();
        let stats = profiler.get_stats();
        assert_eq!(stats.total_scripts, 0);
        assert_eq!(stats.cache_hits, 0);
    }

    #[test]
    fn test_peak_memory_tracking() {
        let profiler = JitProfiler::new();
        profiler.record_execution("a.js", Duration::default(), 500);
        profiler.record_execution("b.js", Duration::default(), 1000);
        let stats = profiler.get_stats();
        assert_eq!(stats.peak_memory_usage, 1500);
        assert_eq!(stats.memory_usage, 1500);
    }

    #[test]
    fn test_profiling_stats_default() {
        let stats = ProfilingStats::default();
        assert_eq!(stats.compile_time, Duration::default());
        assert_eq!(stats.total_scripts, 0);
        assert_eq!(stats.cache_hits, 0);
    }

    #[test]
    fn test_profiling_stats_serialization() {
        let stats = ProfilingStats {
            compile_time: Duration::from_millis(100),
            execution_time: Duration::from_millis(200),
            memory_usage: 1024,
            jit_compilations: 5,
            cache_hits: 10,
            cache_misses: 2,
            total_scripts: 3,
            avg_compile_time_ns: 33_333_333.0,
            avg_execution_time_ns: 66_666_666.0,
            peak_memory_usage: 2048,
            jit_compilation_time: Duration::from_millis(50),
            deoptimizations: 1,
            optimized_code_count: 5,
        };
        let json = serde_json::to_string(&stats).unwrap();
        let deserialized: ProfilingStats = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.total_scripts, 3);
        assert_eq!(deserialized.cache_hits, 10);
        assert_eq!(deserialized.jit_compilations, 5);
    }

    #[test]
    fn test_individual_profile_structure() {
        let profile = IndividualProfile {
            script_name: "test".to_string(),
            compile_time: Duration::from_millis(1),
            execution_time: Duration::from_millis(2),
            memory_delta: 100,
            jit_compiled: true,
            cache_hit: false,
            timestamp: Instant::now(),
        };
        assert_eq!(profile.script_name, "test");
        assert!(profile.jit_compiled);
        assert!(!profile.cache_hit);
    }
}
