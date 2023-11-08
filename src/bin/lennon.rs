use locks::*;
use std::sync::Arc;

define_level!(A);
define_level!(B);
impl<T, U> LockLevelBelow<A<T>> for B<U> {}

fn main() {
    let a = Arc::new(A::new(10));
    let b = Arc::new(B::new(2));

    let a_clone = Arc::clone(&a);
    let b_clone = Arc::clone(&b);

    

    spawn(s, move |s_hdl| loop {
        s_hdl.locks(&*a, |a_hdl, a_data| {
            a_hdl.locks(&*b, |_, b_data| {
                *a_data += 1;
                println!("{a_data}, {b_data}");
            });
        });
    });

    unsafe {
        spawn(s_clone, move |s_hdl| loop {
            s_hdl.locks(&*a_clone, |a_hdl, a_data| {
                a_hdl.locks(&*b_clone, |_, b_data| {
                    *b_data += 1;
                    println!("{a_data}, {b_data}");
                });
            });
        })
        .raw_join()
        .unwrap();
    }
}
