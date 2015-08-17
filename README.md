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

2015/03/17

* ASUS X401A
* Ubuntu GNOME 14.04 LTS 64-bit
* rustc 1.4.0-nightly
* rtForth 0.1.4

```
bench_dup                                     :           6 ns/iter (+/- 0)
bench_evaluate_words_at_beginning_of_wordlist :         514 ns/iter (+/- 15)
bench_evaluate_words_at_middle_of_wordlist    :       3,794 ns/iter (+/- 61)
bench_fib                                     :       4,046 ns/iter (+/- 31)
bench_find_word_at_beginning_of_wordlist      :          19 ns/iter (+/- 1)
bench_find_word_at_end_of_wordlist            :         642 ns/iter (+/- 13)
bench_find_word_at_middle_of_wordlist         :         378 ns/iter (+/- 4)
bench_find_word_not_exist                     :         591 ns/iter (+/- 9)
bench_inner_interpreter_without_nest          :          27 ns/iter (+/- 0)
bench_noop                                    :           0 ns/iter (+/- 0)
bench_over                                    :           8 ns/iter (+/- 0)
bench_rot                                     :           7 ns/iter (+/- 0)
bench_swap                                    :           5 ns/iter (+/- 0)
```

2015/08/13

* ASUS X401A
* Ubuntu GNOME 14.04 LTS 32-bit
* rustc 1.4.0-nightly
* rtForth 0.1.2
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

```
bench_dup                                     :           8 ns/iter (+/- 0)
bench_evaluate_words_at_beginning_of_wordlist :         567 ns/iter (+/- 6)
bench_evaluate_words_at_middle_of_wordlist    :       4,121 ns/iter (+/- 64)
bench_fib                                     :       5,143 ns/iter (+/- 552)
bench_find_word_at_beginning_of_wordlist      :          25 ns/iter (+/- 0)
bench_find_word_at_end_of_wordlist            :         748 ns/iter (+/- 11)
bench_find_word_at_middle_of_wordlist         :         405 ns/iter (+/- 12)
bench_find_word_not_exist                     :         638 ns/iter (+/- 43)
bench_inner_interpreter_without_nest          :          24 ns/iter (+/- 0)
bench_noop                                    :           2 ns/iter (+/- 0)
bench_over                                    :          10 ns/iter (+/- 0)
bench_rot                                     :          11 ns/iter (+/- 0)
bench_swap                                    :           9 ns/iter (+/- 0)
```

