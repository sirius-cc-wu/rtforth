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

* SwiftForth 3.5.7
* gforth 0.7.0
* ficl 4.1.0
* rtForth 0.1.2

benchmark   | SwiftForth | gforth  | rtForth |  ficl
----------- | ---------- | ------- | ------- | -----------
bubble-sort |    1       |     x   |     x   |     x
fib         |    1       |  7.32   | 28.82   | 37.36
matrix-mult |    1       |     x   |     x   |     x
mm-rtcg     |    1       |     x   |     x   |     x
sieve       |    1       |     x   |     x   |     x
ssieve-a    |    1       |     x   |     x   |     x

```
bench_dup                                     :           8 ns/iter (+/- 0)
bench_evaluate_words_at_beginning_of_wordlist :         590 ns/iter (+/- 10)
bench_evaluate_words_at_middle_of_wordlist    :       3,793 ns/iter (+/- 45)
bench_find_word_at_beginning_of_wordlist      :          25 ns/iter (+/- 2)
bench_find_word_at_end_of_wordlist            :         662 ns/iter (+/- 2)
bench_find_word_at_middle_of_wordlist         :         381 ns/iter (+/- 11)
bench_find_word_not_exist                     :         584 ns/iter (+/- 1)
bench_inner_interpreter_without_nest          :          27 ns/iter (+/- 0)
bench_noop                                    :           1 ns/iter (+/- 0)
bench_over                                    :          10 ns/iter (+/- 0)
bench_rot                                     :          11 ns/iter (+/- 0)
bench_swap                                    :           9 ns/iter (+/- 0)
```

