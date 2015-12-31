# Performance

## Profiling

```
$valgrind --tool=callgrind ./target/release/examples/rf ./doc/bench/forth/repeat.fs
$callgrind_annotate callgrind.out.<pid> | less
$kcachegrind callgrind.out.<pid>&
```

### repeat.fs

* Without stack checking, over saves 1/3 ir.
* The most importance fucntion is inner, which takes about 1/2 ir of total run.
* for 80000 bench,  inner executes 9,520,189 ir, over 3,040,038 ir, lit, 2,240,056 
* word_pointer += 1 takes 2.5 ir. But only nest, p_var, p_const, ... use word_pointer. word_pointer can be removed.

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

* JIT: https://github.com/jonathandturner/rustyjit

* Stack checking: Elizabeth said:

Testing is supposed to be an interactive process. You do not
write a bunch of definitions and then try executing the highest-level
one. You try each, in turn, typing appropriate arguments as needed and
then checking the stack afterwords, testing bottom-up. If you do this
you will quickly find stack as well as logical problems. I would never
write a system that checked the stack at every word! Stack checking
should be performed whenever you're at the command-line level.
type:

But others support checking before access:

Gforth effectively does such a check (it uses a memory protection trap
instead of a runtime comparison, but it still signals the error where it
happens).  I've found it very helpful. 

How is that implemented?  There is an unmapped page where the memory
location for the stack item below the bottom of the stack would lie,
so accessing that memory location causes a segmentation violation in
the OS, and when the access is to a place close to the bottom of the
data stack, Gforth translates this into a "-4 throw" (stack
underflow).  So this does not cost any checks at run-time.

There is a cost, though: For accurate error reporting the debugging
engine always stores the IP and the RP in memory.  Also, the debugging
engine keeps all stack items in memory (no stack caching in
registers), and does not use static superinstructions: Stack caching
occasionally loads memory cells below the stack bottom into a register
(when the stack is empty), and for static superinstructions there is
only one IP to report for a sequence of primitives. 


On the other hand, the unmapped page is really "cost-free"; we do it only on
CPUs with MMU, though.  All OSes on these CPUs are now friendly enough.  We
do it on the floating point stack, too, but not on return stack and locals
stack, to avoid aliasing problems.  Early versions of Gforth, when CPUs with
direct mapped non-associative cache were still popular even had a
displacement for the floating point stack, so that all four stacks didn't
alias in normal operating conditions. 

It also might work on some of the current Cortex M's--they don't have
MMU's per se, but they have some memory segmention to protect tasks from
each other.  I don't know the details of this so I'm not sure if it's
really doable. 
