extern crate jrforth;
use jrforth::vm;

#[cfg(not(test))]
fn main() {
	hello::hello();
	vm::new();
	vm::quit();
	vm::bye();
}

#[cfg(not(test))]
mod hello {
	pub fn hello() {
	    println!("JrForth 0.0.1 by ccwu");
	}
}
