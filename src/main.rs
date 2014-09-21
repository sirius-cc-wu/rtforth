extern crate jrforth;
use jrforth::vm;

#[cfg(not(test))]
fn main() {
	hello::hello();
	let vm = vm::VM::new();
	vm::VM::quit(&*vm);
	let word = vm::Word::new(vm, vm::VM::bye);
	word.execute();
}

#[cfg(not(test))]
mod hello {
	pub fn hello() {
	    println!("JrForth 0.0.1 by ccwu");
	}
}
