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
    VM::words(vm);
    VM::quit(vm);
    VM::bye(vm);
}

#[cfg(not(test))]
mod hello {
    pub fn hello() {
        println!("JrForth 0.0.1 by ccwu");
    }
}
