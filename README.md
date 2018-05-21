# rtForth

Forth implemented in Rust, designed for real-time applications.

## Design decisions:

* Safe first, performance later
* Token Threaded + Primitive-centric threaded + Subroutine-threaded (only for x86)

## Usage

Install Rust: 

[Installing Rust](https://doc.rust-lang.org/book/installing-rust.html)

After installation of Rust:

```
$ cargo build --example rf
$ ./target/debug/examples/rf --help         # Display help information.
$ ./target/debug/examples/rf <file>         # Load forth commands in <file>.
$ ./target/debug/examples/rf lib.fs <file>  # Load lib.fs before <file>.
$ cargo build --release --example rf        # Compile optimized token-threaded rtForth.
$ cargo build --example rf --release --features="subroutine-threaded"    # Compile optimized subroutine-threaded rtForth.
```

```
$ cargo run --example rf              # Execute debug version of rtForth.
rtForth v0.1.39, Copyright (C) 2017 Mapacode Inc.
Type 'bye' or press Ctrl-D to exit.
rf> : star 42 emit ;
 ok
rf> star
* ok
rf> star star star
*** ok
rf> bye
```

## Benchmark 2017/06/22

* ASUS X401A
* Ubuntu GNOME 14.04 LTS 32-bit
* rustc 1.19.0-nightly
* rtForth 0.1.39 subroutine-threaded
* SwiftForth 3.6.2
* gforth 0.7.2
* gforth-fast 0.7.2

SwiftForth vs gforth vs rtForth:

benchmark   | SwiftForth | gforth-fast |  gforth  | rtForth
----------- | ---------- | ----------- | -------- | -------
bubble-sort |    1       |     x       |     x    |     x        
fib         |    1       |   3.6       |   5.77   |   6.8
matrix-mult |    1       |     x       |     x    |     x
mm-rtcg     |    1       |     x       |     x    |     x
sieve       |    1       |   1.5       |   2.1    |   6.5
ssieve-a    |    1       |     x       |     x    |     x
repeat      |    1       |   7.9       |  14.5    |  26.5

rtForth subroutine-threading vs token-threading:

threading | subroutine | token
----------|------------|--------
fib       |     1      | 3.1
repeat    |     1      | 2.0
sieve     |     1      | 2.2
