use locks::*;
use std::sync::Arc;

define_level!(S);
define_level!(A);
impl<T, U> LockLevelBelow<S<T>> for A<U> {}
define_level!(B);
impl<T, U> LockLevelBelow<A<T>> for B<U> {}
impl<T, U> LockLevelBelow<S<T>> for B<U> {}

fn main() {
    let s = Arc::new(S::new(()));
    let a = Arc::new(A::new(10));
    let b = Arc::new(B::new(2));

    let s_clone = Arc::clone(&s);
    let a_clone = Arc::clone(&a);
    let b_clone = Arc::clone(&b);

    spawn(s, move |mut s_hdl| loop {
        s_hdl.locks(&*a, |mut a_hdl, a_data| {
            a_hdl.locks(&*b, |_, b_data| println!("{a_data}, {b_data}"));
        });
    });

    spawn(s_clone, move |mut s_hdl| loop {
        s_hdl.locks(&*a_clone, |mut a_hdl, a_data| {
            a_hdl.locks(&*b_clone, |_, b_data| println!("{a_data}, {b_data}"));
        });
    })
    .join()
    .unwrap()
}
