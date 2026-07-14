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
