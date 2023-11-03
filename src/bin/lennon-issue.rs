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

    // let mut scam: Option<Handle<S<()>>> = None;
    // spawn(&*s, move |mut s_hdl| loop {
    //     s_hdl.locks(&*a, |mut a_hdl, a_data| {
    //         a_hdl.locks(&*b, |_, b_data| println!("{a_data}, {b_data}"));
    //     });
    // });

    spawn(s, move |s_hdl| loop {
        let mut scam: Option<&mut Handle<A<i32>>> = None;
        s_hdl.locks(&*a, |a_hdl, a_data| {
            a_hdl.locks(&*b, |_, b_data| {
                println!("{a_data}, {b_data}");
            });
            scam = Some(a_hdl);
        });

        scam.unwrap().locks(&*b, |b_hdl, b_data| {
            println!("Wacko");
            s_hdl.locks(&*a, |a_hdl, a_data| {
                println!("Wacko2");
            })
        });
    });

    spawn(s_clone, move |s_hdl| loop {
        s_hdl.locks(&*a_clone, |a_hdl, a_data| {
            a_hdl.locks(&*b_clone, |_, b_data| {
                println!("{a_data}, {b_data}");
            });
        });
    })
    .join()
    .unwrap()
}
