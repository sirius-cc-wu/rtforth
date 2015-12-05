#![feature(libc)]
#![feature(unique)]
#![feature(test)]
#![feature(plugin)]
#![plugin(clippy)]

extern crate byteorder;
pub mod core;
pub mod loader;
pub mod output;
pub mod exception;
pub mod tools;
pub mod env;
pub mod facility;
pub mod float;

mod word;
mod jitmem;
