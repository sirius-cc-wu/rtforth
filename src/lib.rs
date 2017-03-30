#![feature(libc)]
#![feature(unique)]
#![feature(test)]

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
pub mod jitmem;

pub mod bc {
    // Byte codes
    pub const BC_NOOP: usize = 0;
    pub const BC_EXIT: usize = 1;
    pub const BC_HALT: usize = 2;
    pub const BC_LIT: usize = 3;
    pub const BC_FLIT: usize = 4;
    pub const BC_S_QUOTE: usize = 5;
    pub const BC_BRANCH: usize = 6;
    pub const BC_ZBRANCH: usize = 7;
    pub const BC_DO: usize = 8;
    pub const BC_LOOP: usize = 9;
    pub const BC_PLUS_LOOP: usize = 10;
    pub const BC_UNLOOP: usize = 11;
    pub const BC_LEAVE: usize = 12;
    pub const BC_I: usize = 13;
    pub const BC_J: usize = 14;
    pub const BC_TO_R: usize = 15;
    pub const BC_R_FROM: usize = 16;
    pub const BC_R_FETCH: usize = 17;
    pub const BC_TWO_TO_R: usize = 18;
    pub const BC_TWO_R_FROM: usize = 19;
    pub const BC_TWO_R_FETCH: usize = 20;
}
