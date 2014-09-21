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
		pub s_stack: Box<Vec<int>>,
		pub r_stack: Box<Vec<int>>,
		pub word_list: Box<Vec<Word>>
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

		pub fn quit(vm: &VM) {
			println!("Quit...");
		}

		pub fn bye(vm: &VM) {
			println!("Bye...");
		}
	}

}

