rtForth is not compatible to ANS Forth:

* max_n 1+ causes arithmetic operation overflowed, because rust's strong type. This may be solved with unsafe code.
