extern crate jrforth;
use jrforth::vm::VM;

#[cfg(not(test))]
fn main() {
    hello::hello();
    let vm = &mut VM::new();
    vm.find("");
    println!("vm.found_index of empty string: {}", vm.found_index);
    vm.find("noop");
    println!("vm.found_index of noop: {}", vm.found_index);
    vm.find("bye");
    println!("vm.found_index of bye: {}", vm.found_index);
    vm.find("quit");
    println!("vm.found_index of quit: {}", vm.found_index);
    vm.find("not-exist");
    println!("vm.found_index of not-exist: {}", vm.found_index);
    vm.words();
// Test Compiler
    vm.find("quit");
    let idx = vm.found_index;
    vm.compile_word(idx);
    vm.find("bye");
    let idx = vm.found_index;
    vm.compile_word(idx);
    vm.compile_integer(3);
    vm.compile_integer(2);
    vm.compile_integer(1);
    vm.find(".s");
    let idx = vm.found_index;
    vm.compile_word(idx);
    vm.inner_interpret(1);
}

#[cfg(not(test))]
mod hello {
    pub fn hello() {
        println!("JrForth 0.0.1 by ccwu");
    }
}
