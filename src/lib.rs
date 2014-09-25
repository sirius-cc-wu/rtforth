pub mod vm {

// Word
	pub struct Word {
		is_immediate: bool,
		nfa: uint,
		action: fn(&VM)
	}

impl Word {
	pub fn new(nfa: uint, action: fn(&VM)) -> Word {
		Word {
			is_immediate: false,
			nfa: nfa,
			action: action
		}
	}
}

// Colon Definition
	pub struct ColonDef {
		start: int,
		end: int
	}

// Virtual machine
	pub struct VM {
		is_paused: bool,
		s_stack: Vec<int>,
		r_stack: Vec<int>,
		s_heap: Vec<int>,
		f_heap: Vec<f64>,
		n_heap: Vec<char>,
		word_list: Vec<Word>,
		pub found_index: uint,
		instruction_pointer: uint,
		word_pointer: uint
	}

	impl VM {
		pub fn new() -> VM {
			let mut vm = VM {
				is_paused: true,
				s_stack: Vec::with_capacity(16),
				r_stack: Vec::with_capacity(16),
				s_heap: Vec::with_capacity(64),
				f_heap: Vec::with_capacity(64),
				n_heap: Vec::with_capacity(64),
				word_list: Vec::with_capacity(16),
				found_index: 0,
				instruction_pointer: 0,
				word_pointer: 0
			};
			vm.s_stack.push(0);
			vm.r_stack.push(0);
			vm.add_primitive("", VM::noop);
			vm.add_primitive("noop", VM::noop);
			vm.add_primitive("quit", VM::quit);
			vm.add_primitive("bye", VM::bye);
			vm.word_list.push (Word::new(vm.n_heap.len(), VM::noop));
			vm.word_list.push (Word::new(vm.n_heap.len(), VM::noop));
			vm.word_list.push (Word::new( vm.n_heap.len(), VM::quit));
			vm.word_list.push (Word::new(vm.n_heap.len(), VM::bye));
			vm
		}

		pub fn add_primitive(&mut self, name: &str, action: fn(&VM)) {
			for i in name.chars() {
				self.n_heap.push(i);
			}
		}

		pub fn execute_word(&self, i: uint) {
			(self.word_list[i].action)(self);
		}

		pub fn find(&mut self, name: &str) {
			let mut i = 0u;
			for x in self.word_list.iter() {
//				if x.name == name {
//					break;
//				}
				i += 1u;
			}
			self.found_index = i;
			
		}

// Inner interpreter
		pub fn inner_interpret(&mut self, ip: uint) {
			self.instruction_pointer = ip;
			self.inner();
		}

		pub fn inner(&mut self) {
			while self.instruction_pointer > 0 && self.instruction_pointer < self.s_heap.len() {
				self.word_pointer = self.s_heap[self.instruction_pointer] as uint;
				self.instruction_pointer += 1;
				self.execute_word (self.word_pointer);
			}
			self.instruction_pointer = 0;
		}

// Primitives

		pub fn noop(vm: &VM) {
			// Do nothing
		}
		
		pub fn quit(vm: &VM) {
			println!("Quit...");
		}

		pub fn bye(vm: &VM) {
			println!("Bye...");
		}
	}

}
