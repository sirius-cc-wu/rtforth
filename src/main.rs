extern crate jrforth;
use jrforth::vm::VM;

#[cfg(not(test))]
fn main() {
    hello::hello();
    let vm = &mut VM::new();
    vm.quit();
}

#[cfg(not(test))]
mod hello {
    pub fn hello() {
        println!("JrForth 0.0.1 by ccwu");
    }
}
