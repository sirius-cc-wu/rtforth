use std::str;
use std::num::SignedInt;
use std::str::FromStr;

// Error messages
static S_STACK_UNDERFLOW: &'static str = "Data stack underflow";
static R_STACK_UNDERFLOW: &'static str = "Return stack underflow";
static WORD_NOT_FOUND: &'static str = "Word not found";
static END_OF_INPUT: &'static str = "End of input";

// Word
pub struct Word<'a, 'b> {
    is_immediate: bool,
    hidden: bool,
    nfa: usize,
    dfa: usize,
    name_len: usize,
    action: fn(& mut VM<'a, 'b>)
}

impl<'a, 'b> Word<'a, 'b> {
    pub fn new(nfa: usize, name_len: usize, dfa: usize, action: fn(& mut VM<'a, 'b>)) -> Word<'a, 'b> {
        Word {
            is_immediate: false,
            hidden: false,
            nfa: nfa,
            dfa: dfa,
            name_len: name_len,
            action: action
        }
    }
}

// Virtual machine
pub struct VM<'a, 'b> {
    is_compiling: bool,
    is_paused: bool,
    error_code: isize,
    pub s_stack: Vec<isize>,
    r_stack: Vec<usize>,
    f_stack: Vec<f64>,
    s_heap: Vec<isize>,
    f_heap: Vec<f64>,
    n_heap: String,
    word_list: Vec<Word<'a, 'b>>,
    instruction_pointer: usize,
    word_pointer: usize,
    idx_lit: usize,
    idx_exit: usize,
    idx_flit: usize,
    input_buffer: &'b str,
    input_index: usize,
    last_token: String,
    last_definition: usize
}

