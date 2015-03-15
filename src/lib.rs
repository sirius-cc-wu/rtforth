pub mod vm {

use std::str;

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
        is_compiling: bool,
        is_paused: bool,
        pub s_stack: Vec<isize>,
        r_stack: Vec<usize>,
        s_heap: Vec<isize>,
        f_heap: Vec<f64>,
        n_heap: Vec<u8>,
        word_list: Vec<Word>,
        found_index: isize,
        instruction_pointer: usize,
        word_pointer: usize,
        idx_lit: isize
    }

    impl VM {
        pub fn new() -> VM {
            let mut vm = VM {
                is_compiling: false,
                is_paused: true,
                s_stack: Vec::with_capacity(16),
                r_stack: Vec::with_capacity(16),
                s_heap: Vec::with_capacity(64),
                f_heap: Vec::with_capacity(64),
                n_heap: Vec::with_capacity(64),
                word_list: Vec::with_capacity(16),
                found_index: 0,
                instruction_pointer: 0,
                word_pointer: 0,
                idx_lit: 0
            };
            // index of 0 means not found.
            vm.add_primitive("", VM::noop);
            vm.add_primitive("noop", VM::noop);
            vm.add_primitive("true", VM::p_true);
            vm.add_primitive("false", VM::p_false);
            vm.add_primitive(".s", VM::dot_s);
            vm.add_primitive("words", VM::words);
            vm.add_primitive("lit", VM::lit);;
            vm.add_primitive("exit", VM::exit);
            vm.add_primitive("pause", VM::pause);
            vm.find("lit");
            vm.idx_lit = vm.found_index;
            // S_heap is beginning with noop, because s_heap[0] should not be used.
            vm.find("noop");
            let idx = vm.found_index;
            vm.compile_word(idx);
            vm
        }

        pub fn add_primitive(&mut self, name: &str, action: fn(& mut VM)) {
            self.word_list.push (Word::new(self.n_heap.len(), name.len(), action));
            for i in name.bytes() {
                self.n_heap.push(i);
            }
        }

        pub fn execute_word(&mut self, i: usize) {
            (self.word_list[i].action)(self);
        }

        /// Find the word with name 'name'.
        /// The found index is stored in 'found_index'.
        /// If not found the value of 'found_index' is zero.
        pub fn find(&mut self, name: &str) {
            let mut i = 0isize;
            self.found_index = 0;
            for w in self.word_list.iter() {
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
                    self.found_index = i;
                    break;
                } else {
                    i += 1isize;
                }
            }
        }

        pub fn words(&mut self) {
            for w in self.word_list.iter() {
                let s = match str::from_utf8(&self.n_heap[w.nfa..w.nfa+w.name_len]) {
                    Ok(v) => v,
                    Err(e) => panic!("Invalid word name.")
                };
                println!("{}", s );
            }
        }

        pub fn dot_s(&mut self) {
            println!("<{}>", self.s_stack.len());
            for s in self.s_stack.iter() {
                println!("{}", s);
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

// Compiler

        pub fn compile(&mut self) {
            self.is_compiling = true;
        }

        pub fn compile_word(&mut self, word_index: isize) {
            self.s_heap.push(word_index);
        }

        /// Compile integer 'i'.
        pub fn compile_integer (&mut self, i: isize) {
            self.s_heap.push(self.idx_lit);
            self.s_heap.push(i);
        }

// Primitives

        pub fn noop(&mut self) {
            // Do nothing
        }

        pub fn p_true(&mut self) {
            self.s_stack.push (-1);
        }

        pub fn p_false(&mut self) {
            self.s_stack.push (0);
        }

        pub fn lit(&mut self) {
            self.s_stack.push (self.s_heap[self.instruction_pointer]);
            self.instruction_pointer = self.instruction_pointer + 1;
        }

        pub fn exit(&mut self) {
            match self.r_stack.pop() {
                None => VM::abort (self, "Return stack underflow"),
                Some(x) => self.instruction_pointer = x,
            }
        }

        pub fn pause(&mut self) {
            self.r_stack.push(self.instruction_pointer);
            self.instruction_pointer = 0;
            self.is_paused = true;
        }

        pub fn abort(&mut self, msg: &str) {
            // TODO
        }

    }

    #[cfg(test)]
    mod tests {
        use super::VM;

        #[test]
        fn test_find() {
            let vm = &mut VM::new();
            vm.find("");
            assert_eq!(0isize, vm.found_index);
            vm.find("word-not-exist");
            assert_eq!(0isize, vm.found_index);
            vm.find("noop");
            assert_eq!(1isize, vm.found_index);
        }

        #[test]
        fn test_inner_interpreter_without_nest () {
            let vm = &mut VM::new();
            vm.find("noop");
            let idx = vm.found_index;
            vm.compile_word(idx);
            vm.compile_integer(3);
            vm.compile_integer(2);
            vm.compile_integer(1);
            vm.inner_interpret(1);
            assert_eq!(3usize, vm.s_stack.len());
        }
    }
}
