# rtForth

Simple Forth implemented in Rust

## Design decisions:

* Safe first, performance later
* Token Threaded (Call threading), easy to implement in Rust
* Jit planned in the future to improve the performance

The performance of current implementation is not well because of token threading.
But slow colon definitions can be improved with a Just-In-Time compiler.
After optimization, corresponding slots in word list points to the jitted definitions.

## Usage

Install Rust: 

[Installing Rust](https://doc.rust-lang.org/book/installing-rust.html)

After installation of Rust:

```
$ cargo build --example rf
$ ./target/debug/examples/rf --help         # Display help information.
$ ./target/debug/examples/rf <file>         # Load forth commands in <file>.
$ ./target/debug/examples/rf lib.fs <file>  # Load lib.fs before <file>.
$ cargo build --release --example rf        # Compile optimized rtForth.
```

```
$ cargo run --examples rf              # Execute debug version of rtForth.
rtForth v0.1.7, Copyright (C) 2015 Mapacode Inc.
Type 'bye' or press Ctrl-D to exit.
: star 42 emit ;  ok
star * ok
star star star *** ok
bye 
```

2015/09/3

* ASUS X401A
* Ubuntu GNOME 14.04 LTS 32-bit
* rustc 1.4.0-nightly
* rtForth 0.1.8
* SwiftForth 3.5.7
* gforth 0.7.0
* gforth-fast 0.7.0

benchmark   | SwiftForth | gforth-fase |  gforth  | rtForth
----------- | ---------- | ----------- | -------- | -------
bubble-sort |    1       |     x       |     x    |     x
fib         |    1       |   6.0       |   9.2    |  30.6
matrix-mult |    1       |     x       |     x    |     x
mm-rtcg     |    1       |     x       |     x    |     x
sieve       |    1       |     x       |     x    |     x
ssieve-a    |    1       |     x       |     x    |     x

