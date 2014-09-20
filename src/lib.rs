pub mod vm {

	pub fn quit() {
		let mut s_stack: Vec<int> = Vec::with_capacity(16);
		s_stack.push(0);
		let mut r_stack: Vec<int> = Vec::with_capacity(16);
		r_stack.push(0);
		let mut word_list = Vec::with_capacity(16);
		word_list.push(bye);
	}

	pub fn bye() {
		println!("Bye...");
	}
}
