pub mod vm {

// Word
	pub struct Word<'a> {
		is_immediate: bool,
		name: &'a str,
		action: fn(&VM)
	}

impl<'a> Word<'a> {
	pub fn new(name: &'a str, action: fn(&VM)) -> Box<Word<'a>> {
		box Word {
			is_immediate: false,
			name: name,
			action: action
		}
	}
}

// Virtual machine
	pub struct VM<'a> {
		is_paused: bool,
		s_stack: Box<Vec<int>>,
		r_stack: Box<Vec<int>>,
		word_list: Box<Vec<Box<Word<'a>>>>
	}

	impl<'a> VM<'a> {
		pub fn new() -> Box<VM<'a>> {
			let mut vm = box VM {
				is_paused: true,
				s_stack: box Vec::with_capacity(16),
				r_stack: box Vec::with_capacity(16),
				word_list: box Vec::with_capacity(16)
			};
			vm.s_stack.push(0);
			vm.r_stack.push(0);
			vm.word_list.push (Word::new("noop", VM::noop));
			vm.word_list.push (Word::new("quit", VM::quit));
			vm.word_list.push (Word::new("bye", VM::bye));
			vm
		}

		pub fn execute_word (&self, w: &Word) {
			(w.action)(self);
		}

// Primitives

		pub fn noop (vm: &VM) {
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
