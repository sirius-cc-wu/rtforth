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
$ ./target/release/examples/rf              # Execute optimized rtForth.
```

