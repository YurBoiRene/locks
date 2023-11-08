use std::{
    marker::PhantomData,
    thread::{self, JoinHandle},
};

pub use std::sync::{LockResult, Mutex, MutexGuard, TryLockResult};

pub trait LockLevel {
    type Data;

    unsafe fn lock(&self) -> LockResult<MutexGuard<Self::Data>>;
    unsafe fn try_lock(&self) -> TryLockResult<MutexGuard<Self::Data>>;
}

pub trait LockLevelBelow<HigherLock> {}

pub trait Locks<Lock: LockLevel> {
    fn locks<T, F>(&mut self, other: &Lock, cb: F) -> T
    where
        F: FnOnce(&mut Handle<Lock>, &mut Lock::Data) -> T;
}

pub struct Handle<T>(PhantomData<T>);

impl<T> Handle<T> {
    pub unsafe fn new(_: &T) -> Self {
        Self(PhantomData)
    }
}

impl<HigherLock, LowerLock> Locks<LowerLock> for Handle<HigherLock>
where
    LowerLock: LockLevel + LockLevelBelow<HigherLock>,
{
    fn locks<T, F>(&mut self, child: &LowerLock, cb: F) -> T
    where
        F: FnOnce(&mut Handle<LowerLock>, &mut LowerLock::Data) -> T,
    {
        let mut handle = unsafe { Handle::new(child) };
        let mut guard = unsafe { child.lock() }.unwrap(); // TODO: Error handling
        cb(&mut handle, &mut guard)
    }
}

pub struct LockedJoinHandle<Level, T> {
    join_handle: JoinHandle<T>,
    level_handle: PhantomData<Level>,
}

impl<Level, T> LockedJoinHandle<Level, T> {
    pub fn new(_: &Level, join_handle: JoinHandle<T>) -> Self {
        LockedJoinHandle {
            join_handle,
            level_handle: PhantomData,
        }
    }

    pub fn join(self, _: &mut Handle<Level>) -> std::thread::Result<T> {
        self.join_handle.join()
    }
}

pub fn spawn<F, T, BaseLock>(base: &BaseLock, f: F) -> LockedJoinHandle<BaseLock, T>
where
    F: FnOnce(&mut Handle<BaseLock>) -> T + Send + 'static,
    T: Send + 'static,
{
    let join_hdl = thread::spawn(move || {
        let mut handle = Handle(PhantomData::<BaseLock>);
        f(&mut handle)
    });
    LockedJoinHandle::new(base, join_hdl)
}

pub struct MainLevel;

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

        impl<T> LockLevelBelow<MainLevel> for $name<T> {}
    };
}
