# rtForth

Forth implemented in Rust, designed for real-time applications.

Documentation at [rtForth Primer](https://chengchangwu.github.io/rtforth/).

## Design decisions:

* Safe first, performance later
* Token Threaded + Subroutine-threaded (only for x86)

## Usage

```
cargo install --path ./rtf
rtf --help         # Display help information.
rtf <file>         # Load forth commands in <file>.
rtf lib.fs <file>  # Load lib.fs before <file>.
```

```
$ rtf              # Execute debug version of rtForth.
rtForth v0.6.0, Copyright (C) 2018 Mapacode Inc.
Type 'bye' or press Ctrl-D to exit.
rf> : star 42 emit ;
 ok
rf> star
* ok
rf> star star star
*** ok
rf> bye
```

## Benchmark 2018/08/24

* ACER ASPIRE
* 64-bit Ubuntu GNOME 16.04
* rustc 1.25.0-nightly (a0dcecff9 2018-01-24)
* old: rev 413b0c4 (2017/06/22)
* rtforth rev 413b0c4 (2017/06/22) token-threaded (token-x86-old)
* rtForth rev 413b0c4 (2017/06/22) subroutine-threaded (stc-x86-old)
* rtforth rev ead4a0 token-threaded rev (ttc-x86)
* rtForth rev ead4a0 subroutine-threaded stc-x86)
* SwiftForth 3.7.2
* gforth 0.7.2
* gforth-fast 0.7.2

SwiftForth vs gforth vs rtForth:

benchmark   | SwiftForth | gforth-fast   |  gforth       | stc-x86
------------|:-----------|:--------------|:--------------|:----------------
bubble-sort | 1          | x             | x             | x
fib         | 1 (0.191s) | 6.5 (1.252s)  | 10.1 (1.929s) | 13.6 (2.134s)
matrix-mult | 1          | x             | x             | x
mm-rtcg     | 1          | x             | x             | x
sieve       | 1 (0.402s) | 1.55 (0.625s) | 3.4 (1.367s)  | 8.77 (3.525s)
ssieve-a    | 1          | x             | x             | x
repeat      | 1 (0.218s) | 9.56 (2.084s) | 17.2 (3.755s) | 30.9 (6.738s)

rtForth subroutine-threading vs token-threading:

threading | stc-x86    | token-x86      | stc-x86-old   | token-x86-old
----------|:-----------|:---------------|:--------------|:--------------
fib       | 1 (2.596s) | 4.38 (11.372s) | 0.86 (2.242s) | 6.30 (16.357s)
sieve     | 1 (3.525s) | 3.22 (11.361s) | x             | x
repeat    | 1 (6.738s) | 5.08 (34.273s) | 0.96 (6.489s) | 6.28 (42.359s)
