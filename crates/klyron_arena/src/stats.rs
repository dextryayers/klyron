use crate::alloc::Arena;
use crate::pool::ObjectPool;
use std::marker::PhantomData;
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct ArenaStats {
    pub used_bytes: usize,
    pub allocated_bytes: usize,
    pub chunk_count: usize,
    pub utilization: f64,
}

impl ArenaStats {
    pub fn from_arena(arena: &Arena) -> Self {
        Self {
            used_bytes: arena.len(),
            allocated_bytes: arena.allocated_bytes(),
            chunk_count: arena.chunk_count(),
            utilization: arena.utilization(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct PoolStats<T> {
    pub type_name: &'static str,
    pub capacity: usize,
    pub available: usize,
    pub in_use: usize,
    pub total_allocated: usize,
    pub total_recycled: usize,
    _phantom: PhantomData<T>,
}

impl<T> PoolStats<T> {
    pub fn from_pool(pool: &ObjectPool<T>) -> Self {
        Self {
            type_name: std::any::type_name::<T>(),
            capacity: pool.capacity(),
            available: pool.available(),
            in_use: pool.in_use_count(),
            total_allocated: pool.allocated_count(),
            total_recycled: pool.recycled_count(),
            _phantom: PhantomData,
        }
    }
}

#[derive(Debug, Clone)]
pub struct MemorySnapshot {
    pub timestamp: Instant,
    pub arena_stats: ArenaStats,
    pub pool_stats: Vec<PoolStatsEntry>,
}

#[derive(Debug, Clone)]
pub struct PoolStatsEntry {
    pub name: String,
    pub capacity: usize,
    pub in_use: usize,
    pub available: usize,
}

#[derive(Debug, Clone)]
pub struct AllocationProfile {
    pub total_allocations: u64,
    pub total_bytes: u64,
    pub peak_bytes: u64,
    pub avg_allocation_size: f64,
    pub duration: Duration,
}

pub struct Profiler {
    start: Instant,
    allocations: u64,
    bytes: u64,
    peak: u64,
}

impl Profiler {
    pub fn new() -> Self {
        Self {
            start: Instant::now(),
            allocations: 0,
            bytes: 0,
            peak: 0,
        }
    }

    pub fn record_allocation(&mut self, size: usize) {
        self.allocations += 1;
        self.bytes += size as u64;
        if self.bytes > self.peak {
            self.peak = self.bytes;
        }
    }

    pub fn record_deallocation(&mut self, size: usize) {
        self.bytes = self.bytes.saturating_sub(size as u64);
    }

    pub fn profile(&self) -> AllocationProfile {
        let elapsed = self.start.elapsed();
        AllocationProfile {
            total_allocations: self.allocations,
            total_bytes: self.bytes,
            peak_bytes: self.peak,
            avg_allocation_size: if self.allocations > 0 {
                self.bytes as f64 / self.allocations as f64
            } else {
                0.0
            },
            duration: elapsed,
        }
    }
}

impl Default for Profiler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arena_stats() {
        let arena = Arena::new();
        arena.alloc(42i32);
        let stats = ArenaStats::from_arena(&arena);
        assert!(stats.used_bytes >= 4);
        assert!(stats.allocated_bytes >= 64 * 1024);
        assert_eq!(stats.chunk_count, 1);
        assert!(stats.utilization > 0.0);
    }

    #[test]
    fn test_profiler() {
        let mut prof = Profiler::new();
        prof.record_allocation(100);
        prof.record_allocation(200);
        let p = prof.profile();
        assert_eq!(p.total_allocations, 2);
        assert_eq!(p.total_bytes, 300);
        assert!(p.avg_allocation_size > 0.0);
    }
}
