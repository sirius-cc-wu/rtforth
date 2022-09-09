# rtForth

Forth implemented in Rust, designed to be embeddable in real-time applications.

Documentation in traditional Chinese at
[rtForth 入門](https://chengchangwu.github.io/rtforth/).

## Design decisions:

* Safe first, performance later
* Call threading

## Usage of the example program rtf

```
cargo install --path ./rtf
rtf --help         # Display help information.
rtf <file>         # Load forth commands in <file>.
rtf lib.fth <file>  # Load lib.fth before <file>.
```

```
$ rtf              # Execute debug version of rtForth.
rtForth v0.6.6, Copyright (C) 2022 Mapacode Inc.
Type 'bye' or press Ctrl-D to exit.
: star 
   42 emit ;  ok
: stars 
   0 ?do star loop ;  ok
5 stars ***** ok
bye
```

## Use as a Rust crate

rtForth is designed to be used in a real-time system. After startup most of
the words other than input and output words do not use system calls.

Input and output words can be implemented by applications according to the
operating system used.

See examples/simple.rs and examples/multitask.rs to get know how to embedded
rtforth in a rust application.