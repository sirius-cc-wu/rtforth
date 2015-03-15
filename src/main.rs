extern crate jrforth;
use jrforth::VM;

#[cfg(not(test))]
fn main() {
    hello::hello();
    let vm = &mut VM::new();
    vm.p_false();
    vm.p_true();
    vm.s_stack.push(2);
    vm.dot_s();
    vm.words();
}

#[cfg(not(test))]
mod hello {
    pub fn hello() {
        println!("JrForth 0.0.1 by ccwu");
    }
}
