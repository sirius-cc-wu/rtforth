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

Benchmark
=========

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

bench_dup                                     :           8 ns/iter (+/- 0)
bench_evaluate_words_at_beginning_of_wordlist :         541 ns/iter (+/- 23)
bench_evaluate_words_at_middle_of_wordlist    :       4,377 ns/iter (+/- 70)
bench_fib                                     :       5,680 ns/iter (+/- 71)
bench_find_word_at_beginning_of_wordlist      :          25 ns/iter (+/- 0)
bench_find_word_at_end_of_wordlist            :         786 ns/iter (+/- 38)
bench_find_word_at_middle_of_wordlist         :         449 ns/iter (+/- 11)
bench_find_word_not_exist                     :         729 ns/iter (+/- 19)
bench_inner_interpreter_without_nest          :          24 ns/iter (+/- 0)
bench_noop                                    :           2 ns/iter (+/- 0)
bench_over                                    :          10 ns/iter (+/- 1)
bench_rot                                     :          11 ns/iter (+/- 2)
bench_swap                                    :           9 ns/iter (+/- 0)
bench_to_r_r_fetch_r_from                     :          78 ns/iter (+/- 3)
bench_two_to_r_two_r_fetch_two_r_from         :         119 ns/iter (+/- 16)

