# Deadlock Prevention via Rust's Type System

Deadlocks are a common problem in multiprocessing systems that occur when a thread of execution is waiting for the lock of a resource held by another thread of execution, which is in turn, waiting for a resource held by the first thread. For more information on deadlocks, we refer the reader to [Section 32.3 of *Operating Systems: Three Easy Pieces*](https://pages.cs.wisc.edu/~remzi/OSTEP/threads-bugs.pdf).

Our work introduces a type system in Rust that enforces deadlock free execution from compile time.

**TODO** Add instructions for running examples here.

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
