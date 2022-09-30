use core::cell::UnsafeCell;
use core::fmt;
use core::ops::{Deref, DerefMut, Drop};
use core::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use crate::percore::{getcpu, is_mmu_ready, putcpu};
use crate::VMM;

#[repr(align(32))]
pub struct Mutex<T> {
    data: UnsafeCell<T>,
    lock: AtomicBool,
    owner: AtomicUsize,
}

unsafe impl<T: Send> Send for Mutex<T> {}

unsafe impl<T: Send> Sync for Mutex<T> {}

pub struct MutexGuard<'a, T: 'a> {
    lock: &'a Mutex<T>,
}

impl<'a, T> ! Send for MutexGuard<'a, T> {}

unsafe impl<'a, T: Sync> Sync for MutexGuard<'a, T> {}

impl<T> Mutex<T> {
    pub const fn new(val: T) -> Mutex<T> {
        Mutex {
            lock: AtomicBool::new(false),
            owner: AtomicUsize::new(usize::max_value()),
            data: UnsafeCell::new(val),
        }
    }
}

impl<T> Mutex<T> {
    pub fn try_lock(&self) -> Option<MutexGuard<T>> {
        let cpu = aarch64::affinity();
        if is_mmu_ready() {
            let ordering = Ordering::SeqCst;
            if !self.lock.swap(true, ordering) {
                self.owner.store(cpu, ordering);
                Some(MutexGuard { lock: &self })
            } else {
                None
            }
        } else if cpu == 0 && (!self.lock.load(Ordering::Relaxed) || self.owner.load(Ordering::Relaxed) == cpu) {
            self.lock.store(true, Ordering::Relaxed);
            self.owner.store(cpu, Ordering::Relaxed);
            Some(MutexGuard { lock: &self })
        } else {
            None
        }
    }

    // Once MMU/cache is enabled, do the right thing here. For now, we don't
    // need any real synchronization.
    #[inline(never)]
    pub fn lock(&self) -> MutexGuard<T> {
        // Wait until we can "aquire" the lock, then "acquire" it.
        loop {
            match self.try_lock() {
                Some(guard) => return guard,
                None => continue,
            }
        }
    }

    fn unlock(&self) {
        let ordering = if is_mmu_ready() {
            Ordering::SeqCst
        } else {
            Ordering::Relaxed
        };
        self.lock.store(false, ordering);
    }
}

impl<'a, T: 'a> Deref for MutexGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { &*self.lock.data.get() }
    }
}

impl<'a, T: 'a> DerefMut for MutexGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.lock.data.get() }
    }
}

impl<'a, T: 'a> Drop for MutexGuard<'a, T> {
    fn drop(&mut self) {
        self.lock.unlock()
    }
}

impl<T: fmt::Debug> fmt::Debug for Mutex<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.try_lock() {
            Some(guard) => f.debug_struct("Mutex").field("data", &&*guard).finish(),
            None => f.debug_struct("Mutex").field("data", &"<locked>").finish(),
        }
    }
}