impl<'a, 'b> VM<'a, 'b> {
    pub fn new() -> VM<'a, 'b> {
        let mut vm = VM {
            is_compiling: false,
            is_paused: true,
            error_code: 0,
            s_stack: Vec::with_capacity(16),
            r_stack: Vec::with_capacity(16),
            f_stack: Vec::with_capacity(16),
            s_heap: Vec::with_capacity(64),
            f_heap: Vec::with_capacity(64),
            n_heap: String::with_capacity(64),
            word_list: Vec::with_capacity(16),
            instruction_pointer: 0,
            word_pointer: 0,
            idx_lit: 0,
            idx_exit: 0,
            idx_flit: 0,
            input_buffer: "",
            input_index: 0,
            last_token: String::with_capacity(64),
            last_definition: 0
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
        vm.add_primitive("not", VM::zero_equals);
        vm.add_primitive("=", VM::equals);
        vm.add_primitive("<", VM::less_than);
        vm.add_primitive(">", VM::greater_than);
        vm.add_primitive("<>", VM::not_equals);
        vm.add_primitive("between", VM::between);
        vm.add_primitive("invert", VM::invert);
        vm.add_primitive("and", VM::and);
        vm.add_primitive("or", VM::or);
        vm.add_primitive("xor", VM::xor);
        vm.add_primitive("flit", VM::flit);;
        vm.add_primitive(":", VM::colon);
        vm.add_immediate(";", VM::semicolon);
        vm.add_primitive("constant", VM::constant);
        vm.add_primitive("variable", VM::variable);
        vm.add_primitive("@", VM::fetch);
        vm.add_primitive("!", VM::store);
        vm.idx_lit = vm.find("lit");
        vm.idx_flit = vm.find("flit");
        vm.idx_flit = vm.find("exit");
        // S_heap is beginning with noop, because s_heap[0] should not be used.
        let idx = vm.find("noop");
        vm.compile_word(idx);
        vm
    }

    pub fn add_primitive(&mut self, name: &str, action: fn(& mut VM<'a, 'b>)) {
        self.word_list.push (Word::new(self.n_heap.len(), name.len(), self.s_heap.len(), action));
        self.n_heap.push_str(name);
    }

    pub fn add_immediate(&mut self, name: &str, action: fn(& mut VM<'a, 'b>)) {
        self.add_primitive (name, action);
        match self.word_list.last_mut() {
            Some(w) => w.is_immediate = true,
            None => { /* Impossible */ }
        };
    }

    pub fn execute_word(&mut self, i: usize) {
        self.word_pointer = i;
        (self.word_list[i].action)(self);
    }

    /// Find the word with name 'name'.
    /// If not found returns zero.
    pub fn find(&self, name: &str) -> usize {
        let mut i = 0usize;
        let mut found_index = 0usize;
        for w in self.word_list.iter() {
            let n = &self.n_heap[w.nfa .. w.nfa+w.name_len];
            if !w.hidden && n == name {
                found_index = i;
                break;
            } else {
                i += 1;
            }
        }
        return found_index;
    }

    pub fn words(&mut self) {
        for w in self.word_list.iter() {
            let s = &self.n_heap[w.nfa..w.nfa+w.name_len];
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
            let w = self.s_heap[self.instruction_pointer] as usize;
            self.instruction_pointer += 1;
            self.execute_word (w);
        }
        self.instruction_pointer = 0;
    }

// Compiler

    pub fn compile_word(&mut self, word_index: usize) {
        self.s_heap.push(word_index as isize);
    }

    /// Compile integer 'i'.
    pub fn compile_integer (&mut self, i: isize) {
        self.s_heap.push(self.idx_lit as isize);
        self.s_heap.push(i);
    }

    /// Compile float 'f'.
    pub fn compile_float (&mut self, f: f64) {
        self.s_heap.push(self.idx_flit as isize);
        self.f_heap.push(f);
        self.s_heap.push(self.f_heap.len() as isize);
    }

// Evaluation

    pub fn interpret(& mut self) {
        self.is_compiling = false;
    }

    pub fn compile(& mut self) {
        self.is_compiling = true;
    }

    pub fn scan(&mut self) {
        self.last_token.clear();
        let source = &self.input_buffer[self.input_index..self.input_buffer.len()];
        let mut cnt = 0;
        for ch in source.chars() {
            match ch {
                '\t' | '\n' | '\r' | ' ' => {
                    if !self.last_token.is_empty() {
                        break;
                    }
                },
                _ => self.last_token.push(ch)
            };
            cnt = cnt + 1;
        }
        self.input_index = self.input_index + cnt;
    }

    pub fn evaluate(&mut self, input_buffer: &'b str) {
        let saved_ip = self.instruction_pointer;
        let mut input_index = 0;
        self.instruction_pointer = 0;
        self.error_code = 0;
        self.input_index = 0;
        self.input_buffer = input_buffer;
        loop {
            self.scan();
            if self.last_token.is_empty() {
                break;
            }
            match FromStr::from_str(&self.last_token) {
                Ok(t) => {
                    if self.is_compiling {
                        self.compile_integer(t);
                    } else {
                        self.s_stack.push (t);
                    }
                    continue
                },
                Err(e) => {}
            };
            match FromStr::from_str(&self.last_token) {
                Ok(t) => {
                    if self.is_compiling {
                        self.compile_float(t);
                    } else {
                        self.f_stack.push (t);
                    }
                    continue
                },
                Err(e) => {}
            };
            let found_index = self.find(&self.last_token);
            if found_index != 0 {
                if !self.is_compiling || self.word_list[found_index].is_immediate {
                    self.execute_word(found_index);
                    if self.instruction_pointer != 0 {
                        self.inner();
                    }
                } else {
                    self.compile_word(found_index);
                }
            } else {
                self.abort(WORD_NOT_FOUND);
            }
            if self.has_error() {
                break;
            }
        }
        self.instruction_pointer = saved_ip;
    }

// High level definitions

    pub fn nest(&mut self) {
        self.r_stack.push(self.instruction_pointer);
        self.instruction_pointer = self.word_list[self.word_pointer].dfa;
    }

    pub fn p_var(&mut self) {
        self.s_stack.push(self.word_list[self.word_pointer].dfa as isize);
    }

    pub fn p_const(&mut self) {
        self.s_stack.push(self.s_heap[self.word_list[self.word_pointer].dfa]);
    }

    pub fn define(&mut self, action: fn(& mut VM<'a, 'b>)) {
        self.scan();
        if !self.last_token.is_empty() {
            let mut w = Word::new(self.n_heap.len(), self.last_token.len(), self.s_heap.len(), action);
            self.last_definition = self.word_list.len();
            self.word_list.push (w);
            self.n_heap.push_str(&self.last_token);
        } else {
            self.last_definition = 0;
            self.abort (END_OF_INPUT);
        }
    }

    pub fn colon(&mut self) {
        self.define(VM::nest);
        if self.last_definition != 0 {
            self.word_list[self.last_definition].hidden = true;
            self.compile();
        }
    }

    pub fn semicolon(&mut self) {
        if self.last_definition != 0 {
            self.s_heap.push(self.idx_exit as isize); 
            self.word_list[self.last_definition].hidden = false;
        }
        self.interpret();
    }

    pub fn variable(&mut self) {
        self.define(VM::p_var);
        self.s_heap.push(0);
    }

    pub fn constant(&mut self) {
        match self.s_stack.pop() {
            Some(v) => {
                self.define(VM::p_const);
                if self.last_definition != 0 {
                    self.s_heap.push(v);
                }
            },
            None => self.abort(S_STACK_UNDERFLOW)
        }
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

    pub fn flit(&mut self) {
        self.f_stack.push (self.f_heap[self.s_heap[self.instruction_pointer] as usize]);
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

    pub fn equals(&mut self) {
        match self.s_stack.pop() {
            Some(t) =>
                match self.s_stack.pop() {
                    Some(n) => self.s_stack.push(if t==n {-1} else {0}),
                    None => self.abort(S_STACK_UNDERFLOW)
                },
            None => self.abort(S_STACK_UNDERFLOW)
        }
    }

    pub fn less_than(&mut self) {
        match self.s_stack.pop() {
            Some(t) =>
                match self.s_stack.pop() {
                    Some(n) => self.s_stack.push(if n<t {-1} else {0}),
                    None => self.abort(S_STACK_UNDERFLOW)
                },
            None => self.abort(S_STACK_UNDERFLOW)
        }
    }

    pub fn greater_than(&mut self) {
        match self.s_stack.pop() {
            Some(t) =>
                match self.s_stack.pop() {
                    Some(n) => self.s_stack.push(if n>t {-1} else {0}),
                    None => self.abort(S_STACK_UNDERFLOW)
                },
            None => self.abort(S_STACK_UNDERFLOW)
        }
    }

    pub fn not_equals(&mut self) {
        match self.s_stack.pop() {
            Some(t) =>
                match self.s_stack.pop() {
                    Some(n) => self.s_stack.push(if n!=t {-1} else {0}),
                    None => self.abort(S_STACK_UNDERFLOW)
                },
            None => self.abort(S_STACK_UNDERFLOW)
        }
    }

    pub fn between(&mut self) {
        match self.s_stack.pop() {
            Some(x3) =>
                match self.s_stack.pop() {
                    Some(x2) =>
                        match self.s_stack.pop() {
                            Some(x1) => self.s_stack.push(if x1>=x2 && x1<=x3 {-1} else {0}),
                            None => self.abort(S_STACK_UNDERFLOW)
                        },
                    None => self.abort(S_STACK_UNDERFLOW)
                },
            None => self.abort(S_STACK_UNDERFLOW)
        }
    }

    pub fn invert(&mut self) {
        match self.s_stack.pop() {
            Some(t) => self.s_stack.push(!t),
            None => self.abort(S_STACK_UNDERFLOW)
        }
    }

    pub fn and(&mut self) {
        match self.s_stack.pop() {
            Some(t) =>
                match self.s_stack.pop() {
                    Some(n) => self.s_stack.push(t & n),
                    None => self.abort(S_STACK_UNDERFLOW)
                },
            None => self.abort(S_STACK_UNDERFLOW)
        }
    }

    pub fn or(&mut self) {
        match self.s_stack.pop() {
            Some(t) =>
                match self.s_stack.pop() {
                    Some(n) => self.s_stack.push(t | n),
                    None => self.abort(S_STACK_UNDERFLOW)
                },
            None => self.abort(S_STACK_UNDERFLOW)
        }
    }

    pub fn xor(&mut self) {
        match self.s_stack.pop() {
            Some(t) =>
                match self.s_stack.pop() {
                    Some(n) => self.s_stack.push(t ^ n),
                    None => self.abort(S_STACK_UNDERFLOW)
                },
            None => self.abort(S_STACK_UNDERFLOW)
        }
    }

    pub fn exit(&mut self) {
        match self.r_stack.pop() {
            None => self.abort (R_STACK_UNDERFLOW),
            Some(x) => self.instruction_pointer = x,
        }
    }

    pub fn fetch(&mut self) {
        match self.s_stack.pop() {
            Some(t) => self.s_stack.push(self.s_heap[t as usize]),
            None => self.abort(S_STACK_UNDERFLOW)
        };
    }

    pub fn store(&mut self) {
        match self.s_stack.pop() {
            Some(t) =>
                match self.s_stack.pop() {
                    Some(n) => { self.s_heap[t as usize] = n; },
                    None => self.abort(S_STACK_UNDERFLOW)
                },
            None => self.abort(S_STACK_UNDERFLOW)
        }
    }

    pub fn pause(&mut self) {
        self.r_stack.push(self.instruction_pointer);
        self.instruction_pointer = 0;
        self.is_paused = true;
    }

// Error handlling

    pub fn has_error(&self) -> bool {
        return self.error_code != 0;
    }

    pub fn abort(&mut self, msg: &str) {
        // TODO
        println!("{}", msg);
    }

}

#[cfg(test)]
mod tests {
    use super::VM;
    use std::str;

    #[test]
    fn test_find() {
        let vm = &mut VM::new();
        assert_eq!(0usize, vm.find(""));
        assert_eq!(0usize, vm.find("word-not-exist"));
        assert_eq!(1usize, vm.find("noop"));
    }

    #[test]
    fn test_inner_interpreter_without_nest () {
        let vm = &mut VM::new();
        let idx = vm.find("noop");
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

    #[test]
    fn test_less_than () {
        let vm = &mut VM::new();
        vm.s_stack.push(-1);
        vm.s_stack.push(0);
        vm.less_than();
        assert_eq!(vm.s_stack, [-1]);
        vm.drop();
        vm.s_stack.push(0);
        vm.s_stack.push(0);
        vm.less_than();
        assert_eq!(vm.s_stack, [0]);
    }

    #[test]
    fn test_equals () {
        let vm = &mut VM::new();
        vm.s_stack.push(0);
        vm.s_stack.push(0);
        vm.equals();
        assert_eq!(vm.s_stack, [-1]);
        vm.drop();
        vm.s_stack.push(-1);
        vm.s_stack.push(0);
        vm.equals();
        assert_eq!(vm.s_stack, [0]);
        vm.drop();
        vm.s_stack.push(1);
        vm.s_stack.push(0);
        vm.equals();
        assert_eq!(vm.s_stack, [0]);
    }

    #[test]
    fn test_greater_than () {
        let vm = &mut VM::new();
        vm.s_stack.push(1);
        vm.s_stack.push(0);
        vm.greater_than();
        assert_eq!(vm.s_stack, [-1]);
        vm.drop();
        vm.s_stack.push(0);
        vm.s_stack.push(0);
        vm.greater_than();
        assert_eq!(vm.s_stack, [0]);
    }

    #[test]
    fn test_not_equals () {
        let vm = &mut VM::new();
        vm.s_stack.push(0);
        vm.s_stack.push(0);
        vm.not_equals();
        assert_eq!(vm.s_stack, [0]);
        vm.drop();
        vm.s_stack.push(-1);
        vm.s_stack.push(0);
        vm.not_equals();
        assert_eq!(vm.s_stack, [-1]);
        vm.drop();
        vm.s_stack.push(1);
        vm.s_stack.push(0);
        vm.not_equals();
        assert_eq!(vm.s_stack, [-1]);
    }

    #[test]
    fn test_between () {
        let vm = &mut VM::new();
        vm.s_stack.push(1);
        vm.s_stack.push(1);
        vm.s_stack.push(2);
        vm.between();
        assert_eq!(vm.s_stack, [-1]);
        vm.drop();
        vm.two_drop();
        vm.s_stack.push(1);
        vm.s_stack.push(0);
        vm.s_stack.push(1);
        vm.between();
        assert_eq!(vm.s_stack, [-1]);
        vm.drop();
        vm.two_drop();
        vm.s_stack.push(0);
        vm.s_stack.push(1);
        vm.s_stack.push(2);
        vm.between();
        assert_eq!(vm.s_stack, [0]);
        vm.drop();
        vm.two_drop();
        vm.s_stack.push(3);
        vm.s_stack.push(1);
        vm.s_stack.push(2);
        vm.between();
        assert_eq!(vm.s_stack, [0]);
    }

    #[test]
    fn test_invert () {
        let vm = &mut VM::new();
        vm.s_stack.push(707);
        vm.invert();
        assert_eq!(vm.s_stack, [-708]);
    }

    #[test]
    fn test_and () {
        let vm = &mut VM::new();
        vm.s_stack.push(707);
        vm.s_stack.push(007);
        vm.and();
        assert_eq!(vm.s_stack, [3]);
    }

    #[test]
    fn test_or () {
        let vm = &mut VM::new();
        vm.s_stack.push(707);
        vm.s_stack.push(07);
        vm.or();
        assert_eq!(vm.s_stack, [711]);
    }

    #[test]
    fn test_xor () {
        let vm = &mut VM::new();
        vm.s_stack.push(707);
        vm.s_stack.push(07);
        vm.xor();
        assert_eq!(vm.s_stack, [708]);
    }

    #[test]
    fn test_scan () {
        let vm = &mut VM::new();
        vm.input_buffer = "hello world\t\r\n\"";
        vm.input_index = 0;
        vm.scan();
        assert_eq!(vm.last_token, "hello");
        assert_eq!(vm.input_index, 5);
        vm.scan();
        assert_eq!(vm.last_token, "world");
        assert_eq!(vm.input_index, 11);
        vm.scan();
        assert_eq!(vm.last_token, "\"");
    }

    #[test]
    fn test_evaluate () {
        let vm = &mut VM::new();
        let input_buffer = "false true dup 1+ 2 -3";
        vm.evaluate(input_buffer);
        assert_eq!(vm.s_stack, [0, -1, 0, 2, -3]);
    }

    #[test]
    fn test_evaluate_f64 () {
        let vm = &mut VM::new();
        let input_buffer = "1.0 2.5";
        vm.evaluate(input_buffer);
        assert_eq!(vm.f_stack.len(), 2);
        assert!(0.99999 < vm.f_stack[0]);
        assert!(vm.f_stack[0] < 1.00001);
        assert!(2.49999 < vm.f_stack[1]);
        assert!(vm.f_stack[1] < 2.50001);
    }

    #[test]
    fn test_colon_and_semi_colon() {
        let vm = &mut VM::new();
        let input_buffer = ": 2+3 2 3 + ; 2+3";
        vm.evaluate(input_buffer);
        assert_eq!(vm.s_stack, [5]);
    }

    #[test]
    fn test_constant () {
        let vm = &mut VM::new();
        let input_buffer = "5 constant x x x";
        vm.evaluate(input_buffer);
        assert_eq!(vm.s_stack, [5, 5]);
    }

    #[test]
    fn test_variable () {
        let vm = &mut VM::new();
        let input_buffer = "variable x  x @  3 x !  x @";
        vm.evaluate(input_buffer);
        assert_eq!(vm.s_stack, [0, 3]);
    }

}
