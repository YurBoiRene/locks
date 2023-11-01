use locks::*;
use std::{sync::Arc, thread, ops::Deref};

define_level!(A);
define_level!(B);
impl<T, U> LockLevelBelow<A<T>> for B<U> {}

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
