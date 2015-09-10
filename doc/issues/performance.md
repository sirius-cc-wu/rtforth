# Performance

## Profiling

$valgrind --dsymutil=yes --tool=callgrind ./target/release/examples/rf ./doc/bench/forth/repeat.fs
$callgrind_annotate callgrind.out.10966 | less

* Without stack checking, over saves 1/3 ir.
* The most importance fucntion is inner, which takes about 1/2 ir of total run.
* 以是指執行時，但對 CNC 而言，最重要的可能是每 ms 處理接受到的指令的速度。

## Benchmark from cargo bench

2015/09/09

```
bench_2drop                                  :           3 ns/iter (+/- 0)
bench_2dup                                   :           6 ns/iter (+/- 0)
bench_2over                                  :           5 ns/iter (+/- 0)
bench_2swap                                  :           4 ns/iter (+/- 0)
bench_compile_words_at_beginning_of_wordlist :       2,983 ns/iter (+/- 100)
bench_compile_words_at_end_of_wordlist       :       7,886 ns/iter (+/- 121)
bench_drop                                   :           3 ns/iter (+/- 0)
bench_dup                                    :           3 ns/iter (+/- 1)
bench_fib                                    :       5,228 ns/iter (+/- 397)
bench_find_word_at_beginning_of_wordlist     :          19 ns/iter (+/- 0)
bench_find_word_at_end_of_wordlist           :         635 ns/iter (+/- 3)
bench_find_word_not_exist                    :         610 ns/iter (+/- 8)
bench_inner_interpreter_without_nest         :          30 ns/iter (+/- 0)
bench_minus                                  :           5 ns/iter (+/- 0)
bench_mod                                    :          10 ns/iter (+/- 0)
bench_nip                                    :           3 ns/iter (+/- 0)
bench_noop                                   :           2 ns/iter (+/- 0)
bench_one_minus                              :           2 ns/iter (+/- 0)
bench_one_plus                               :           2 ns/iter (+/- 0)
bench_over                                   :           3 ns/iter (+/- 0)
bench_plus                                   :           5 ns/iter (+/- 0)
bench_rot                                    :           4 ns/iter (+/- 0)
bench_slash                                  :          13 ns/iter (+/- 0)
bench_slash_mod                              :          12 ns/iter (+/- 0)
bench_star                                   :           6 ns/iter (+/- 0)
bench_swap                                   :           3 ns/iter (+/- 0)
bench_to_r_r_fetch_r_from                    :          66 ns/iter (+/- 6)
bench_two_to_r_two_r_fetch_two_r_from        :          77 ns/iter (+/- 1)
```

## Possilble solutions to improve the performance

* Do not return Option<Exception> all every instructions, set error_code and check it when necessary.

