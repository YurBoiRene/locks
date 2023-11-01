use locks::*;
use std::{sync::Arc, thread};

define_level!(A);
define_level!(B);
impl<T, U> LockLevelBelow<A<T>> for B<U> {}

fn oops<T: LockLevel>(lock: &T) -> (Token<T>, MutexGuard<T::Data>) {
    baselock2!(let (tok, _hdl, val) = lock(lock).unwrap());
    (tok, val)
}

//#[deny(unsafe_op_in_unsafe_fn)]
//unsafe 
fn oops2<T: LockLevel>(lock: &T) -> MutexGuard<T::Data> {
    baselock2!(let (_t, _hdl, guard) = lock(lock).unwrap());
    guard
}

fn main() {
    let a = Arc::new(A::new(10));
    let b = Arc::new(B::new(2));

    let a_clone = Arc::clone(&a);
    let b_clone = Arc::clone(&b);

    thread::spawn(move || loop {
        baselock!(let (a_hdl, a_val) = lock(&*a_clone).unwrap());
        let (_, b_val) = a_hdl.lock(&*b_clone).unwrap();

        println!("t1 {a_val} {b_val}");
    });

    thread::spawn(move || loop {
        // Deadlock, but still compiles
        let b_val = oops(&*b);
        let a_val = oops(&*b);
        println!("t2 {a_val} {b_val}");
    })
    .join()
    .unwrap();
}
