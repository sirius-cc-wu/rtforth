rtForth
=======

Simple Forth implemented in Rust

Design decisions:

* Token Threaded (Call threading), easy to implement in Rust
* Jit planned in the future to improve the performance

The performance of current implementation is not well because of token threading.
But slow colon definitions can be improved with a Just-In-Time compiler.
After optimization, corresponding slots in word list points to the jitted definitions.

Usage
=====

Install Rust compiler and cargo the Rust package manager at first.

```
$ cargo build
$ ./target/debug/rf --help   # Display help information.
$ ./target/debug/rf <file>   # Load forth commands in <file>.
$ ./target/debug/rf          # Load lib.fs in current directory.
$ cargo build --release      # Compile optimized rtForth.
$ ./target/release/rf        # Execute optimized rtForth and load lib.fs in current directory.
```

Benchmark
=========

See benchmarks in doc/bench/forth/.

2015/08/13

benchmark   | SwiftForth | gforth   | rtForth
----------- | ---------- | -------- | -------
bubble-sort |    1       |     x    |     x 
fib         |    1       |  7.32    | 28.82 
matrix-mult |    1       |     x    |
mm-rtcg     |    1       |     x    |
sieve       |    1       |     x    |
ssieve-a    |    1       |     x    |

