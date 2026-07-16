#[derive(Debug, Clone)]
pub struct MemoryLimits {
    pub max_heap_size: Option<usize>,
    pub max_stack_size: Option<usize>,
    pub gc_threshold: Option<usize>,
    pub max_array_buffer_size: Option<usize>,
}

impl MemoryLimits {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_heap(mut self, bytes: usize) -> Self {
        self.max_heap_size = Some(bytes);
        self
    }

    pub fn with_stack(mut self, bytes: usize) -> Self {
        self.max_stack_size = Some(bytes);
        self
    }

    pub fn heap_size_mb(&self) -> Option<f64> {
        self.max_heap_size.map(|b| b as f64 / 1_048_576.0)
    }

    pub fn unlimited() -> Self {
        Self {
            max_heap_size: None,
            max_stack_size: None,
            gc_threshold: None,
            max_array_buffer_size: None,
        }
    }

    pub fn conservative() -> Self {
        Self {
            max_heap_size: Some(256 * 1_048_576),
            max_stack_size: Some(1024 * 1024),
            gc_threshold: Some(192 * 1_048_576),
            max_array_buffer_size: Some(64 * 1_048_576),
        }
    }

    pub fn restricted() -> Self {
        Self {
            max_heap_size: Some(32 * 1_048_576),
            max_stack_size: Some(256 * 1024),
            gc_threshold: Some(24 * 1_048_576),
            max_array_buffer_size: Some(8 * 1_048_576),
        }
    }
}

impl Default for MemoryLimits {
    fn default() -> Self {
        Self::conservative()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_limits_default() {
        let limits = MemoryLimits::default();
        assert_eq!(limits.max_heap_size, Some(256 * 1_048_576));
        assert_eq!(limits.max_stack_size, Some(1024 * 1024));
    }

    #[test]
    fn test_memory_limits_new() {
        let limits = MemoryLimits::new();
        assert_eq!(limits.max_heap_size, Some(256 * 1_048_576));
    }

    #[test]
    fn test_memory_limits_with_heap() {
        let limits = MemoryLimits::new().with_heap(128 * 1_048_576);
        assert_eq!(limits.max_heap_size, Some(128 * 1_048_576));
        assert_eq!(limits.max_stack_size, Some(1024 * 1024));
    }

    #[test]
    fn test_memory_limits_with_stack() {
        let limits = MemoryLimits::new().with_stack(512 * 1024);
        assert_eq!(limits.max_stack_size, Some(512 * 1024));
    }

    #[test]
    fn test_memory_limits_chaining() {
        let limits = MemoryLimits::new()
            .with_heap(64 * 1_048_576)
            .with_stack(256 * 1024);
        assert_eq!(limits.max_heap_size, Some(64 * 1_048_576));
        assert_eq!(limits.max_stack_size, Some(256 * 1024));
    }

    #[test]
    fn test_heap_size_mb() {
        let limits = MemoryLimits::new().with_heap(104_857_600);
        let mb = limits.heap_size_mb().unwrap();
        assert!((mb - 100.0).abs() < 0.1);
    }

    #[test]
    fn test_heap_size_mb_none() {
        let limits = MemoryLimits::unlimited();
        assert!(limits.heap_size_mb().is_none());
    }

    #[test]
    fn test_unlimited() {
        let limits = MemoryLimits::unlimited();
        assert!(limits.max_heap_size.is_none());
        assert!(limits.max_stack_size.is_none());
        assert!(limits.gc_threshold.is_none());
        assert!(limits.max_array_buffer_size.is_none());
    }

    #[test]
    fn test_conservative() {
        let limits = MemoryLimits::conservative();
        assert_eq!(limits.max_heap_size, Some(256 * 1_048_576));
        assert_eq!(limits.max_stack_size, Some(1024 * 1024));
        assert_eq!(limits.gc_threshold, Some(192 * 1_048_576));
        assert_eq!(limits.max_array_buffer_size, Some(64 * 1_048_576));
    }

    #[test]
    fn test_restricted() {
        let limits = MemoryLimits::restricted();
        assert_eq!(limits.max_heap_size, Some(32 * 1_048_576));
        assert_eq!(limits.max_stack_size, Some(256 * 1024));
        assert_eq!(limits.gc_threshold, Some(24 * 1_048_576));
        assert_eq!(limits.max_array_buffer_size, Some(8 * 1_048_576));
    }

    #[test]
    fn test_memory_limits_debug() {
        let limits = MemoryLimits::default();
        let debug = format!("{:?}", limits);
        assert!(debug.contains("max_heap_size"));
        assert!(debug.contains("max_stack_size"));
    }
}
