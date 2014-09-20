pub mod vm {

	pub struct VM {
		pub s_stack: Vec<int>,
		pub r_stack: Vec<int>,
		pub word_list: Vec<int>
	}

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
