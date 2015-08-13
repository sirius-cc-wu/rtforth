rtForth
=======

Simple Forth implemented in Rust

Design decisions:

* Token Threaded (Call threading), easy to implement in Rust
* Jit planned in the future to improve the performance

The performance of current implementation is not well because of token threading.
But slow colon definitions can be improved with a Just-In-Time compiler.
After optimization, corresponding slots in word list points to the jitted definitions.

Benchmark
=========

See benchmarks in doc/bench/forth/.

2015/08/13

            | gforth | rtForth
------------|--------|---------
bubble-sort |        |        
fib         |    1   |   3.8   
matrix-mult |        |        
mm-rtcg     |        |
sieve       |        |        
ssieve-a    |        |         

