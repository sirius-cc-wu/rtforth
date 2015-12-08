Safe
====

rtForth is implemented in Rust. Safe should be rtForth's first priority, not performance.

Because users can access Forth's data structure directly, rtForth should be
designed with the following goals in mind:

* Forth programmer cannot execute arbitray code from rtForth.
=> ! should not work on arbitrary memory.
=> execute should not work on arbitrary memory.
=> seperation of data and code.
=> no execution vectors in mutable data area.
=> return stack only used for fuction return.
