use locks::*;
use std::{sync::Arc, thread, ops::Deref};

define_level!(A);
define_level!(B);
define_level!(C);

impl<T, U> LockLevelBelow<A<T>> for B<U> {}

impl<T, U> LockLevelBelow<A<T>> for C<U> {}
impl<T, U> LockLevelBelow<B<T>> for C<U> {}

fn some_function<'a, 'b, T: LockLevel, U: LockLevel + LockLevelBelow<T>>(lock: &'a T, lock2: &'b U) -> (Token<T>, (MutexGuard<'a, T::Data>, MutexGuard<'b, U::Data>)) {
    baselock2!(let (t, hdl, guard1) = lock(lock).unwrap());
    let (_, guard2) = hdl.lock(lock2).unwrap();
    (t, (guard1, guard2))
}

fn main() {
    let a = Arc::new(A::new(String::from("foo")));
    let b = Arc::new(B::new(String::from("bar")));
    let c = Arc::new(C::new(String::from("baz")));

    let a_clone = Arc::clone(&a);
    let b_clone = Arc::clone(&b);
    let c_clone = Arc::clone(&c);

    thread::spawn(move || loop {
        baselock2!(let (_tok, hdl, a_val) = lock(&*a_clone).unwrap());

        hdl.with(&*b_clone, |hdl, b_val| {
            hdl.with(&*c_clone, |hdl, c_val| {
                println!("t1 {a_val} {b_val} {c_val}")
            });
        });
        
        //let (b_val, c_val) = hdl.call(some_function(&*b_clone, &*c_clone));

        //let (hdl, b_val2) = hdl.lock(&*b_clone).unwrap();
        //let b_val = 5;

        //println!("t1 {a_val} {b_val} {c_val} {b_val2}");
    });

    thread::spawn(move || loop {
        // No deadlock!)
        baselock!(let (a_hdl, a_val) = lock(&*a).unwrap());
        let (_, b_val) = a_hdl.lock(&*b).unwrap();

        // Deadlock, but no compile!
        // baselock!(let (b_hdl, b_val) = lock(&*b).unwrap());
        // let (_, a_val) = b_hdl.lock(&*a).unwrap();

        println!("t2 {a_val} {b_val}");
    })
    .join()
    .unwrap();
}
