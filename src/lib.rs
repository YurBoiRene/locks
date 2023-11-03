use std::{
    marker::PhantomData,
    sync::{PoisonError, TryLockError},
    thread::{self, JoinHandle},
};

pub use std::sync::{LockResult, Mutex, MutexGuard, TryLockResult};

pub trait LockLevel {
    type Data;

    unsafe fn lock(&self) -> LockResult<MutexGuard<Self::Data>>;
    unsafe fn try_lock(&self) -> TryLockResult<MutexGuard<Self::Data>>;
}

pub trait LockLevelBelow<HigherLock: LockLevel> {}

pub trait Locks<Lock: LockLevel> {
    fn locks<T, F>(&mut self, other: &Lock, cb: F) -> T
    where
        F: FnOnce(Handle<Lock>, &mut Lock::Data) -> T;
}

pub struct Handle<T>(PhantomData<T>);

impl<T: LockLevel> Handle<T> {
    pub unsafe fn new(_: &T) -> Self {
        Self(PhantomData)
    }
}

#[must_use = "You must validate the integrity of the token"]
pub struct Token<T>(PhantomData<T>);

impl<T: LockLevel> Token<T> {
    pub unsafe fn new(_: &T) -> Self {
        Self(PhantomData)
    }
}

impl<HigherLock, LowerLock> Locks<LowerLock> for Handle<HigherLock>
where
    HigherLock: LockLevel,
    LowerLock: LockLevel + LockLevelBelow<HigherLock>,
{
    fn locks<T, F>(&mut self, child: &LowerLock, cb: F) -> T
    where
        F: FnOnce(Handle<LowerLock>, &mut LowerLock::Data) -> T,
    {
        let handle = unsafe { Handle::new(child) };
        let mut guard = unsafe { child.lock() }.unwrap(); // TODO: Error handling
        cb(handle, &mut guard)
    }
}

pub fn spawn<F, T, BaseLock>(base: &BaseLock, f: F) -> JoinHandle<T>
where
    BaseLock: LockLevel + Send + 'static,
    F: FnOnce(Handle<BaseLock>) -> T + Send + 'static,
    T: Send + 'static,
{
    let handle = unsafe { Handle::new(base) };

    thread::spawn(|| f(handle))
}

// TODO: temporary
#[macro_export]
macro_rules! define_level {
    ($name:ident) => {
        struct $name<T> {
            mutex: Mutex<T>,
        }

        impl<T> $name<T> {
            fn new(arg: T) -> Self {
                Self {
                    mutex: Mutex::new(arg),
                }
            }
        }

        impl<T> LockLevel for $name<T> {
            type Data = T;

            unsafe fn lock(&self) -> LockResult<MutexGuard<Self::Data>> {
                self.mutex.lock()
            }

            unsafe fn try_lock(&self) -> TryLockResult<MutexGuard<Self::Data>> {
                self.mutex.try_lock()
            }
        }
    };
}
