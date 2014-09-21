pub mod vm {

// Word
	pub struct Word {
		pub is_immediate: bool,
		pub action: fn()
	}

impl Word {
	pub fn new(action: fn()) -> Word {
		Word {
			is_immediate: false,
			action: action
		}
	}
}

// Virtual machine
	pub struct VM {
		is_paused: bool,
//		pub s_stack: Vec<int>,
//		pub r_stack: Vec<int>,
		pub word_list: Vec<fn()>
	}

	impl VM {
		pub fn new() -> VM {
			let mut s_stack: Vec<int> = Vec::with_capacity(16);
			s_stack.push(0);
			let mut r_stack: Vec<int> = Vec::with_capacity(16);
			r_stack.push(0);
			let vm = VM {
				is_paused: true,
				word_list: Vec::with_capacity(16)
			};
			vm
		}

		pub fn quit(&self) {
		}
	
		pub fn bye(&self) {
			println!("Bye...");
		}
	}

}

