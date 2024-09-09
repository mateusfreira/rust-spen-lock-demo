use std::ops::{Deref, DerefMut};
use std::thread;
use std::{cell::UnsafeCell, sync::atomic::AtomicBool};

fn main() {
    loop {
        let value_lock = NunSpinLock::new(Vec::new());
        thread::scope(|s| {
            s.spawn(|| {
                let mut value = value_lock.lock();
                value.push(1);
            });
            s.spawn(|| {
                let mut value = value_lock.lock();
                value.push(2);
                value.push(2);
            });
        });
        let value = value_lock.lock();
        println!("{}", format!("Value! {:?}", value.as_slice()));
        assert!(value.as_slice() == &[1, 2, 2] || value.as_slice() == &[2, 2, 1]);
        println!("All tests passed!");
        thread::sleep(std::time::Duration::from_millis(100));
    }
}

pub struct NunGuard<'a, T> {
    lock: &'a NunSpinLock<T>,
}

impl<'a, T> Drop for NunGuard<'a, T> {
    fn drop(&mut self) {
        // Safety: Guard existing means we have locked the lock
        self.lock
            .locked
            .store(false, std::sync::atomic::Ordering::Release);
    }
}

impl<T> Deref for NunGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &T {
        // Safety: Guard existing means we have locked the lock
        unsafe { &*self.lock.value.get() }
    }
}

impl<T> DerefMut for NunGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        // Safety: Guard existing means we have locked the lock
        unsafe { &mut *self.lock.value.get() }
    }
}

pub struct NunSpinLock<T> {
    locked: AtomicBool,
    value: UnsafeCell<T>,
}

unsafe impl<T> Sync for NunSpinLock<T> where T: Send {}

impl<T> NunSpinLock<T> {
    pub const fn new(value: T) -> Self {
        NunSpinLock {
            locked: AtomicBool::new(false),
            value: UnsafeCell::new(value),
        }
    }

    pub fn lock<'a>(&self) -> NunGuard<T> {
        while self.locked.swap(true, std::sync::atomic::Ordering::Acquire) {
            std::hint::spin_loop();
        }
        NunGuard { lock: self }
    }

    // Safety: The caller must the &T reference returned by lock() is gone before
    pub unsafe fn unlock(&self) {}
}
