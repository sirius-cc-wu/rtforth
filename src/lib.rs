pub mod vm {

// Word
	pub struct Word<'a> {
		is_immediate: bool,
		name: &'a str,
		action: fn(&VM)
	}

impl<'a> Word<'a> {
	pub fn new(name: &'a str, action: fn(&VM)) -> Word<'a> {
		Word {
			is_immediate: false,
			name: name,
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
	pub struct VM<'a> {
		is_paused: bool,
		s_stack: Vec<int>,
		r_stack: Vec<int>,
		s_heap: Vec<int>,
		f_heap: Vec<f64>,
		word_list: Vec<Word<'a>>,
		pub found_index: uint,
		instruction_pointer: uint,
		word_pointer: uint
	}

	impl<'a> VM<'a> {
		pub fn new() -> VM<'a> {
			let mut vm = VM {
				is_paused: true,
				s_stack: Vec::with_capacity(16),
				r_stack: Vec::with_capacity(16),
				s_heap: Vec::with_capacity(64),
				f_heap: Vec::with_capacity(64),
				word_list: Vec::with_capacity(16),
				found_index: 0,
				instruction_pointer: 0,
				word_pointer: 0
			};
			vm.s_stack.push(0);
			vm.r_stack.push(0);
			vm.word_list.push (Word::new("", VM::noop));
			vm.word_list.push (Word::new("noop", VM::noop));
			vm.word_list.push (Word::new("quit", VM::quit));
			vm.word_list.push (Word::new("bye", VM::bye));
			vm
		}

		pub fn execute_word(&self, i: uint) {
			(self.word_list[i].action)(self);
		}

		pub fn find(&mut self, name: &str) {
			let mut i = 0u;
			for x in self.word_list.iter() {
				if x.name == name {
					break;
				}
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
