extern crate jrforth;
use jrforth::vm::VM;

#[cfg(not(test))]
fn main() {
	hello::hello();
	let vm = VM::new();
	VM::quit(&*vm);
	VM::bye(&*vm);
}

#[cfg(not(test))]
mod hello {
	pub fn hello() {
	    println!("JrForth 0.0.1 by ccwu");
	}
}
