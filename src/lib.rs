use std::{
    marker::PhantomData,
    sync::{LockResult, MutexGuard},
    thread::{self, JoinHandle},
};

pub mod prelude {
    pub use super::{
        define_level, order_level, Handle, LockLevel, LockLevelBelow, LockedJoinHandle, Locks,
        MainLevel,
    };
}

/// Trait marking a type as a lock level. It should most likely be created with [define_level!]
pub trait LockLevel {
    type Data;

    unsafe fn lock(&self) -> LockResult<MutexGuard<Self::Data>>;
}

/// Trait marking a lock as `<` another lock in the partial ordering. It should most likely
/// be implemented using [order_level!].
pub trait LockLevelBelow<HigherLock> {}

/// Trait enabling the type system to actually use the locks. It does not need to be implemented
/// by hand. It is implemented for all [Handle] in terms of LockLevelBelow.
pub trait Locks<Lock: LockLevel> {
    fn with<T, F>(&mut self, other: &Lock, cb: F) -> T
    where
        F: FnOnce(&mut Handle<Lock>, &mut Lock::Data) -> T;
}

/// Allows you to take any lock of lower lock level than that of the `T` in the handle.
/// Mutable references will be passed around to various functions taking locks to ensure
/// they are not duplicated allowing lock shenanigans.
pub struct Handle<T>(PhantomData<T>);

impl<T> Handle<T> {
    pub unsafe fn new(_: &T) -> Self {
        Self(PhantomData)
    }
}

/// Trait implementation allowing for all `Handle<HigherLevel>` to lock any `LockLevel``
/// marked as below `HigherLevel`.
impl<HigherLock, LowerLock> Locks<LowerLock> for Handle<HigherLock>
where
    LowerLock: LockLevel + LockLevelBelow<HigherLock>,
{
    fn with<T, F>(&mut self, child: &LowerLock, cb: F) -> T
    where
        F: FnOnce(&mut Handle<LowerLock>, &mut LowerLock::Data) -> T,
    {
        let mut handle = unsafe { Handle::new(child) };
        let mut guard = unsafe { child.lock() }.unwrap();
        cb(&mut handle, &mut guard)
    }
}

/// Protects thread joins from deadlocks by guarding them with a handle.
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

/// Creates a new thread that must be joined using a [Handle] >= `BaseLock`.
///
/// This construct exists to avoid deadlocking when joining threads.
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

/// Marker level used to be able to join threads from main. It does
/// hold any data. The first line of `fn main()` should be acquiring
/// a mutable reference to a [Handle] of this level. This action
/// should only be performed once in main.
///
/// ```rust
/// use locks::prelude::*;
/// fn main() {
///     /// Safety: Only constructed once.
///     let hdl = &mut unsafe { Handle::new(&MainLevel) };
///     // ...
/// }
/// ```
pub struct MainLevel;

/// Defines a lock level generic over `T` of the given name.
/// ```rust
/// use locks::prelude::*;
///
/// define_level!(A);
/// define_level!(B);
///
/// fn main() {
///     let a = A::new(1);
///     let b = B::new(1);
/// }
/// ```
#[macro_export]
macro_rules! define_level {
    ($name:ident) => {
        struct $name<T> {
            mutex: ::std::sync::Mutex<T>,
        }

        impl<T> $name<T> {
            fn new(arg: T) -> Self {
                Self {
                    mutex: ::std::sync::Mutex::new(arg),
                }
            }
        }

        impl<T> locks::LockLevel for $name<T> {
            type Data = T;

            unsafe fn lock(&self) -> ::std::sync::LockResult<::std::sync::MutexGuard<Self::Data>> {
                self.mutex.lock()
            }
        }

        impl<T> locks::LockLevelBelow<MainLevel> for $name<T> {}
    };
}

/// Defines the partial ordering for lock levels.
///
/// Note: Must be called for all sublevels as rust cannot
/// do transitive trait implementations.
///
/// ```rust
/// use locks::prelude::*;
///
/// define_level!(A);
/// define_level!(B);
/// define_level!(C);
///
/// order_level!(B < A);
/// order_level!(C < A);
/// order_level!(C < B);
///
/// fn main() {
///     let main = &mut unsafe { Handle::new(&MainLevel) };
///
///     let a = A::new(1);
///     let b = B::new(2);
///     let c = C::new(3);
///
///     main.with(&a, |hdl, a| {
///         hdl.with(&b, |hdl, b| {
///             hdl.with(&c, |_, c| {
///                 *c = 5;
///                 println!("{a} {b} {c}");
///             });
///         });
///
///         hdl.with(&c, |_, c| {
///             println!("{a} {c}");
///         });
///     });
/// }
#[macro_export]
macro_rules! order_level {
    ($lhs:ident < $rhs:ident) => {
        impl<T, U> locks::LockLevelBelow<$rhs<T>> for $lhs<U> {}
    };
}
