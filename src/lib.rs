pub mod vm {

// Word
	pub struct Word {
		vm: Box<VM>,
		pub is_immediate: bool,
		pub action: fn(&VM)
	}

impl Word {
	pub fn new(vm: Box<VM>, action: fn(&VM)) -> Word {
		Word {
			vm: vm,
			is_immediate: false,
			action: action
		}
	}
	
	pub fn execute(&self) {
		(self.action)(&*self.vm);
	}
}

// Virtual machine
	pub struct VM {
		is_paused: bool,
//		pub s_stack: Vec<int>,
//		pub r_stack: Vec<int>,
		pub word_list: Box<Vec<Word>>
	}

	impl VM {
		pub fn new() -> Box<VM> {
			let mut s_stack: Vec<int> = Vec::with_capacity(16);
			s_stack.push(0);
			let mut r_stack: Vec<int> = Vec::with_capacity(16);
			r_stack.push(0);
			let vm = box VM {
				is_paused: true,
				word_list: box Vec::with_capacity(16)
			};
			vm
		}

		pub fn quit(vm: &VM) {
			println!("Quit...");
		}

		pub fn bye(vm: &VM) {
			println!("Bye...");
		}
	}

}

