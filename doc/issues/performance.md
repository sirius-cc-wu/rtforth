Performance
===========

Benchmark against SwiftForth and GForth
=======================================

See benchmarks in doc/bench/forth/.

2015/08/13

* ASUS X401A
* Ubuntu GNOME 14.04 LTS 32-bit
* rustc 1.4.0-nightly
* rtForth 0.1.6
* SwiftForth 3.5.7
* gforth 0.7.0

benchmark   | SwiftForth | gforth  | rtForth
----------- | ---------- | ------- | -------
bubble-sort |    1       |     x   |     x
fib         |    1       |  5.23   | 18.72
matrix-mult |    1       |     x   |     x
mm-rtcg     |    1       |     x   |     x
sieve       |    1       |     x   |     x
ssieve-a    |    1       |     x   |     x


Benchmark from cargo bench
===========================

2015/08/22

```
bench_2drop                                  :           5 ns/iter (+/- 0)
bench_2dup                                   :           9 ns/iter (+/- 0)
bench_2over                                  :          13 ns/iter (+/- 2)
bench_2swap                                  :           7 ns/iter (+/- 0)
bench_compile_words_at_beginning_of_wordlist :       1,372 ns/iter (+/- 18)
bench_compile_words_at_middle_of_wordlist    :       4,342 ns/iter (+/- 141)
bench_drop                                   :           6 ns/iter (+/- 0)
bench_dup                                    :           6 ns/iter (+/- 0)
bench_fib                                    :       4,698 ns/iter (+/- 275)
bench_find_word_at_beginning_of_wordlist     :          25 ns/iter (+/- 2)
bench_find_word_at_end_of_wordlist           :         717 ns/iter (+/- 33)
bench_find_word_at_middle_of_wordlist        :         415 ns/iter (+/- 7)
bench_find_word_not_exist                    :         638 ns/iter (+/- 32)
bench_inner_interpreter_without_nest         :          24 ns/iter (+/- 0)
bench_nip                                    :           5 ns/iter (+/- 0)
bench_noop                                   :           2 ns/iter (+/- 0)
bench_over                                   :           6 ns/iter (+/- 0)
bench_rot                                    :           3 ns/iter (+/- 0)
bench_swap                                   :           2 ns/iter (+/- 0)
bench_to_r_r_fetch_r_from                    :          71 ns/iter (+/- 1)
bench_two_to_r_two_r_fetch_two_r_from        :          87 ns/iter (+/- 2)
```

Next Steps to Improve Performance
=================================

* Performance of return stack was improved with unsafe code, those code
  (push, fetch and pop) cannot be wrapped in an implementation of struct Stack,
  because aborting push() and pop() needs help of VM. Which suggests an error
  handler shared between VM and Stack. Still learning Rc and RefCell to
  understand the possibilities.

* After the above problem is solved, the struct Stack could be used for data
  stack to improved data stack's performance. It is believed that unsafe code
  could bring more than 10% improvement for those forth instructions which
  operates on more than one item on data stack, like + - rot over nip 2swap
  etc.

* Already improved 2drop 2dup 2over 2swap drop dup nip over rot swap. But the
  bench code use s_stack.push() or s_stack.pop() which is slow. That's why some
  benchmarks degrade.

