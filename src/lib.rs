use std::{
    marker::PhantomData,
    sync::{PoisonError, TryLockError},
};

pub use std::sync::{LockResult, Mutex, MutexGuard, TryLockResult};

pub trait LockLevel {
    type Data;

    unsafe fn lock(&self) -> LockResult<MutexGuard<Self::Data>>;
    unsafe fn try_lock(&self) -> TryLockResult<MutexGuard<Self::Data>>;
}

pub trait LockLevelBelow<HigherLock: LockLevel> {}

pub trait Locks<Lock: LockLevel> {
    fn locks<T, F>(self, other: &Lock, cb: F) -> T
    where
        F: FnOnce(Handle<Lock>, &mut Lock::Data) -> T;

    fn call<T>(&self, ret: (Token<Lock>, T)) -> T;
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
    fn locks<T, F>(self, child: &LowerLock, cb: F) -> T
    where
        F: FnOnce(Handle<LowerLock>, &mut LowerLock::Data) -> T,
    {
        let handle = unsafe {Handle::new(child)};
        let mut guard = unsafe { child.lock() }.unwrap(); // TODO: Error handling
        cb(handle, &mut guard)
    }

    fn call<T>(&self, ret: (Token<LowerLock>, T)) -> T {
        ret.1
    }
}

trait UseOnlyOnce {} // TODO?

#[macro_export]
macro_rules! baselock2 {
    (let ($token:ident, $handle:ident, $data:ident) = lock($a:expr)) => {
        let $token = unsafe { Token::new($a) };
        let $handle = unsafe { Handle::new($a) };
        let $data = unsafe { $a.lock() };
        macro_rules! baselock2 {
            () => {};
        }
    };

    // Convenience macros for dealing w/ the Result
    (let ($token:ident, $handle:ident, $data:ident) = lock($a:expr)?) => {
        baselock2!(let ($token, $handle, $data) = lock($a));
        let $data = $data?;
    };
    (let ($token:ident, $handle:ident, $data:ident) = lock($a:expr).unwrap()) => {
        baselock2!(let ($token, $handle, $data) = lock($a));
        let $data = $data.unwrap();
    };
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
