extern crate jrforth;

#[cfg(not(test))]
fn main() {
	hello::hello();
	jrforth::bye();
}

#[cfg(not(test))]
mod hello {
	pub fn hello() {
	    println!("JrForth 0.0.1 by ccwu");
	}
}
