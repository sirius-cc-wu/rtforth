# rtForth

Forth implemented in Rust, designed for real-time applications.

Documentation at [rtForth Primer](https://mapacode.github.io/rtforth/).

## Design decisions:

* Safe first, performance later
* Token Threaded + Subroutine-threaded (only for x86)

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

## Benchmark 2018/08/24

* ACER ASPIRE
* 64-bit Ubuntu GNOME 16.04
* rustc 1.25.0-nightly (a0dcecff9 2018-01-24)
* rtforth rev ead4a0 token-threaded rev (ttc-x86)
* rtForth rev ead4a0 subroutine-threaded stc-x86)
* SwiftForth 3.7.2
* gforth 0.7.2
* gforth-fast 0.7.2

SwiftForth vs gforth vs rtForth:

benchmark   | SwiftForth | gforth-fast   |  gforth       | rtForth-stc-x86
------------|:-----------|:--------------|:--------------|:----------------
bubble-sort | 1          | x             | x             | x
fib         | 1 (0.191s) | 6.5 (1.252s)  | 10.1 (1.929s) | 13.6 (2.134s)
matrix-mult | 1          | x             | x             | x
mm-rtcg     | 1          | x             | x             | x
sieve       | 1 (0.402s) | 1.55 (0.625s) | 3.4 (1.367s)  | 8.77 (3.525s)
ssieve-a    | 1          | x             | x             | x
repeat      | 1 (0.218s) | 9.56 (2.084s) | 17.2 (3.755s) | 30.9 (6.738s)

rtForth subroutine-threading vs token-threading:

threading | stc-x86    | token-x86
----------|------------|---------------
fib       | 1 (2.596s) | 4.38 (11.372s)
sieve     | 1 (3.525s) | 3.22 (11.361s)
repeat    | 1 (6.738s) | 5.08 (34.273s)
