echo
echo SwiftForth
echo ----------
echo
time sf ./doc/bench/forth/repeat.fs
time sf ./doc/bench/forth/repeat.fs
time sf ./doc/bench/forth/repeat.fs
echo
echo gforth-fast
echo ----------
echo
time gforth-fast ./doc/bench/forth/repeat.fs
time gforth-fast ./doc/bench/forth/repeat.fs
time gforth-fast ./doc/bench/forth/repeat.fs
echo
echo gforth
echo ----------
echo
time gforth ./doc/bench/forth/repeat.fs
time gforth ./doc/bench/forth/repeat.fs
time gforth ./doc/bench/forth/repeat.fs
echo
echo rtForth
echo ----------
echo
time rf ./doc/bench/forth/repeat.fs
time rf ./doc/bench/forth/repeat.fs
time rf ./doc/bench/forth/repeat.fs
