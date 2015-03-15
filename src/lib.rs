use std::str;
use std::num::SignedInt;

// Error messages
static S_STACK_UNDERFLOW: &'static str = "Data stack underflow";
static R_STACK_UNDERFLOW: &'static str = "Return stack underflow";

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
        vm.add_primitive("swap", VM::swap);
        vm.add_primitive("dup", VM::dup);
        vm.add_primitive("drop", VM::drop);
        vm.add_primitive("nip", VM::nip);
        vm.add_primitive("over", VM::over);
        vm.add_primitive("rot", VM::rot);
        vm.add_primitive("2drop", VM::two_drop);
        vm.add_primitive("2dup", VM::two_dup);
        vm.add_primitive("2swap", VM::two_swap);
        vm.add_primitive("2over", VM::two_over);
        vm.add_primitive("1+", VM::one_plus);
        vm.add_primitive("1-", VM::one_minus);
        vm.add_primitive("-", VM::minus);
        vm.add_primitive("+", VM::plus);
        vm.add_primitive("*", VM::star);
        vm.add_primitive("/", VM::slash);
        vm.add_primitive("mod", VM::p_mod);
        vm.add_primitive("/mod", VM::slash_mod);
        vm.add_primitive("abs", VM::abs);
        vm.add_primitive("negate", VM::negate);
        vm.add_primitive("0=", VM::zero_equals);
        vm.add_primitive("0<", VM::zero_less);
        vm.add_primitive("0>", VM::zero_greater);
        vm.add_primitive("0<>", VM::zero_not_equals);
