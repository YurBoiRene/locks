# Deadlock Prevention via Rust's Type System

This was a project made for CSE 5349 (Rustworthy Computing) at The Ohio State University by Lennon Anderson, Adrian Vovk, Kyle Rosenberg, and Chris Barlas.

Deadlocks are a common problem in multiprocessing systems that occur when a thread of execution is waiting for the lock of a resource held by another thread of execution, which is in turn, waiting for a resource held by the first thread. For more information on deadlocks, we refer the reader to [Section 32.3 of *Operating Systems: Three Easy Pieces*](https://pages.cs.wisc.edu/~remzi/OSTEP/threads-bugs.pdf).

We introduce `locks`, a crate providing a lock types that ensure deadlock free execution verified at compile time. This is done by enforcing that each thread takes locks in a consistent ordering.

Examples can be found in `src/bin/`. To run an example (e.g. combined-accounts):

```sh
cargo run --bin combined-accounts
```

For information on each example, check out the doc at the top of the file.

## Programming Model/How to Use

First, a special main function is needed to give us our `MainLevel` handle. The `MainLevel` handle is the highest "lock" possible (can lock any lock level) and holds no data. It will be used to lock lower locks containing data and join spawned threads that utilize locks.

The `locks::main` attribute adds the `MainLevel` handle variable `main` to the scope.

```rust
use locks::prelude::*;

#[locks::main]
fn main() {
    // Rest of main...
}
```

Then we declare a set of locks with a total order.

```rust
define_level!(A);
define_level!(B);
order_level!(B < A);
```

We can then make new locks containing the accessed data. We also wrap them in Arcs to clone and send them to spawned threads.

```rust
let a = Arc::new(A::new(10));
let b = Arc::new(B::new(2));

let a_clone = Arc::clone(&a);
let b_clone = Arc::clone(&b);
```

To extract data from a lock we can use the `with` function implemented on handles. Given a lock level **below** the held handle with a closure, `with` will run the closure with a handle to the taken lock and a mutable reference to its held data.

```rust
main.with(&*a, |a_hdl, a_data| {
    // Do stuff with a's data
    *a_data += 1;
});
```

You can also take any lock below the handle's lock.

```rust
main.with(&*a, |a_hdl, a_data| { *a_data += 1 }); // Fine
main.with(&*b, |b_hdl, b_data| { *b_data += 1 }); // Fine
a.with(&*b, |b_hdl, b_data| { *b_data += 1 }); // Fine
b.with(&*a, |a_hdl, a_data| { *a_data += 1 }); // Will not compile
```

Handles are always given as mutable references and so cannot be sent to threads directly. To solve this there is `spawn()`.

To create a thread with the use of deadlock free locks use `spawn`. In this code snippet the thread `t1` is created with a level of `MainLevel`. This level information is kept in t1's type. `spawn` also gives a handle of the requested lock to the closure to lock locks below itself.

```rust
let t1 = spawn(&MainLevel, move |main_hdl| loop {
    // Take locks in spawned thread
    main.with(&*a, |a_hdl, a_data| { *a_data += 1 });

    // Do more thread tasks...
});
```

Threads in locks can be joined using `join` by passing a handle to the lock level of the thread.

```rust
t1.join(main).unwrap();
```

An important detail here is that a thread can be spawned with **any** lock level, no matter what locks are held. It is only on joining that the lock of the same level as the thread must be held.

## Previous Work

Our solution is based on [*A Type System for Preventing Data Races and Deadlocks in Java Programs*](#references). Boyapati et al. propose a static type system that guarantees programs are free of data races and deadlocks using a partial order of locks. In addition to ensuring deadlock free execution, their type system allows a recursive tree-based data structure to define the partial order and be changed at runtime.

The core idea involves invalidating "Circular wait": one of the four qualities a program must have to deadlock.

> **Circular wait:** There exists a circular chain of threads such that each thread holds one or more resources (e.g., locks) that are being re-quested by the next thread in the chain

*(Arpaci-Dusseau 2019, 455)*

Invalidating circular wait is done by defining a partial order of locks and enforcing that a thread holding more than one lock must acquire them in descending order.

The type system proposed in Boyapati et al. is implemented for Java and requires a modification to the Java type system. Modifying the core language is not ideal as it makes it difficult to adapt to updates in the core language and slows widespread adoption from users.

Rust's rich type system makes a prime candidate for implementing the system proposed in Boyapati et al. Additionally, Java is not purely statically typed and presents challenges to enforcing a static type system. The completely static type system of Rust on the other hand makes it ideal for implementing this system.

## Our Work

We implement a system to define a total order of lock levels as well as methods to acquire their locks. Using Rust's type system, we enforce that the program will not compile unless the locks are taken in descending order.

## Limitations

Our system requires all lock levels to be statically defined and cannot be modified at runtime like Boyapati et al.

## Future Work

Our system requires manually defined lock levels and lock level relationships that become exponentially verbose as the number of lock levels increases. The burden on the programmer of creating all these entries can be relieved by implementing a macro to generate them automatically given a list of levels.

Our system should be polished and packaged as a crate for easy use and distribution.

## References

Operating Systems: Three Easy Pieces. Remzi H. Arpaci-Dusseau and Andrea C. Arpaci-Dusseau Arpaci-Dusseau Books. August, 2018 (Version 1.00) or July, 2019 (Version 1.01) http://www.ostep.org

“A Type System for Preventing Data Races and Deadlocks in Java Programs”. Chandrasekhar Boyapati, Robert Lee, Martin Rinard. Laboratory for Computer Science, Massachusetts Institute of Technology. March, 2002.
