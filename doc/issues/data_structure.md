# Data Space

## Problem

Rust's vector is not a good implementation for data space:

Question:

* allot is difficult to implement.

Solution:

* Use byteorder to handle an allocated memory. Apply this strategy to f_heap at first.

Question:

* byteorder doesn't know isize/usize.

Solution:

* s_stack is kept isize wide, but s_heap's content defaults to i32. So same s_heap content can be run from both 32 bit systems and 64 bit systems.

Question:

* lit doesn't differentiate u32 and i32. It's an error a u32 with highest bit set was pushed on to stack by lit.

Solution:

* rtForth does not use u32.

# Stacks

## Problem

Rust's vector is not a good implementation for stack:

* stack operations (swap, dup, drop, rot, nip...) are heavy because of nature of vector.

## Solution

A new data structure with swap, nip, dup, drop, rot, allot.. as its primitives. These primitives is implemented with unsafe code for performance reasons.

