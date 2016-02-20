#![feature(libc)]
#![feature(unique)]
#![feature(test)]
#![feature(plugin)]
#![feature(raw)]
#![feature(deprecated)]

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

mod jitmem;
