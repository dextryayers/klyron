use std::alloc::{alloc, dealloc, Layout};
use std::cell::UnsafeCell;
use std::marker::PhantomData;
use std::mem;
use std::ptr::NonNull;

const DEFAULT_CHUNK_SIZE: usize = 64 * 1024;

struct Chunk {
    ptr: NonNull<u8>,
    cap: usize,
    pos: usize,
}

impl Chunk {
    fn new(size: usize) -> Self {
        let layout = Layout::from_size_align(size, mem::align_of::<u64>()).unwrap();
        let ptr = NonNull::new(unsafe { alloc(layout) }).expect("Arena allocation failed");
        Self { ptr, cap: size, pos: 0 }
    }

    fn remaining(&self) -> usize {
        self.cap - self.pos
    }

    unsafe fn alloc_bytes(&mut self, size: usize, align: usize) -> NonNull<u8> {
        let align = align.max(mem::align_of::<u64>());
        let start = self.ptr.as_ptr() as usize + self.pos;
        let misalignment = (align - (start % align)) % align;
        let start_aligned = start + misalignment;
        let end = start_aligned + size;
        if end > self.ptr.as_ptr() as usize + self.cap {
            panic!("Arena chunk out of memory");
        }
        self.pos = end - self.ptr.as_ptr() as usize;
        NonNull::new_unchecked(start_aligned as *mut u8)
    }
}

impl Drop for Chunk {
    fn drop(&mut self) {
        let layout = Layout::from_size_align(self.cap, mem::align_of::<u64>()).unwrap();
        unsafe { dealloc(self.ptr.as_ptr(), layout) }
    }
}

pub struct Arena {
    chunks: UnsafeCell<Vec<Chunk>>,
    chunk_size: usize,
}

unsafe impl Send for Arena {}
unsafe impl Sync for Arena {}

impl Arena {
    pub fn new() -> Self {
        Self { chunks: UnsafeCell::new(Vec::new()), chunk_size: DEFAULT_CHUNK_SIZE }
    }

    pub fn with_chunk_size(size: usize) -> Self {
        Self { chunks: UnsafeCell::new(Vec::new()), chunk_size: size }
    }

    pub fn alloc<T>(&self, val: T) -> &mut T {
        let layout = Layout::new::<T>();
        let ptr = self.alloc_raw(layout);
        unsafe {
            let p = ptr.as_ptr() as *mut T;
            p.write(val);
            &mut *p
        }
    }

    pub fn alloc_slice<T: Copy>(&self, vals: &[T]) -> &mut [T] {
        if vals.is_empty() {
            return &mut [];
        }
        let layout = Layout::array::<T>(vals.len()).unwrap();
        let ptr = self.alloc_raw(layout);
        unsafe {
            std::ptr::copy_nonoverlapping(vals.as_ptr(), ptr.as_ptr() as *mut T, vals.len());
            std::slice::from_raw_parts_mut(ptr.as_ptr() as *mut T, vals.len())
        }
    }

    pub fn alloc_str(&self, s: &str) -> &mut str {
        let bytes = self.alloc_slice(s.as_bytes());
        unsafe { std::str::from_utf8_unchecked_mut(bytes) }
    }

    fn alloc_raw(&self, layout: Layout) -> NonNull<u8> {
        let chunks = unsafe { &mut *self.chunks.get() };
        let size = layout.size();
        let align = layout.align();
        if let Some(chunk) = chunks.last_mut() {
            if chunk.remaining() >= size {
                return unsafe { chunk.alloc_bytes(size, align) };
            }
        }
        let chunk_size = self.chunk_size.max(size + align);
        let mut chunk = Chunk::new(chunk_size);
        let ptr = unsafe { chunk.alloc_bytes(size, align) };
        chunks.push(chunk);
        ptr
    }

    pub fn reset(&self) {
        let chunks = unsafe { &mut *self.chunks.get() };
        chunks.clear();
    }

    pub fn len(&self) -> usize {
        let chunks = unsafe { &*self.chunks.get() };
        chunks.iter().map(|c| c.pos).sum()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl Drop for Arena {
    fn drop(&mut self) {
        self.chunks.get_mut().clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_alloc_int() {
        let arena = Arena::new();
        let val = arena.alloc(42i32);
        assert_eq!(*val, 42);
    }

    #[test]
    fn test_alloc_slice() {
        let arena = Arena::new();
        let slice = arena.alloc_slice(&[1, 2, 3, 4, 5]);
        assert_eq!(slice, &[1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_alloc_str() {
        let arena = Arena::new();
        let s = arena.alloc_str("hello arena");
        assert_eq!(s, "hello arena");
    }

    #[test]
    fn test_reset() {
        let arena = Arena::new();
        arena.alloc(42i32);
        assert!(arena.len() > 0);
        arena.reset();
        assert_eq!(arena.len(), 0);
    }

    #[test]
    fn test_multiple_allocs() {
        let arena = Arena::new();
        for i in 0..1000 {
            let val = arena.alloc(i);
            assert_eq!(*val, i);
        }
    }
}
