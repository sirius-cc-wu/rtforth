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
$ ./target/debug/rf lib.fs <file>   # Load lib.fs before <file>.
$ cargo build --release      # Compile optimized rtForth.
$ ./target/release/rf        # Execute optimized rtForth.
```

Benchmark against GForth
=====================

See benchmarks in doc/bench/forth/.

2015/08/13

* ASUS X401A
* Ubuntu GNOME 14.04 LTS 32-bit
* rustc 1.4.0-nightly
* rtForth 0.1.5
* SwiftForth 3.5.7
* gforth 0.7.0
* ficl 4.1.0

benchmark   | SwiftForth | gforth  | rtForth |  ficl
----------- | ---------- | ------- | ------- | -----------
bubble-sort |    1       |     x   |     x   |     x
fib         |    1       |  7.32   | 28.82   | 37.36
matrix-mult |    1       |     x   |     x   |     x
mm-rtcg     |    1       |     x   |     x   |     x
sieve       |    1       |     x   |     x   |     x
ssieve-a    |    1       |     x   |     x   |     x


Benchmark from cargo bench
===========================

2015/08/22

```
bench_dup                                     :           8 ns/iter (+/- 0)
bench_evaluate_words_at_beginning_of_wordlist :         562 ns/iter (+/- 4)
bench_evaluate_words_at_middle_of_wordlist    :       4,471 ns/iter (+/- 40)
bench_fib                                     :       5,285 ns/iter (+/- 141)
bench_find_word_at_beginning_of_wordlist      :          25 ns/iter (+/- 0)
bench_find_word_at_end_of_wordlist            :         771 ns/iter (+/- 33)
bench_find_word_at_middle_of_wordlist         :         456 ns/iter (+/- 14)
bench_find_word_not_exist                     :         721 ns/iter (+/- 35)
bench_inner_interpreter_without_nest          :          27 ns/iter (+/- 0)
bench_noop                                    :           1 ns/iter (+/- 0)
bench_over                                    :          10 ns/iter (+/- 0)
bench_rot                                     :          11 ns/iter (+/- 0)
bench_swap                                    :           9 ns/iter (+/- 0)
bench_to_r_r_fetch_r_from                     :          76 ns/iter (+/- 1)
bench_two_to_r_two_r_fetch_two_r_from         :          89 ns/iter (+/- 3)
```
