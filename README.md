rtForth
=======

Simple Forth implemented in Rust

Design decisions:

* Token Threaded (Call threading), easy to implement in Rust
* Jit planned in the future to improve the performance

The performance of current implementation is not well because of token threading.
But slow colon definitions can be improved with a Just-In_Time compiler.
After the optimization, the corresponding slots in word list points to the jitted definitions.


