#[cfg(not(test))]
fn main() {
	hello::hello();
	vm::bye();
}

mod hello {
	pub fn hello() {
	    println!("JrForth 0.0.1 by ccwu");
	}
}

mod vm;