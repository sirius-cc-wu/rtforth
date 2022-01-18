#[cfg(target_arch = "arm")]
#[macro_export]
macro_rules! primitive {
    (fn $args:tt) => { fn $args };
    (fn $f:ident $args:tt $body:tt) => { fn $f $args $body };
    (fn $f:ident $args:tt -> isize $body:tt) => { fn $f $args -> isize $body };
    (fn $f:ident $args:tt -> &mut [usize; 2] $body:tt) => { fn $f $args -> &mut [usize; 2] $body };
}

#[cfg(any(target_arch = "x86"))]
#[macro_export]
macro_rules! primitive {
    (fn $args:tt) => { extern "fastcall" fn $args };
    (fn $f:ident $args:tt $body:tt) => { extern "fastcall" fn $f $args $body };
    (fn $f:ident $args:tt -> isize $body:tt) => { extern "fastcall" fn $f $args -> isize $body };
    (fn $f:ident $args:tt -> &mut [usize; 2] $body:tt)
        => { extern "fastcall" fn $f $args -> &mut [usize; 2] $body };
}

#[cfg(any(target_arch = "x86_64"))]
#[macro_export]
macro_rules! primitive {
    (fn $args:tt) => { extern fn $args };
    (fn $f:ident $args:tt $body:tt) => { extern fn $f $args $body };
    (fn $f:ident $args:tt -> isize $body:tt) => { extern fn $f $args -> isize $body };
    (fn $f:ident $args:tt -> &mut [usize; 2] $body:tt)
        => { extern fn $f $args -> &mut [usize; 2] $body };
}

#[macro_use]
extern crate approx;
extern crate hibitset;

pub mod core;
pub mod env;
pub mod exception;
pub mod facility;
pub mod file_access;
pub mod float;
pub mod loader;
pub mod memory;
pub mod mock_vm;
pub mod output;
pub(crate) mod parser;
pub mod tools;
pub mod units;

use core::Core;
use memory::Memory;
use std::result;

pub const TRUE: isize = -1;
pub const FALSE: isize = 0;
pub const NUM_TASKS: usize = 8;

pub type Result = result::Result<(), isize>;