//        vm.add_primitive("=", VM::equals);
//        vm.add_primitive("<", VM::less_than);
//        vm.add_primitive(">", VM::greater_than);
//        vm.add_primitive("invert", VM::invert);
//        vm.add_primitive("and", VM::and);
//        vm.add_primitive("or", VM::or);
//        vm.add_primitive("xor", VM::xor);
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

    pub fn swap(&mut self) {
        match self.s_stack.pop() {
            Some(t) =>
                match self.s_stack.pop() {
                    Some(n) => { self.s_stack.push(t); self.s_stack.push(n); },
                    None => self.abort(S_STACK_UNDERFLOW)
                },
            None => self.abort(S_STACK_UNDERFLOW)
        }
    }

    pub fn dup(&mut self) {
        match self.s_stack.pop() {
            Some(t) => { self.s_stack.push(t); self.s_stack.push(t); },
            None => self.abort(S_STACK_UNDERFLOW)
        }
    }

    pub fn drop(&mut self) {
        match self.s_stack.pop() {
            Some(t) => {},
            None => self.abort(S_STACK_UNDERFLOW)
        }
    }

    pub fn nip(&mut self) {
        match self.s_stack.pop() {
            Some(t) =>
                match self.s_stack.pop() {
                    Some(n) => { self.s_stack.push(t); },
                    None => self.abort(S_STACK_UNDERFLOW)
                },
            None => self.abort(S_STACK_UNDERFLOW)
        }
    }

    pub fn over(&mut self) {
        match self.s_stack.pop() {
            Some(t) =>
                match self.s_stack.pop() {
                    Some(n) => { self.s_stack.push(n); self.s_stack.push(t); self.s_stack.push(n); },
                    None => self.abort(S_STACK_UNDERFLOW)
                },
            None => self.abort(S_STACK_UNDERFLOW)
        }
    }

    pub fn rot(&mut self) {
        match self.s_stack.pop() {
            Some(t) =>
                match self.s_stack.pop() {
                    Some(n) =>
                        match self.s_stack.pop() {
                            Some(third) => { self.s_stack.push(n); self.s_stack.push(t); self.s_stack.push(third); },
                            None => self.abort(S_STACK_UNDERFLOW)
                        },
                    None => self.abort(S_STACK_UNDERFLOW)
                },
            None => self.abort(S_STACK_UNDERFLOW)
        }
    }

    pub fn two_drop(&mut self) {
        match self.s_stack.pop() {
            Some(t) =>
                match self.s_stack.pop() {
                    Some(n) => {},
                    None => self.abort(S_STACK_UNDERFLOW)
                },
            None => self.abort(S_STACK_UNDERFLOW)
        }
    }

    pub fn two_dup(&mut self) {
        match self.s_stack.pop() {
            Some(t) =>
                match self.s_stack.pop() {
                    Some(n) => {
                        self.s_stack.push(n);
                        self.s_stack.push(t);
                        self.s_stack.push(n);
                        self.s_stack.push(t);
                    },
                    None => self.abort(S_STACK_UNDERFLOW)
                },
            None => self.abort(S_STACK_UNDERFLOW)
        }
    }

    pub fn two_swap(&mut self) {
        match self.s_stack.pop() {
            Some(x4) =>
                match self.s_stack.pop() {
                    Some(x3) =>
                        match self.s_stack.pop() {
                            Some(x2) =>
                                match self.s_stack.pop() {
                                    Some(x1) => {
                                        self.s_stack.push(x3);
                                        self.s_stack.push(x4);
                                        self.s_stack.push(x1);
                                        self.s_stack.push(x2);
                                    },
                                    None => self.abort(S_STACK_UNDERFLOW)
                                },
                            None => self.abort(S_STACK_UNDERFLOW)
                        },
                    None => self.abort(S_STACK_UNDERFLOW)
                },
            None => self.abort(S_STACK_UNDERFLOW)
        }
    }

    pub fn two_over(&mut self) {
        match self.s_stack.pop() {
            Some(x4) =>
                match self.s_stack.pop() {
                    Some(x3) =>
                        match self.s_stack.pop() {
                            Some(x2) =>
                                match self.s_stack.pop() {
                                    Some(x1) => {
                                        self.s_stack.push(x1);
                                        self.s_stack.push(x2);
                                        self.s_stack.push(x3);
                                        self.s_stack.push(x4);
                                        self.s_stack.push(x1);
                                        self.s_stack.push(x2);
                                    },
                                    None => self.abort(S_STACK_UNDERFLOW)
                                },
                            None => self.abort(S_STACK_UNDERFLOW)
                        },
                    None => self.abort(S_STACK_UNDERFLOW)
                },
            None => self.abort(S_STACK_UNDERFLOW)
        }
    }

    pub fn one_plus(&mut self) {
        match self.s_stack.pop() {
            Some(t) =>
                self.s_stack.push(t+1),
            None => self.abort(S_STACK_UNDERFLOW)
        }
    }

    pub fn one_minus(&mut self) {
        match self.s_stack.pop() {
            Some(t) =>
                self.s_stack.push(t-1),
            None => self.abort(S_STACK_UNDERFLOW)
        }
    }

    pub fn plus(&mut self) {
        match self.s_stack.pop() {
            Some(t) =>
                match self.s_stack.pop() {
                    Some(n) => { self.s_stack.push(t+n); },
                    None => self.abort(S_STACK_UNDERFLOW)
                },
            None => self.abort(S_STACK_UNDERFLOW)
        }
    }

    pub fn minus(&mut self) {
        match self.s_stack.pop() {
            Some(t) =>
                match self.s_stack.pop() {
                    Some(n) => { self.s_stack.push(n-t); },
                    None => self.abort(S_STACK_UNDERFLOW)
                },
            None => self.abort(S_STACK_UNDERFLOW)
        }
    }

    pub fn star(&mut self) {
        match self.s_stack.pop() {
            Some(t) =>
                match self.s_stack.pop() {
                    Some(n) => { self.s_stack.push(n*t); },
                    None => self.abort(S_STACK_UNDERFLOW)
                },
            None => self.abort(S_STACK_UNDERFLOW)
        }
    }

    pub fn slash(&mut self) {
        match self.s_stack.pop() {
            Some(t) =>
                match self.s_stack.pop() {
                    Some(n) => { self.s_stack.push(n/t); },
                    None => self.abort(S_STACK_UNDERFLOW)
                },
            None => self.abort(S_STACK_UNDERFLOW)
        }
    }

    pub fn p_mod(&mut self) {
        match self.s_stack.pop() {
            Some(t) =>
                match self.s_stack.pop() {
                    Some(n) => { self.s_stack.push(n%t); },
                    None => self.abort(S_STACK_UNDERFLOW)
                },
            None => self.abort(S_STACK_UNDERFLOW)
        }
    }

    pub fn slash_mod(&mut self) {
        match self.s_stack.pop() {
            Some(t) =>
                match self.s_stack.pop() {
                    Some(n) => {
                        self.s_stack.push(n%t);
                        self.s_stack.push(n/t);
                    },
                    None => self.abort(S_STACK_UNDERFLOW)
                },
            None => self.abort(S_STACK_UNDERFLOW)
        }
    }

    pub fn abs(&mut self) {
        match self.s_stack.pop() {
            Some(t) => self.s_stack.push(t.abs()),
            None => self.abort(S_STACK_UNDERFLOW)
        }
    }

    pub fn negate(&mut self) {
        match self.s_stack.pop() {
            Some(t) => self.s_stack.push(-t),
            None => self.abort(S_STACK_UNDERFLOW)
        }
    }

    pub fn zero_less(&mut self) {
        match self.s_stack.pop() {
            Some(t) => self.s_stack.push(if t<0 {-1} else {0}),
            None => self.abort(S_STACK_UNDERFLOW)
        }
    }

    pub fn zero_equals(&mut self) {
        match self.s_stack.pop() {
            Some(t) => self.s_stack.push(if t==0 {-1} else {0}),
            None => self.abort(S_STACK_UNDERFLOW)
        }
    }

    pub fn zero_greater(&mut self) {
        match self.s_stack.pop() {
            Some(t) => self.s_stack.push(if t>0 {-1} else {0}),
            None => self.abort(S_STACK_UNDERFLOW)
        }
    }

    pub fn zero_not_equals(&mut self) {
        match self.s_stack.pop() {
            Some(t) => self.s_stack.push(if t!=0 {-1} else {0}),
            None => self.abort(S_STACK_UNDERFLOW)
        }
    }

    pub fn exit(&mut self) {
        match self.r_stack.pop() {
            None => self.abort (R_STACK_UNDERFLOW),
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

    #[test]
    fn test_drop() {
        let vm = &mut VM::new();
        vm.s_stack.push(1);
        vm.drop();
        assert!(vm.s_stack.len()==0);
    }

    #[test]
    fn test_nip() {
        let vm = &mut VM::new();
        vm.s_stack.push(1);
        vm.s_stack.push(2);
        vm.nip();
        assert!(vm.s_stack.len()==1);
        assert!(vm.s_stack.last() == Some(&2));
    }


    #[test]
    fn test_swap () {
        let vm = &mut VM::new();
        vm.s_stack.push(1);
        vm.s_stack.push(2);
        vm.swap();
        assert_eq!(vm.s_stack, [2,1]);
    }

    #[test]
    fn test_dup () {
        let vm = &mut VM::new();
        vm.s_stack.push(1);
        vm.dup();
        assert_eq!(vm.s_stack, [1, 1]);
    }

    #[test]
    fn test_over () {
        let vm = &mut VM::new();
        vm.s_stack.push(1);
        vm.s_stack.push(2);
        vm.over();
        assert_eq!(vm.s_stack, [1,2,1]);
    }

    #[test]
    fn test_rot () {
        let vm = &mut VM::new();
        vm.s_stack.push(1);
        vm.s_stack.push(2);
        vm.s_stack.push(3);
        vm.rot();
        assert_eq!(vm.s_stack, [2, 3, 1]);
    }

    #[test]
    fn test_2drop () {
        let vm = &mut VM::new();
        vm.s_stack.push(1);
        vm.s_stack.push(2);
        vm.two_drop();
        assert!(vm.s_stack.len()==0);
    }

    #[test]
    fn test_2dup () {
        let vm = &mut VM::new();
        vm.s_stack.push(1);
        vm.s_stack.push(2);
        vm.two_dup();
        assert_eq!(vm.s_stack, [1, 2, 1, 2]);
    }

    #[test]
    fn test_2swap () {
        let vm = &mut VM::new();
        vm.s_stack.push(1);
        vm.s_stack.push(2);
        vm.s_stack.push(3);
        vm.s_stack.push(4);
        vm.two_swap();
        assert_eq!(vm.s_stack, [3, 4, 1, 2]);
    }

    #[test]
    fn test_2over () {
        let vm = &mut VM::new();
        vm.s_stack.push(1);
        vm.s_stack.push(2);
        vm.s_stack.push(3);
        vm.s_stack.push(4);
        vm.two_over();
        assert_eq!(vm.s_stack, [1, 2, 3, 4, 1, 2]);
    }

    #[test]
    fn test_one_plus () {
        let vm = &mut VM::new();
        vm.s_stack.push(1);
        vm.one_plus();
        assert_eq!(vm.s_stack, [2]);
    }

    #[test]
    fn test_one_minus () {
        let vm = &mut VM::new();
        vm.s_stack.push(2);
        vm.one_minus();
        assert_eq!(vm.s_stack, [1]);
    }

    #[test]
    fn test_minus () {
        let vm = &mut VM::new();
        vm.s_stack.push(5);
        vm.s_stack.push(7);
        vm.minus();
        assert_eq!(vm.s_stack, [-2]);
    }

    #[test]
    fn test_plus () {
        let vm = &mut VM::new();
        vm.s_stack.push(5);
        vm.s_stack.push(7);
        vm.plus();
        assert_eq!(vm.s_stack, [12]);
    }

    #[test]
    fn test_star () {
        let vm = &mut VM::new();
        vm.s_stack.push(5);
        vm.s_stack.push(7);
        vm.star();
        assert_eq!(vm.s_stack, [35]);
    }

    #[test]
    fn test_slash () {
        let vm = &mut VM::new();
        vm.s_stack.push(30);
        vm.s_stack.push(7);
        vm.slash();
        assert_eq!(vm.s_stack, [4]);
    }

    #[test]
    fn test_mod () {
        let vm = &mut VM::new();
        vm.s_stack.push(30);
        vm.s_stack.push(7);
        vm.p_mod();
        assert_eq!(vm.s_stack, [2]);
    }

    #[test]
    fn test_slash_mod () {
        let vm = &mut VM::new();
        vm.s_stack.push(30);
        vm.s_stack.push(7);
        vm.slash_mod();
        assert_eq!(vm.s_stack, [2, 4]);
    }

    #[test]
    fn test_abs () {
        let vm = &mut VM::new();
        vm.s_stack.push(-30);
        vm.abs();
        assert_eq!(vm.s_stack, [30]);
    }

    #[test]
    fn test_negate () {
        let vm = &mut VM::new();
        vm.s_stack.push(30);
        vm.negate();
        assert_eq!(vm.s_stack, [-30]);
    }

    #[test]
    fn test_zero_less () {
        let vm = &mut VM::new();
        vm.s_stack.push(-1);
        vm.zero_less();
        assert_eq!(vm.s_stack, [-1]);
        vm.drop();
        vm.s_stack.push(0);
        vm.zero_less();
        assert_eq!(vm.s_stack, [0]);
    }

    #[test]
    fn test_zero_equals () {
        let vm = &mut VM::new();
        vm.s_stack.push(0);
        vm.zero_equals();
        assert_eq!(vm.s_stack, [-1]);
        vm.drop();
        vm.s_stack.push(-1);
        vm.zero_equals();
        assert_eq!(vm.s_stack, [0]);
        vm.drop();
        vm.s_stack.push(1);
        vm.zero_equals();
        assert_eq!(vm.s_stack, [0]);
    }

    #[test]
    fn test_zero_greater () {
        let vm = &mut VM::new();
        vm.s_stack.push(1);
        vm.zero_greater();
        assert_eq!(vm.s_stack, [-1]);
        vm.drop();
        vm.s_stack.push(0);
        vm.zero_greater();
        assert_eq!(vm.s_stack, [0]);
    }

    #[test]
    fn test_zero_not_equals () {
        let vm = &mut VM::new();
        vm.s_stack.push(0);
        vm.zero_not_equals();
        assert_eq!(vm.s_stack, [0]);
        vm.drop();
        vm.s_stack.push(-1);
        vm.zero_not_equals();
        assert_eq!(vm.s_stack, [-1]);
        vm.drop();
        vm.s_stack.push(1);
        vm.zero_not_equals();
        assert_eq!(vm.s_stack, [-1]);
    }

}
