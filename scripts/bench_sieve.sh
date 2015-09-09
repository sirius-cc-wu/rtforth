echo
echo SwiftForth
echo ----------
echo
time sf ./doc/bench/forth/sieve.fs
time sf ./doc/bench/forth/sieve.fs
time sf ./doc/bench/forth/sieve.fs
echo
echo gforth-fast
echo ----------
echo
time gforth-fast ./doc/bench/forth/sieve.fs
time gforth-fast ./doc/bench/forth/sieve.fs
time gforth-fast ./doc/bench/forth/sieve.fs
echo
echo gforth
echo ----------
echo
time gforth ./doc/bench/forth/sieve.fs
time gforth ./doc/bench/forth/sieve.fs
time gforth ./doc/bench/forth/sieve.fs
echo
echo rtForth
echo ----------
echo
time rf lib.fs ./doc/bench/forth/sieve.fs
time rf lib.fs ./doc/bench/forth/sieve.fs
time rf lib.fs ./doc/bench/forth/sieve.fs
