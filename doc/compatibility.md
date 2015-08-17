rtForth is not yet compatible to ANS Forth in the following way:

* rtForth is case-sensitivity for performance reason.
* Data-space is an array of isize (32/64-bit cells), not byte-addressable.
* Some words are not compatible to ANS FORTH.

The following words are not compatible to ANS Forth:

* parse 
* flush


