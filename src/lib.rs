pub mod vm {

// Word
    pub struct Word {
        is_immediate: bool,
        nfa: usize,
        name_len: usize,
        action: fn(& mut VM)
    }

impl Word {
    pub fn new(nfa: usize, name_len: usize, action: fn(& mut VM)) -> Word {
        Word {
            is_immediate: false,
            nfa: nfa,
            name_len: name_len,
            action: action
        }
    }
}

// Colon Definition
    pub struct ColonDef {
        start: isize,
        end: isize
    }

// Virtual machine
    pub struct VM {
        is_paused: bool,
        s_stack: Vec<isize>,
        r_stack: Vec<usize>,
        s_heap: Vec<isize>,
        f_heap: Vec<f64>,
        n_heap: Vec<u8>,
        word_list: Vec<Word>,
        pub found_index: usize,
        instruction_pointer: usize,
        word_pointer: usize
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
            vm.add_primitive("exit", VM::exit);
            vm.add_primitive("pause", VM::pause);
            vm.add_primitive("quit", VM::quit);
            vm.add_primitive("bye", VM::bye);
            vm
        }

        pub fn add_primitive(&mut self, name: &str, action: fn(& mut VM)) {
            self.word_list.push (Word::new(self.n_heap.len(), name.len(), action));
            for i in name.bytes() {
                self.n_heap.push(i);
            }
        }

        pub fn execute_word(& mut self, i: usize) {
            (self.word_list[i].action)(self);
        }

        pub fn find(&mut self, name: &str) {
            let mut i = 0usize;
            for w in self.word_list.iter() {
                println!("{} = {} ?", w.name_len, name.len());
                let mut j = 0usize;
                if w.name_len == name.len() {
                    for ch in name.bytes() {
                        if self.n_heap[j+w.nfa] != ch {
                            break;
                        }
                        j = j + 1;
                    }
                }
                if j == name.len() {
                    break;
                } else {
                    i += 1usize;
                }
            }
            self.found_index = i;
        }

        pub fn words(&mut self) {
            let ref word_list = self.word_list;
            for w in word_list {
                for i in (0 .. w.name_len) {
                    println!("{}", self.n_heap[w.nfa+i] as char);
                }
                println!("{}", ' ');
            }
        }

// Inner interpreter
        pub fn inner_interpret(&mut self, ip: usize) {
            self.instruction_pointer = ip;
            self.inner();
        }

        pub fn inner(&mut self) {
            while self.instruction_pointer > 0 && self.instruction_pointer < self.s_heap.len() {
                self.word_pointer = self.s_heap[self.instruction_pointer] as usize;
                self.instruction_pointer += 1;
                let w = self.word_pointer;
                self.execute_word (w);
            }
            self.instruction_pointer = 0;
        }

// Primitives

        pub fn noop(vm: & mut VM) {
            // Do nothing
        }
        
        pub fn quit(vm: & mut VM) {
            println!("Quit...");
        }

        pub fn bye(vm: & mut VM) {
            println!("Bye...");
        }

        pub fn exit(vm: & mut VM) {
            match vm.r_stack.pop() {
                None => VM::abort (vm, "Data stack underflow"),
                Some(x) => vm.instruction_pointer = x,
            }
        }

        pub fn pause(vm: & mut VM) {
            vm.r_stack.push(vm.instruction_pointer);
            vm.instruction_pointer = 0;
            vm.is_paused = true;
        }

        pub fn abort(vm: &VM, msg: &str) {
            // TODO
        }
    }

}
