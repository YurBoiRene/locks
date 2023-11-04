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

    let s_clone2 = Arc::clone(&s);
    let a_clone2 = Arc::clone(&a);
    let b_clone2 = Arc::clone(&b);

    // spawn(s, move |s_hdl| {
    //     s_hdl.locks(&*a, |a_hdl, a_data| {
    //         println!("a is locked: {a_data}");
    //         spawn(a_clone, move |a_hdl2| loop {
    //             a_hdl2.locks(&*b, |a_hdl, a_data| {
    //                 println!("A is locked {a_data}");
    //             });
    //         })
    //         .join(a_hdl);
    //     });
    // });

    spawn(s, move |s_hdl| {
        s_hdl.locks(&*a, |a_hdl, a_data| {
            println!("a is locked: {a_data}");
            spawn(a_clone, move |a_hdl2| loop {
                a_hdl2.locks(&*b, |a_hdl, a_data| {
                    println!("A is locked {a_data}");
                });
            })
            .join(a_hdl)
            .unwrap();
        });
    });

    unsafe {
        spawn(s_clone2, move |s_hdl| loop {
            s_hdl.locks(&*a_clone2, |a_hdl, a_data| {
                a_hdl.locks(&*b_clone2, |_, b_data| {
                    println!("{a_data}, {b_data}");
                });
            });
        })
        .raw_join()
        .unwrap()
    }
}
