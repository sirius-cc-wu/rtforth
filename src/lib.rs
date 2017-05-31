#![feature(libc)]
#![feature(test)]
#![feature(asm)]

extern crate byteorder;
pub mod vm;
pub mod core;
pub mod loader;
pub mod output;
pub mod exception;
pub mod tools;
pub mod env;
pub mod facility;
pub mod float;
pub mod dataspace;
pub mod codespace;

pub const TRUE: isize = -1;
pub const FALSE: isize = 0;
