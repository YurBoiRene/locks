# Deadlocks Demo

1. Deadlocking Code (Define the problem)
2. Solution (Our Library)
3. Implementation (lib.rs)
4. Baselock Issue & Next Steps





```
        // fn something(b: Handle<B>) -> (i32, PhantomData<A<i32>>) {
        //     baselock!(let (hdl, val) = lock(*a).unwrap() with b_handle);
        //     todo!();
        // }

        // sync(a)
        // a.sync(b)
        // // Do something
        // // b is dropped
        // // a is dropped


        // let stack = todo!();

        // let stack = stack.pop(held_lock);

        // let (retval,_) = something(b_handle);

        // let retval = call! { b_handle, something() }

```