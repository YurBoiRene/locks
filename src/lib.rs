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
    fn lock(self, other: &Lock) -> LockResult<(Handle<Lock>, MutexGuard<Lock::Data>)>;
    fn try_lock(self, other: &Lock) -> TryLockResult<(Handle<Lock>, MutexGuard<Lock::Data>)>;
    fn call<T>(&self, ret: (Token<Lock>, T)) -> T;

    fn with<T, F>(self, other: &Lock, cb: F) -> T
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
    fn lock(
        self,
        child: &LowerLock,
    ) -> LockResult<(Handle<LowerLock>, MutexGuard<LowerLock::Data>)> {
        let child_handle = unsafe { Handle::new(child) };
        let child_guard = unsafe { child.lock() };
        match child_guard {
            Ok(res) => Ok((child_handle, res)),
            Err(e) => Err(PoisonError::new((child_handle, e.into_inner()))),
        }
    }

    fn try_lock(
        self,
        child: &LowerLock,
    ) -> TryLockResult<(Handle<LowerLock>, MutexGuard<LowerLock::Data>)> {
        let child_handle = unsafe { Handle::new(child) };
        let child_guard = unsafe { child.try_lock() };
        match child_guard {
            Ok(res) => Ok((child_handle, res)),
            Err(TryLockError::Poisoned(e)) => {
                Err(PoisonError::new((child_handle, e.into_inner())).into())
            }
            Err(TryLockError::WouldBlock) => Err(TryLockError::WouldBlock),
        }
    }

    fn call<T>(&self, ret: (Token<LowerLock>, T)) -> T {
        ret.1
    }

    fn with<T, F>(self, other: &LowerLock, cb: F) -> T
    where
        F: FnOnce(Handle<LowerLock>, &mut LowerLock::Data) -> T,
    {
        let (hdl, mut guard) = self.lock(other).unwrap();
        cb(hdl, &mut guard)
    }
}

trait UseOnlyOnce {} // TODO?

#[macro_export]
macro_rules! baselock {
    (let ($handle:ident, $data:ident) = lock($a:expr)) => {
        let $handle = unsafe { Handle::new($a) };
        let $data = unsafe { $a.lock() };
        macro_rules! baselock {
            () => {};
        }
        //impl<T: LockLevel> UseOnlyOnce for T {}
        // TODO!!!! ^^^^^
        // Idea 1: Just take variable name idents and declare the vars in the macro
    };

    // Convenience macros for dealing w/ the Result
    (let ($handle:ident, $data:ident) = lock($a:expr)?) => {
        baselock!(let ($handle, $data) = lock($a));
        let $data = $data?;
    };
    (let ($handle:ident, $data:ident) = lock($a:expr).unwrap()) => {
        baselock!(let ($handle, $data) = lock($a));
        let $data = $data.unwrap();
    };
}

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
