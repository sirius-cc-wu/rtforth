#![feature(libc)]
#![feature(test)]

#[cfg(target_arch = "arm")]
#[macro_export]
macro_rules! primitive {
    (fn $args:tt) => { fn $args };
    (fn $f:ident $args:tt $body:tt) => { fn $f $args $body };
    (fn $f:ident $args:tt -> isize $body:tt) => { fn $f $args -> isize $body };
    (fn $f:ident $args:tt -> &mut [usize; 2] $body:tt) => { fn $f $args -> &mut [usize; 2] $body };
}

#[cfg(target_arch = "x86")]
#[macro_export]
macro_rules! primitive {
    (fn $args:tt) => { extern "fastcall" fn $args };
    (fn $f:ident $args:tt $body:tt) => { extern "fastcall" fn $f $args $body };
    (fn $f:ident $args:tt -> isize $body:tt) => { extern "fastcall" fn $f $args -> isize $body };
    (fn $f:ident $args:tt -> &mut [usize; 2] $body:tt) => { extern "fastcall" fn $f $args -> &mut [usize; 2] $body };
}

extern crate uom;

pub mod vm;
pub mod core;
pub mod loader;
pub mod output;
pub mod exception;
pub mod tools;
pub mod env;
pub mod facility;
pub mod float;
pub mod units;
pub mod dataspace;
pub mod codespace;

pub const TRUE: isize = -1;
pub const FALSE: isize = 0;
