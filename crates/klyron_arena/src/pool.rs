use std::cell::UnsafeCell;
use std::marker::PhantomData;
use std::mem;

pub struct ObjectPool<T> {
    slots: UnsafeCell<Vec<Slot<T>>>,
    free_list: UnsafeCell<Vec<usize>>,
    alloc_count: UnsafeCell<usize>,
    recycle_count: UnsafeCell<usize>,
}

struct Slot<T> {
    data: mem::MaybeUninit<T>,
    in_use: bool,
}

unsafe impl<T: Send> Send for ObjectPool<T> {}
unsafe impl<T: Sync> Sync for ObjectPool<T> {}

impl<T> ObjectPool<T> {
    pub fn new(capacity: usize) -> Self {
        let mut slots = Vec::with_capacity(capacity);
        for _ in 0..capacity {
            slots.push(Slot { data: mem::MaybeUninit::uninit(), in_use: false });
        }
        let free_list: Vec<usize> = (0..capacity).rev().collect();
        Self {
            slots: UnsafeCell::new(slots),
            free_list: UnsafeCell::new(free_list),
            alloc_count: UnsafeCell::new(0),
            recycle_count: UnsafeCell::new(0),
        }
    }

    pub fn acquire(&self, init: T) -> PoolGuard<T> {
        let slots = unsafe { &mut *self.slots.get() };
        let free = unsafe { &mut *self.free_list.get() };

        let idx = if let Some(idx) = free.pop() {
            idx
        } else {
            let idx = slots.len();
            slots.push(Slot { data: mem::MaybeUninit::uninit(), in_use: false });
            idx
        };

        let slot = &mut slots[idx];
        slot.data.write(init);
        slot.in_use = true;

        unsafe {
            *self.alloc_count.get() += 1;
        }

        PoolGuard { pool: self as *const Self as *mut Self, idx, _phantom: PhantomData }
    }

    pub fn capacity(&self) -> usize {
        let slots = unsafe { &*self.slots.get() };
        slots.len()
    }

    pub fn available(&self) -> usize {
        let free = unsafe { &*self.free_list.get() };
        free.len()
    }

    pub fn allocated_count(&self) -> usize {
        unsafe { *self.alloc_count.get() }
    }

    pub fn recycled_count(&self) -> usize {
        unsafe { *self.recycle_count.get() }
    }

    pub fn in_use_count(&self) -> usize {
        self.capacity() - self.available()
    }
}

pub struct PoolGuard<T> {
    pool: *mut ObjectPool<T>,
    idx: usize,
    _phantom: PhantomData<T>,
}

impl<T> std::ops::Deref for PoolGuard<T> {
    type Target = T;
    fn deref(&self) -> &T {
        let slots = unsafe { &*self.pool }.slots.get();
        let slots = unsafe { &*slots };
        unsafe { slots[self.idx].data.assume_init_ref() }
    }
}

impl<T> std::ops::DerefMut for PoolGuard<T> {
    fn deref_mut(&mut self) -> &mut T {
        let slots = unsafe { &*self.pool }.slots.get();
        let slots = unsafe { &mut *slots };
        unsafe { slots[self.idx].data.assume_init_mut() }
    }
}

impl<T> Drop for PoolGuard<T> {
    fn drop(&mut self) {
        let pool = unsafe { &*self.pool };
        let slots = unsafe { &mut *pool.slots.get() };
        let free = unsafe { &mut *pool.free_list.get() };

        if self.idx < slots.len() {
            let slot = &mut slots[self.idx];
            if slot.in_use {
                unsafe {
                    slot.data.assume_init_drop();
                }
                slot.in_use = false;
                free.push(self.idx);
                unsafe {
                    *pool.recycle_count.get() += 1;
                }
            }
        }
    }
}

impl<T> Clone for PoolGuard<T>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        let val = (**self).clone();
        let pool = unsafe { &*self.pool };
        pool.acquire(val)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pool_acquire_recycle() {
        let pool = ObjectPool::new(4);
        assert_eq!(pool.available(), 4);

        {
            let guard = pool.acquire(42u32);
            assert_eq!(*guard, 42);
            assert_eq!(pool.available(), 3);
        }

        assert_eq!(pool.available(), 4);
        assert_eq!(pool.recycled_count(), 1);
    }

    #[test]
    fn test_pool_capacity_growth() {
        let pool = ObjectPool::new(2);
        let mut guards = Vec::new();
        for i in 0..10 {
            guards.push(pool.acquire(i));
        }
        assert_eq!(pool.capacity(), 10);
    }

    #[test]
    fn test_pool_mutate() {
        let pool = ObjectPool::new(1);
        let mut guard = pool.acquire(10u32);
        assert_eq!(*guard, 10);
        *guard = 20;
        assert_eq!(*guard, 20);
    }
}
