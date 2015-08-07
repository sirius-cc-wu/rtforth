extern crate rtforth;
use rtforth::core::VM;
use rtforth::loader::HasLoader;
use rtforth::output::Output;
use rtforth::tools::Tools;
use std::env;

#[cfg(not(test))]
fn main() {
    let vm = &mut VM::new();
    vm.patch_output_primitives();
    vm.patch_tools();

    let args: Vec<_> = env::args().collect();
    if args.len() > 1 {
        if args[1] == "--help" {
            println!("Usage: rtforth [options] [file]");
            println!("rtForth will load lib.fs if no options and file is given.");
            println!("Options:");
            println!("  --help");
        } else {
            vm.load(&args[1]);
        }
    } else {
        hello::hello();
        vm.load("lib.fs");
    }
}

#[cfg(not(test))]
mod hello {
    pub fn hello() {
        println!("rtForth 0.0.2 by ccwu");
    }
}
