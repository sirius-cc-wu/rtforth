Safe
====

rtForth is implemented in Rust. Safe should be rtForth's first priority, not performance.

Because users can access Forth's data structure directly, rtForth should be
designed with the following goals in mind:

* Forth programmer cannot execute arbitray code from rtForth.
