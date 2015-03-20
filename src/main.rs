extern crate jrforth;
use jrforth::VM;

#[cfg(not(test))]
fn main() {
    hello::hello();
    let vm = &mut VM::new();
    vm.load("lib.fs");
}

#[cfg(not(test))]
mod hello {
    pub fn hello() {
        println!("JrForth 0.0.1 by ccwu");
    }
}
