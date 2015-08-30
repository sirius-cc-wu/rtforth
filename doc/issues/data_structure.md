# Data Space

## Problem

Rust's vector is not a good implementation for data space:

* allot is difficult to implement.

## Solution:

* Use rustc-serialize to handle an allocated memory. Apply this strategy to f_heap at first.

# Stacks

## Problem

Rust's vector is not a good implementation for stack:

* stack operations (swap, dup, drop, rot, nip...) are heavy because of nature of vector.

## Solution

A new data structure with swap, nip, dup, drop, rot, allot.. as its primitives. These primitives is implemented with unsafe code for performance reasons.

