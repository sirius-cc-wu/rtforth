extern crate jrforth;
use jrforth::vm::VM;

#[cfg(not(test))]
fn main() {
	hello::hello();
	let vm = &mut VM::new();
	vm.find("noop");
	println!("vm.found_index of noop: {}", vm.found_index);
	vm.find("bye");
	println!("vm.found_index of bye: {}", vm.found_index);
	vm.find("quit");
	VM::quit(vm);
	VM::bye(vm);
}

#[cfg(not(test))]
mod hello {
	pub fn hello() {
	    println!("JrForth 0.0.1 by ccwu");
	}
}
