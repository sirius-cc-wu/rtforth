echo
echo SwiftForth
echo ----------
echo
time sf ./doc/bench/forth/fib.fs
time sf ./doc/bench/forth/fib.fs
time sf ./doc/bench/forth/fib.fs
echo
echo gforth-fast
echo ----------
echo
time gforth-fast ./doc/bench/forth/fib.fs
time gforth-fast ./doc/bench/forth/fib.fs
time gforth-fast ./doc/bench/forth/fib.fs
echo
echo gforth
echo ----------
echo
time gforth ./doc/bench/forth/fib.fs
time gforth ./doc/bench/forth/fib.fs
time gforth ./doc/bench/forth/fib.fs
echo
echo rtForth
echo ----------
echo
time rf ./doc/bench/forth/fib.fs
time rf ./doc/bench/forth/fib.fs
time rf ./doc/bench/forth/fib.fs
