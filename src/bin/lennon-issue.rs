use locks::*;
use std::sync::Arc;

define_level!(A);
define_level!(B);
impl<T, U> LockLevelBelow<A<T>> for B<U> {}

fn main() {
    let main = &mut unsafe { Handle::new(&MainLevel) }; // TODO

    let a = Arc::new(A::new(10));
    let b = Arc::new(B::new(2));

    let a_clone = Arc::clone(&a);
    let b_clone = Arc::clone(&b);

    let t1 = spawn(&MainLevel, move |hdl| loop {
        hdl.locks(&*a, |hdl, a_data| {
            hdl.locks(&*b, |hdl, b_data| {
                *a_data += 1;
                println!("t1: {a_data}, {b_data}");
            });
        });
    });

    let t2 = spawn(&MainLevel, move |hdl| loop {
        hdl.locks(&*b_clone, |hdl, a_data| {
            hdl.locks(&*a_clone, |hdl, b_data| {
                *b_data += 1;
                println!("t2: {a_data}, {b_data}");
            })
        })
    });

    t1.join(main).unwrap();
    t2.join(main).unwrap();
}
