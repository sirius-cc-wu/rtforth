pub mod vm {

// Word
	pub struct Word {
		is_immediate: bool,
		action: fn(&VM)
	}

impl Word {
	pub fn new(action: fn(&VM)) -> Box<Word> {
		box Word {
			is_immediate: false,
			action: action
		}
	}
	
}

// Virtual machine
	pub struct VM {
		is_paused: bool,
		s_stack: Box<Vec<int>>,
		r_stack: Box<Vec<int>>,
		word_list: Box<Vec<Box<Word>>>
	}

	impl VM {
		pub fn new() -> Box<VM> {
			let mut vm = box VM {
				is_paused: true,
				s_stack: box Vec::with_capacity(16),
				r_stack: box Vec::with_capacity(16),
				word_list: box Vec::with_capacity(16)
			};
			vm.s_stack.push(0);
			vm.r_stack.push(0);
			vm
		}

		pub fn execute_word (&self, w: &Word) {
			(w.action)(self);
		}

		pub fn quit(&self) {
			println!("Quit...");
		}

		pub fn bye(&self) {
			println!("Bye...");
		}
	}

}

