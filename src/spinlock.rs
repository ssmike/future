use std::sync::atomic::{Ordering, AtomicBool};
use std::ops::{DerefMut, Deref};
use std::cell::UnsafeCell;
use std::option::Option;
use std::mem;

#[derive(Default)]
pub struct Spinlock<T> {
    locked: AtomicBool,
    data: UnsafeCell<T>,
    read_only: AtomicBool
}

unsafe impl<T: Send + Sync> Sync for Spinlock<T> {}
unsafe impl<T: Send> Send for Spinlock<T> {}

pub struct SpinlockGuard<'t, T: 't> {
    parent: &'t Spinlock<T>,
}

impl<'t, T: 't> Drop for SpinlockGuard<'t, T> {
    fn drop(self: &mut SpinlockGuard<'t, T>) {
        self.parent.locked.store(false, Ordering::SeqCst);
    }
}

impl<'t, T: 't> Deref for SpinlockGuard<'t, T> {
    type Target = T;

    fn deref(self: &SpinlockGuard<'t, T>) -> &T {
        unsafe {mem::transmute(self.parent.data.get())}
    }
}

impl<'t, T: 't> DerefMut for SpinlockGuard<'t, T> {
    fn deref_mut(self: &mut SpinlockGuard<'t, T>) -> &mut T {
        unsafe {mem::transmute(self.parent.data.get())}
    }
}

impl<T> Spinlock<T> {
    pub fn new(value: T) -> Spinlock<T> {
        Spinlock {
            locked: AtomicBool::new(false),
            read_only: AtomicBool::new(false),
            data: UnsafeCell::from(value)
        }
    }

    fn read_only(self: &Spinlock<T>) -> bool {
        self.read_only.load(Ordering::SeqCst)
    }

    fn take(self: &Spinlock<T>) -> bool {
        while !self.locked.compare_and_swap(false, true, Ordering::SeqCst) {
            if self.read_only() {
                return false;
            }
        }
        true
    }

    pub fn lock<'t>(self: &'t Spinlock<T>) -> Option<SpinlockGuard<'t, T>> {
        if self.take() {
            Some(SpinlockGuard{parent: self})
        } else {
            None
        }
    }

    pub fn share(self: &Spinlock<T>) -> &T {
        if !self.read_only() {
            self.take();
            self.read_only.store(true, Ordering::SeqCst)
        }
        unsafe {mem::transmute(self.data.get())}
    }
}
