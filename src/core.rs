extern crate alloc;
extern crate test;

use self::alloc::{heap, oom};
use std::mem;
use std::ptr::{Unique, self};
use std::str::FromStr;
use exception::Exception;

use exception::Exception::{
    NoException,
    Abort,
    UnexpectedEndOfFile,
    UndefinedWord,
    StackOverflow,
    StackUnderflow,
    ReturnStackUnderflow,
    ReturnStackOverflow,
    UnsupportedOperation,
};

// Word
pub struct Word {
    is_immediate: bool,
    hidden: bool,
    pub nfa: usize,
    dfa: usize,
    pub name_len: usize,
    action: fn(& mut VM)
}

impl Word {
    pub fn new(nfa: usize, name_len: usize, dfa: usize, action: fn(& mut VM)) -> Word {
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

pub struct Stack<T> {
    ptr: Unique<T>,
    cap: usize,
    len: usize
}

impl<T> Stack<T> {
    fn with_capacity(cap: usize) -> Self {
        assert!(cap > 0 && cap <= 2048, "Invalid stack capacity");
        let align = mem::align_of::<isize>();
        let elem_size = mem::size_of::<isize>();
        unsafe {
            let ptr = heap::allocate(cap*elem_size, align);
            if ptr.is_null() { oom(); }
            Stack{ ptr: Unique::new(ptr as *mut _), cap: cap, len: 0 }
        }
    }

    pub fn push(&mut self, v: T) {
        if self.len >= self.cap {
// TODO: self is VM, not Stack.
//            self.abort_with_error(ReturnStackOverflow)
            assert!(self.len < self.cap, "Stack overflow");
        } else {
            unsafe {
                ptr::write(self.ptr.offset(self.len as isize), v);
            }
            self.len += 1;
        }
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.len == 0 {
// TODO: self is VM, not Stack.
//            self.abort_with_error(ReturnStackUnderflow)
            assert!(self.len > 0, "Stack underflow");
            None
        } else {
            self.len -= 1;
            unsafe {
                Some(ptr::read(self.ptr.offset(self.len as isize)))
            }
        }
    }

    pub fn push2(&mut self, v1: T, v2: T) {
        if self.len + 2 > self.cap {
// TODO: self is VM, not Stack.
//            self.abort_with_error(ReturnStackOverflow)
            assert!(self.len + 2  <= self.cap, "Stack overflow");
        } else {
            unsafe {
                ptr::write(self.ptr.offset(self.len as isize), v1);
                ptr::write(self.ptr.offset((self.len+1) as isize), v2);
            }
            self.len += 2;
        }
    }

    pub fn last(&self) -> Option<T> {
        if self.len == 0 {
// TODO: self is VM, not Stack.
//            self.abort_with_error(ReturnStackUnderflow)
            assert!(self.len > 0, "Stack underflow");
            None
        } else {
            unsafe {
                Some(ptr::read(self.ptr.offset((self.len-1) as isize)))
            }
        }
    }

    fn clear(&mut self) {
        self.len = 0;
    }

    pub fn len(&mut self) -> usize {
        self.len
    }
}

// Virtual machine
pub struct VM {
    is_compiling: bool,
    is_paused: bool,
    pub error_code: isize,
    pub s_stack: Stack<isize>,
    r_stack: Stack<isize>,
    pub f_stack: Vec<f64>,
    pub s_heap: Vec<isize>,
    pub f_heap: Vec<f64>,
    pub n_heap: String,
    pub word_list: Vec<Word>,
    instruction_pointer: usize,
    word_pointer: usize,
    pub idx_lit: usize,
    idx_exit: usize,
    pub idx_flit: usize,
    idx_zero_branch: usize,
    idx_branch: usize,
    pub idx_type: usize,
    pub input_buffer: String,
    pub source_index: usize,
    pub last_token: String,
    last_definition: usize,
    pub output_buffer: String
}

impl VM {
    pub fn new() -> VM {
        let mut vm = VM {
            is_compiling: false,
            is_paused: true,
            error_code: NoException as isize,
            s_stack: Stack::with_capacity(64),
            r_stack: Stack::with_capacity(64),
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
            idx_zero_branch: 0,
            idx_branch: 0,
            idx_type: 0,
            input_buffer: String::with_capacity(128),
            source_index: 0,
            last_token: String::with_capacity(64),
            last_definition: 0,
            output_buffer: String::with_capacity(128),
        };
        // index of 0 means not found.
        vm.add_primitive("", VM::noop);
        vm.add_primitive("noop", VM::noop);
        vm.add_primitive("true", VM::p_true);
        vm.add_primitive("false", VM::p_false);
        vm.add_primitive("cell+", VM::cell_plus);
        vm.add_primitive("cells", VM::cells);
        vm.add_primitive("align", VM::align);
        vm.add_primitive("aligned", VM::aligned);
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
        vm.add_primitive("parse-word", VM::parse_word);;
        vm.add_primitive("char", VM::char);
        vm.add_immediate("[char]", VM::bracket_char);
        vm.add_primitive("parse", VM::parse);
        vm.add_immediate("(", VM::imm_paren);
        vm.add_primitive("evaluate", VM::evaluate);;
        vm.add_primitive(":", VM::colon);
        vm.add_immediate(";", VM::semicolon);
        vm.add_primitive("constant", VM::constant);
        vm.add_primitive("variable", VM::variable);
        vm.add_primitive("create", VM::create);
        vm.add_primitive("@", VM::fetch);
        vm.add_primitive("!", VM::store);
        vm.add_primitive("'", VM::tick);
        vm.add_primitive("execute", VM::execute);
        vm.add_primitive("]", VM::compile);
        vm.add_immediate("[", VM::interpret);
        vm.add_primitive("here", VM::here);
        vm.add_primitive(",", VM::comma);
        vm.add_primitive(">r", VM::to_r);
        vm.add_primitive("r>", VM::r_from);
        vm.add_primitive("r@", VM::r_fetch);
        vm.add_primitive("2>r", VM::two_to_r);
        vm.add_primitive("2r>", VM::two_r_from);
        vm.add_primitive("2r@", VM::two_r_fetch);
        vm.add_primitive("0branch", VM::zero_branch);
        vm.add_primitive("branch", VM::branch);
        vm.add_immediate("if", VM::imm_if);
        vm.add_immediate("else", VM::imm_else);
        vm.add_immediate("then", VM::imm_then);
        vm.add_immediate("begin", VM::imm_begin);
        vm.add_immediate("while", VM::imm_while);
        vm.add_immediate("repeat", VM::imm_repeat);
        vm.add_immediate("again", VM::imm_again);
        vm.add_immediate("recurse", VM::imm_recurse);
        vm.add_immediate("\\", VM::imm_backslash);
        vm.add_primitive("marker", VM::marker);
        vm.add_primitive("quit", VM::quit);
        vm.add_primitive("abort", VM::abort);

        vm.idx_lit = vm.find("lit");
        vm.idx_exit = vm.find("exit");
        vm.idx_zero_branch = vm.find("0branch");
        vm.idx_branch = vm.find("branch");
        // S_heap is beginning with noop, because s_heap[0] should not be used.
        let idx = vm.find("noop");
        vm.compile_word(idx);
        vm
    }

    pub fn add_primitive(&mut self, name: &str, action: fn(& mut VM)) {
        self.word_list.push (Word::new(self.n_heap.len(), name.len(), self.s_heap.len(), action));
        self.n_heap.push_str(name);
    }

    pub fn add_immediate(&mut self, name: &str, action: fn(& mut VM)) {
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
    fn compile_integer (&mut self, i: isize) {
        self.s_heap.push(self.idx_lit as isize);
        self.s_heap.push(i);
    }

    /// Compile float 'f'.
    fn compile_float (&mut self, f: f64) {
        self.s_heap.push(self.idx_flit as isize);
        self.s_heap.push(self.f_heap.len() as isize);
        self.f_heap.push(f);
    }

// Evaluation

    pub fn interpret(& mut self) {
        self.is_compiling = false;
    }

    pub fn compile(& mut self) {
        self.is_compiling = true;
    }

    pub fn set_source(&mut self, s: &str) {
        self.input_buffer.clear();
        self.input_buffer.push_str(s);
        self.source_index = 0;
    }

    /// Run-time: ( "ccc" -- )
    ///
    /// Parse word delimited by white space, skipping leading white spaces.
    pub fn parse_word(&mut self) {
        self.last_token.clear();
        let source = &self.input_buffer[self.source_index..self.input_buffer.len()];
        let mut cnt = 0;
        for ch in source.chars() {
            cnt = cnt + 1;
            match ch {
                '\t' | '\n' | '\r' | ' ' => {
                    if !self.last_token.is_empty() {
                        break;
                    }
                },
                _ => self.last_token.push(ch)
            };
        }
        self.source_index = self.source_index + cnt;
    }

    /// Run-time: ( "&lt;spaces&gt;name" -- char)
    ///
    /// Skip leading space delimiters. Parse name delimited by a space. Put the value of its first character onto the stack.
    pub fn char(&mut self) {
        self.parse_word();
        match self.last_token.chars().nth(0) {
            Some(c) => self.s_stack.push(c as isize),
            // if none is found, push 0 onto stack.
            None => self.s_stack.push(0)
        }
    }

    /// Compilation: ( "&lt;spaces&gt;name" -- )
    ///
    /// Skip leading space delimiters. Parse name delimited by a space. Append the run-time semantics given below to the current definition.
    ///
    /// Run-time: ( -- char )
    ///
    /// Place char, the value of the first character of name, on the stack.
    pub fn bracket_char(&mut self) {
        self.char();
        match self.s_stack.pop() {
            Some(ch) => self.compile_integer(ch),
            None => self.abort_with_error(StackUnderflow)
        }
    }

    /// Run-time: ( char "ccc&lt;char&gt;" -- )
    ///
    /// Parse ccc delimited by the delimiter char.
    pub fn parse(&mut self) {
        match self.s_stack.pop() {
            Some(v) => {
                self.last_token.clear();
                let source = &self.input_buffer[self.source_index..self.input_buffer.len()];
                let mut cnt = 0;
                for ch in source.chars() {
                    cnt = cnt + 1;
                    if ch as isize == v {
                        break;
                    } else {
                        self.last_token.push(ch);
                    }
                }
                self.source_index = self.source_index + cnt;
            },
            None => {
                self.abort_with_error(StackUnderflow);
            }
        }
    }

    pub fn imm_paren(&mut self) {
        self.s_stack.push(')' as isize);
        self.parse();
    }

    pub fn imm_backslash(&mut self) {
        self.source_index = self.input_buffer.len(); 
    }

    pub fn evaluate(&mut self) {
        let saved_ip = self.instruction_pointer;
        self.instruction_pointer = 0;
        self.error_code = NoException as isize;
        loop {
            self.parse_word();
            if self.last_token.is_empty() {
                break;
            }
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
                // Integer?
                match FromStr::from_str(&self.last_token) {
                    Ok(t) => {
                        if self.is_compiling {
                            self.compile_integer(t);
                        } else {
                            self.s_stack.push (t);
                        }
                        continue
                    },
                    Err(_) => {
                        // Floating point?
                        match FromStr::from_str(&self.last_token) {
                            Ok(t) => {
                                if self.idx_flit == 0 {
                                    print!("{} ", "Floating point");
                                    self.abort_with_error(UnsupportedOperation);
                                } else {
                                    if self.is_compiling {
                                        self.compile_float(t);
                                    } else {
                                        self.f_stack.push (t);
                                    }
                                    continue
                                }
                            },
                            Err(_) => {
                                print!("{} ", &self.last_token);
                                self.abort_with_error(UndefinedWord);
                            }
                        };
                    }
                };

            }
            if self.has_error() {
                break;
            }
        }
        self.instruction_pointer = saved_ip;
    }

// High level definitions

    pub fn nest(&mut self) {
        if self.r_stack.len == self.r_stack.cap {
            self.abort_with_error(ReturnStackOverflow)
        } else {
            unsafe {
                ptr::write(self.r_stack.ptr.offset(self.r_stack.len as isize), self.instruction_pointer as isize);
            }
            self.r_stack.len += 1;
            self.instruction_pointer = self.word_list[self.word_pointer].dfa;
        }
    }

    pub fn p_var(&mut self) {
        self.s_stack.push(self.word_list[self.word_pointer].dfa as isize);
    }

    pub fn p_const(&mut self) {
        self.s_stack.push(self.s_heap[self.word_list[self.word_pointer].dfa]);
    }

    pub fn p_fconst(&mut self) {
        self.f_stack.push(self.f_heap[self.s_heap[self.word_list[self.word_pointer].dfa] as usize]);
    }

    pub fn p_fvar(&mut self) {
        self.s_stack.push(self.s_heap[self.word_list[self.word_pointer].dfa]);
    }

    pub fn define(&mut self, action: fn(& mut VM)) {
        self.parse_word();
        if !self.last_token.is_empty() {
            let w = Word::new(self.n_heap.len(), self.last_token.len(), self.s_heap.len(), action);
            self.last_definition = self.word_list.len();
            self.word_list.push (w);
            self.n_heap.push_str(&self.last_token);
        } else {
            self.last_definition = 0;
            self.abort_with_error (UnexpectedEndOfFile);
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

    pub fn create(&mut self) {
        self.define(VM::p_var);
    }

    pub fn variable(&mut self) {
        self.define(VM::p_var);
        self.s_heap.push(0);
    }

    pub fn constant(&mut self) {
        match self.s_stack.pop() {
            Some(v) => {
                self.define(VM::p_const);
                self.s_heap.push(v);
            },
            None => self.abort_with_error(StackUnderflow)
        }
    }

    pub fn unmark(&mut self) {
        let dfa = self.word_list[self.word_pointer].dfa;
        let flen = self.s_heap[dfa] as usize;
        let nlen = self.s_heap[dfa+1] as usize;
        let wlen = self.s_heap[dfa+2] as usize;
        let slen = self.s_heap[dfa+3] as usize;
        self.f_heap.truncate(flen);
        self.n_heap.truncate(nlen);
        self.word_list.truncate(wlen);
        self.s_heap.truncate(slen);
    }

    pub fn marker(&mut self) {
        self.define(VM::unmark);
        let flen = self.f_heap.len() as isize;
        let nlen = self.n_heap.len() as isize;
        let wlen = self.word_list.len() as isize;
        let slen = self.s_heap.len() as isize;
        self.s_heap.push(flen);
        self.s_heap.push(nlen);
        self.s_heap.push(wlen);
        self.s_heap.push(slen);
    }

// Control

    pub fn branch(&mut self) {
        self.instruction_pointer = ((self.instruction_pointer as isize)+ self.s_heap[self.instruction_pointer]) as usize;
    }

    pub fn zero_branch(&mut self) {
        match self.s_stack.pop() {
            Some(v) => {
                if v == 0 {
                    self.branch()
                } else {
                    self.instruction_pointer = self.instruction_pointer + 1;
                }
            },
            None => self.abort_with_error(StackUnderflow)
        }
    }

    pub fn imm_if(&mut self) {
        self.s_heap.push(self.idx_zero_branch as isize);
        self.s_heap.push(0);
        self.s_stack.push(self.s_heap.len() as isize);
    }

    pub fn imm_else(&mut self) {
        self.s_heap.push(self.idx_branch as isize);
        self.s_heap.push(0);
        match self.s_stack.pop() {
            Some(v) => {
                let if_part = v;
                let else_part = self.s_heap.len() as isize;
                self.s_heap[(if_part-1) as usize] = else_part-if_part+1;
                self.s_stack.push(else_part);
            },
            None => self.abort_with_error(StackUnderflow)
        };
    }

    pub fn imm_then(&mut self) {
        match self.s_stack.pop() {
            Some(v) => {
                let branch_part = v;
                self.s_heap[(branch_part-1) as usize] = (self.s_heap.len() as isize) - branch_part + 1;
            },
            None => self.abort_with_error(StackUnderflow)
        };
    }

    pub fn imm_begin(&mut self) {
        self.s_stack.push(self.s_heap.len() as isize);
    }

    pub fn imm_while(&mut self) {
        self.s_heap.push(self.idx_zero_branch as isize);
        self.s_stack.push(self.s_heap.len() as isize);
        self.s_heap.push(0);
    }

    pub fn imm_repeat(&mut self) {
        self.s_heap.push(self.idx_branch as isize);
        match self.s_stack.pop() {
            Some(while_part) => {
                match self.s_stack.pop() {
                    Some(begin_part) => {
                        let len = self.s_heap.len() as isize;
                        self.s_heap.push(begin_part-len);
                        self.s_heap[(while_part) as usize] = len - while_part + 1;
                    },
                    None => self.abort_with_error(StackUnderflow)
                };
            },
            None => self.abort_with_error(StackUnderflow)
        };
    }

    pub fn imm_again(&mut self) {
        self.s_heap.push(self.idx_branch as isize);
        match self.s_stack.pop() {
            Some(v) => {
                let len = self.s_heap.len() as isize;
                self.s_heap.push(v-len);
            },
            None => self.abort_with_error(StackUnderflow)
        };
    }

    pub fn imm_recurse(&mut self) {
        self.s_heap.push(self.last_definition as isize);
    }

// Primitives

    pub fn noop(&mut self) {
        // Do nothing
    }

    /// Run-time: ( -- true )
    ///
    /// Return a true flag, a single-cell value with all bits set. 
    pub fn p_true(&mut self) {
        self.s_stack.push (-1);
    }

    /// Run-time: ( -- false )
    ///
    /// Return a false flag.
    pub fn p_false(&mut self) {
        self.s_stack.push (0);
    }

    /// Run-time: (a-addr1 -- a-addr2 )
    ///
    /// Add the size in address units of a cell to a-addr1, giving a-addr2.
    pub fn cell_plus(&mut self) {
        match self.s_stack.pop() {
            Some(v) => self.s_stack.push(v + 1),
            None => self.abort_with_error(StackUnderflow)
        }
    }

    /// Run-time: (n1 -- n2 )
    ///
    /// n2 is the size in address units of n1 cells. 
    pub fn cells(&mut self) {
        // Do nothing.
    }

    /// Run-time: ( -- )
    ///
    /// If the data-space pointer is not aligned, reserve enough space to align it. 
    pub fn align(&mut self) {
        // Do nothing.
    }

    /// Run-time: (addr -- a-addr )
    ///
    /// a-addr is the first aligned address greater than or equal to addr. 
    pub fn aligned(&mut self) {
        // Do nothing.
    }

    pub fn lit(&mut self) {
        if self.s_stack.len >= self.s_stack.cap {
            self.abort_with_error(StackOverflow)
        } else {
            unsafe {
                ptr::write(self.s_stack.ptr.offset((self.s_stack.len) as isize), self.s_heap[self.instruction_pointer]);
            }
            self.s_stack.len += 1;
        }
        self.instruction_pointer = self.instruction_pointer + 1;
    }

    pub fn flit(&mut self) {
        self.f_stack.push (self.f_heap[self.s_heap[self.instruction_pointer] as usize]);
        self.instruction_pointer = self.instruction_pointer + 1;
    }

    pub fn swap(&mut self) {
        if self.s_stack.len < 2 {
            self.abort_with_error(StackUnderflow)
        } else {
            unsafe {
                let t = ptr::read(self.s_stack.ptr.offset((self.s_stack.len-1) as isize)); 
                ptr::write(self.s_stack.ptr.offset((self.s_stack.len-1) as isize), ptr::read(self.s_stack.ptr.offset((self.s_stack.len-2) as isize)));
                ptr::write(self.s_stack.ptr.offset((self.s_stack.len-2) as isize), t);
            }
        }
    }

    pub fn dup(&mut self) {
        if self.s_stack.len < 1 {
            self.abort_with_error(StackUnderflow)
        } else if self.s_stack.len >= self.s_stack.cap {
            self.abort_with_error(StackOverflow)
        } else {
            unsafe {
                ptr::write(self.s_stack.ptr.offset((self.s_stack.len) as isize), ptr::read(self.s_stack.ptr.offset((self.s_stack.len-1) as isize)));
                self.s_stack.len += 1;
            }
        }
    }

    pub fn drop(&mut self) {
        if self.s_stack.len < 1 {
            self.abort_with_error(StackUnderflow)
        } else {
            self.s_stack.len -= 1;
        }
    }

    pub fn nip(&mut self) {
        if self.s_stack.len < 2 {
            self.abort_with_error(StackUnderflow)
        } else {
            unsafe {
                self.s_stack.len -= 1;
                ptr::write(self.s_stack.ptr.offset((self.s_stack.len-1) as isize), ptr::read(self.s_stack.ptr.offset((self.s_stack.len) as isize)));
            }
        }
    }

    pub fn over(&mut self) {
        if self.s_stack.len < 2 {
            self.abort_with_error(StackUnderflow)
        } else if self.s_stack.len >= self.s_stack.cap {
            self.abort_with_error(StackOverflow)
        } else {
            unsafe {
                ptr::write(self.s_stack.ptr.offset((self.s_stack.len) as isize), ptr::read(self.s_stack.ptr.offset((self.s_stack.len-2) as isize)));
                self.s_stack.len += 1;
            }
        }
    }

    pub fn rot(&mut self) {
        if self.s_stack.len < 3 {
            self.abort_with_error(StackUnderflow)
        } else {
            unsafe {
                let t = ptr::read(self.s_stack.ptr.offset((self.s_stack.len-1) as isize)); 
                let n = ptr::read(self.s_stack.ptr.offset((self.s_stack.len-2) as isize)); 
                ptr::write(self.s_stack.ptr.offset((self.s_stack.len-1) as isize), ptr::read(self.s_stack.ptr.offset((self.s_stack.len-3) as isize)));
                ptr::write(self.s_stack.ptr.offset((self.s_stack.len-2) as isize), t);
                ptr::write(self.s_stack.ptr.offset((self.s_stack.len-3) as isize), n);
            }
        }
    }

    pub fn two_drop(&mut self) {
        if self.s_stack.len < 2 {
            self.abort_with_error(StackUnderflow)
        } else {
            self.s_stack.len -= 2;
        }
    }

    pub fn two_dup(&mut self) {
        if self.s_stack.len < 2 {
            self.abort_with_error(StackUnderflow)
        } else if self.s_stack.len + 2 > self.s_stack.cap {
            self.abort_with_error(StackOverflow)
        } else {
            unsafe {
                self.s_stack.len += 2;
                ptr::write(self.s_stack.ptr.offset((self.s_stack.len-1) as isize), ptr::read(self.s_stack.ptr.offset((self.s_stack.len-3) as isize)));
                ptr::write(self.s_stack.ptr.offset((self.s_stack.len-2) as isize), ptr::read(self.s_stack.ptr.offset((self.s_stack.len-4) as isize)));
            }
        }
    }

    pub fn two_swap(&mut self) {
        if self.s_stack.len < 4 {
            self.abort_with_error(StackUnderflow)
        } else {
            unsafe {
                let t = ptr::read(self.s_stack.ptr.offset((self.s_stack.len-1) as isize)); 
                let n = ptr::read(self.s_stack.ptr.offset((self.s_stack.len-2) as isize)); 
                ptr::write(self.s_stack.ptr.offset((self.s_stack.len-1) as isize), ptr::read(self.s_stack.ptr.offset((self.s_stack.len-3) as isize)));
                ptr::write(self.s_stack.ptr.offset((self.s_stack.len-2) as isize), ptr::read(self.s_stack.ptr.offset((self.s_stack.len-4) as isize)));
                ptr::write(self.s_stack.ptr.offset((self.s_stack.len-3) as isize), t);
                ptr::write(self.s_stack.ptr.offset((self.s_stack.len-4) as isize), n);
            }
        }
    }

    pub fn two_over(&mut self) {
        if self.s_stack.len < 4 {
            self.abort_with_error(StackUnderflow)
        } else if self.s_stack.len + 2 > self.s_stack.cap {
            self.abort_with_error(StackOverflow)
        } else {
            unsafe {
                self.s_stack.len += 2;
                ptr::write(self.s_stack.ptr.offset((self.s_stack.len-1) as isize), ptr::read(self.s_stack.ptr.offset((self.s_stack.len-5) as isize)));
                ptr::write(self.s_stack.ptr.offset((self.s_stack.len-2) as isize), ptr::read(self.s_stack.ptr.offset((self.s_stack.len-6) as isize)));
            }
        }
    }

    pub fn one_plus(&mut self) {
        if self.s_stack.len < 1 {
            self.abort_with_error(StackUnderflow)
        } else {
            unsafe {
                ptr::write(self.s_stack.ptr.offset((self.s_stack.len-1) as isize), ptr::read(self.s_stack.ptr.offset((self.s_stack.len-1) as isize)).wrapping_add(1));
            }
        }
    }

    pub fn one_minus(&mut self) {
        if self.s_stack.len < 1 {
            self.abort_with_error(StackUnderflow)
        } else {
            unsafe {
                ptr::write(self.s_stack.ptr.offset((self.s_stack.len-1) as isize), ptr::read(self.s_stack.ptr.offset((self.s_stack.len-1) as isize))-1);
            }
        }
    }

    pub fn plus(&mut self) {
        if self.s_stack.len < 2 {
            self.abort_with_error(StackUnderflow)
        } else {
            unsafe {
                self.s_stack.len -= 1;
                ptr::write(self.s_stack.ptr.offset((self.s_stack.len-1) as isize),
                    ptr::read(self.s_stack.ptr.offset((self.s_stack.len-1) as isize)) + ptr::read(self.s_stack.ptr.offset((self.s_stack.len) as isize)));
            }
        }
    }

    pub fn minus(&mut self) {
        if self.s_stack.len < 2 {
            self.abort_with_error(StackUnderflow)
        } else {
            unsafe {
                self.s_stack.len -= 1;
                ptr::write(self.s_stack.ptr.offset((self.s_stack.len-1) as isize),
                    ptr::read(self.s_stack.ptr.offset((self.s_stack.len-1) as isize)) - ptr::read(self.s_stack.ptr.offset((self.s_stack.len) as isize)));
            }
        }
    }

    pub fn star(&mut self) {
        if self.s_stack.len < 2 {
            self.abort_with_error(StackUnderflow)
        } else {
            unsafe {
                self.s_stack.len -= 1;
                ptr::write(self.s_stack.ptr.offset((self.s_stack.len-1) as isize),
                    ptr::read(self.s_stack.ptr.offset((self.s_stack.len-1) as isize)) * ptr::read(self.s_stack.ptr.offset((self.s_stack.len) as isize)));
            }
        }
    }

    pub fn slash(&mut self) {
        if self.s_stack.len < 2 {
            self.abort_with_error(StackUnderflow)
        } else {
            unsafe {
                self.s_stack.len -= 1;
                ptr::write(self.s_stack.ptr.offset((self.s_stack.len-1) as isize),
                    ptr::read(self.s_stack.ptr.offset((self.s_stack.len-1) as isize)) / ptr::read(self.s_stack.ptr.offset((self.s_stack.len) as isize)));
            }
        }
    }

    pub fn p_mod(&mut self) {
        if self.s_stack.len < 2 {
            self.abort_with_error(StackUnderflow)
        } else {
            unsafe {
                self.s_stack.len -= 1;
                ptr::write(self.s_stack.ptr.offset((self.s_stack.len-1) as isize),
                    ptr::read(self.s_stack.ptr.offset((self.s_stack.len-1) as isize)) % ptr::read(self.s_stack.ptr.offset((self.s_stack.len) as isize)));
            }
        }
    }

    pub fn slash_mod(&mut self) {
        if self.s_stack.len < 2 {
            self.abort_with_error(StackUnderflow)
        } else {
            unsafe {
                let t = ptr::read(self.s_stack.ptr.offset((self.s_stack.len-1) as isize)); 
                let n = ptr::read(self.s_stack.ptr.offset((self.s_stack.len-2) as isize)); 
                ptr::write(self.s_stack.ptr.offset((self.s_stack.len-2) as isize), n%t);
                ptr::write(self.s_stack.ptr.offset((self.s_stack.len-1) as isize), n/t);
            }
        }
    }

    pub fn abs(&mut self) {
        match self.s_stack.pop() {
            Some(t) => self.s_stack.push(t.abs()),
            None => self.abort_with_error(StackUnderflow)
        }
    }

    pub fn negate(&mut self) {
        match self.s_stack.pop() {
            Some(t) => self.s_stack.push(-t),
            None => self.abort_with_error(StackUnderflow)
        }
    }

    pub fn zero_less(&mut self) {
        match self.s_stack.pop() {
            Some(t) => self.s_stack.push(if t<0 {-1} else {0}),
            None => self.abort_with_error(StackUnderflow)
        }
    }

    pub fn zero_equals(&mut self) {
        match self.s_stack.pop() {
            Some(t) => self.s_stack.push(if t==0 {-1} else {0}),
            None => self.abort_with_error(StackUnderflow)
        }
    }

    pub fn zero_greater(&mut self) {
        match self.s_stack.pop() {
            Some(t) => self.s_stack.push(if t>0 {-1} else {0}),
            None => self.abort_with_error(StackUnderflow)
        }
    }

    pub fn zero_not_equals(&mut self) {
        match self.s_stack.pop() {
            Some(t) => self.s_stack.push(if t!=0 {-1} else {0}),
            None => self.abort_with_error(StackUnderflow)
        }
    }

    pub fn equals(&mut self) {
        match self.s_stack.pop() {
            Some(t) =>
                match self.s_stack.pop() {
                    Some(n) => self.s_stack.push(if t==n {-1} else {0}),
                    None => self.abort_with_error(StackUnderflow)
                },
            None => self.abort_with_error(StackUnderflow)
        }
    }

    pub fn less_than(&mut self) {
        match self.s_stack.pop() {
            Some(t) =>
                match self.s_stack.pop() {
                    Some(n) => self.s_stack.push(if n<t {-1} else {0}),
                    None => self.abort_with_error(StackUnderflow)
                },
            None => self.abort_with_error(StackUnderflow)
        }
    }

    pub fn greater_than(&mut self) {
        match self.s_stack.pop() {
            Some(t) =>
                match self.s_stack.pop() {
                    Some(n) => self.s_stack.push(if n>t {-1} else {0}),
                    None => self.abort_with_error(StackUnderflow)
                },
            None => self.abort_with_error(StackUnderflow)
        }
    }

    pub fn not_equals(&mut self) {
        match self.s_stack.pop() {
            Some(t) =>
                match self.s_stack.pop() {
                    Some(n) => self.s_stack.push(if n!=t {-1} else {0}),
                    None => self.abort_with_error(StackUnderflow)
                },
            None => self.abort_with_error(StackUnderflow)
        }
    }

    pub fn between(&mut self) {
        match self.s_stack.pop() {
            Some(x3) =>
                match self.s_stack.pop() {
                    Some(x2) =>
                        match self.s_stack.pop() {
                            Some(x1) => self.s_stack.push(if x1>=x2 && x1<=x3 {-1} else {0}),
                            None => self.abort_with_error(StackUnderflow)
                        },
                    None => self.abort_with_error(StackUnderflow)
                },
            None => self.abort_with_error(StackUnderflow)
        }
    }

    pub fn invert(&mut self) {
        match self.s_stack.pop() {
            Some(t) => self.s_stack.push(!t),
            None => self.abort_with_error(StackUnderflow)
        }
    }

    pub fn and(&mut self) {
        match self.s_stack.pop() {
            Some(t) =>
                match self.s_stack.pop() {
                    Some(n) => self.s_stack.push(t & n),
                    None => self.abort_with_error(StackUnderflow)
                },
            None => self.abort_with_error(StackUnderflow)
        }
    }

    pub fn or(&mut self) {
        match self.s_stack.pop() {
            Some(t) =>
                match self.s_stack.pop() {
                    Some(n) => self.s_stack.push(t | n),
                    None => self.abort_with_error(StackUnderflow)
                },
            None => self.abort_with_error(StackUnderflow)
        }
    }

    pub fn xor(&mut self) {
        match self.s_stack.pop() {
            Some(t) =>
                match self.s_stack.pop() {
                    Some(n) => self.s_stack.push(t ^ n),
                    None => self.abort_with_error(StackUnderflow)
                },
            None => self.abort_with_error(StackUnderflow)
        }
    }

    pub fn exit(&mut self) {
        if self.r_stack.len == 0 {
            self.abort_with_error(ReturnStackUnderflow)
        } else {
            self.r_stack.len -= 1;
            unsafe {
                self.instruction_pointer = ptr::read(self.r_stack.ptr.offset(self.r_stack.len as isize)) as usize; 
            }
        }
    }

    pub fn fetch(&mut self) {
        match self.s_stack.pop() {
            Some(t) => self.s_stack.push(self.s_heap[t as usize]),
            None => self.abort_with_error(StackUnderflow)
        };
    }

    pub fn store(&mut self) {
        match self.s_stack.pop() {
            Some(t) =>
                match self.s_stack.pop() {
                    Some(n) => { self.s_heap[t as usize] = n; },
                    None => self.abort_with_error(StackUnderflow)
                },
            None => self.abort_with_error(StackUnderflow)
        }
    }

    pub fn tick(&mut self) {
        self.parse_word();
        if !self.last_token.is_empty() {
            let found_index = self.find(&self.last_token);
            if found_index != 0 {
                self.s_stack.push(found_index as isize);
            } else {
                self.abort_with_error(UndefinedWord);
            }
        } else {
            self.abort_with_error(UnexpectedEndOfFile);
        }
    }

    pub fn execute(&mut self) {
        match self.s_stack.pop() {
            Some(t) => self.execute_word(t as usize),
            None => self.abort_with_error(StackUnderflow)
        };
    }

    pub fn here(&mut self) {
        self.s_stack.push(self.s_heap.len() as isize);
    }

    pub fn comma(&mut self) {
        match self.s_stack.pop() {
            Some(v) => self.s_heap.push(v),
            None => self.abort_with_error(StackUnderflow)
        }
    }

    pub fn to_r(&mut self) {
        match self.s_stack.pop() {
            Some(v) => {
                if self.r_stack.len >= self.r_stack.cap {
                    self.abort_with_error(ReturnStackOverflow)
                } else {
                    unsafe {
                        ptr::write(self.r_stack.ptr.offset(self.r_stack.len as isize), v);
                    }
                    self.r_stack.len += 1;
                }
            },
            None => self.abort_with_error(StackUnderflow)
        }
    }

    pub fn r_from(&mut self) {
        if self.r_stack.len == 0 {
            self.abort_with_error(ReturnStackUnderflow)
        } else {
            self.r_stack.len -= 1;
            unsafe {
                self.s_stack.push(ptr::read(self.r_stack.ptr.offset(self.r_stack.len as isize))); 
            }
        }
    }

    pub fn r_fetch(&mut self) {
        if self.r_stack.len == 0 {
            self.abort_with_error(ReturnStackUnderflow)
        } else {
            unsafe {
                self.s_stack.push(ptr::read(self.r_stack.ptr.offset((self.r_stack.len-1) as isize))); 
            }
        }
    }

    pub fn two_to_r(&mut self) {
        match self.s_stack.pop() {
            Some(t) =>
                match self.s_stack.pop() {
                    Some(n) => {
                        if self.r_stack.len >= self.r_stack.cap-1 {
                            self.abort_with_error(ReturnStackOverflow)
                        } else {
                            unsafe {
                                ptr::write(self.r_stack.ptr.offset(self.r_stack.len as isize), n);
                                ptr::write(self.r_stack.ptr.offset((self.r_stack.len+1) as isize), t);
                            }
                            self.r_stack.len += 2;
                        }
                    },
                    None => self.abort_with_error(StackUnderflow)
                },
            None => self.abort_with_error(StackUnderflow)
        }
    }

    pub fn two_r_from(&mut self) {
        if self.r_stack.len < 2 {
            self.abort_with_error(ReturnStackUnderflow)
        } else {
            self.r_stack.len -= 2;
            unsafe {
                self.s_stack.push(ptr::read(self.r_stack.ptr.offset(self.r_stack.len as isize))); 
                self.s_stack.push(ptr::read(self.r_stack.ptr.offset((self.r_stack.len+1) as isize))); 
            }
        }
    }

    pub fn two_r_fetch(&mut self) {
        if self.r_stack.len < 2 {
            self.abort_with_error(ReturnStackUnderflow)
        } else {
            unsafe {
                self.s_stack.push(ptr::read(self.r_stack.ptr.offset((self.r_stack.len-2) as isize))); 
                self.s_stack.push(ptr::read(self.r_stack.ptr.offset((self.r_stack.len-1) as isize))); 
            }
        }
    }

    pub fn pause(&mut self) {
        if self.r_stack.len == self.r_stack.cap {
            self.abort_with_error(ReturnStackOverflow)
        } else {
            unsafe {
                ptr::write(self.r_stack.ptr.offset(self.r_stack.len as isize), self.instruction_pointer as isize);
            }
            self.r_stack.len += 1;
            self.instruction_pointer = 0;
            self.is_paused = true;
        }
    }

// Error handlling

    pub fn has_error(&self) -> bool {
        return self.error_code != NoException as isize;
    }

    pub fn abort_with_error(&mut self, e: Exception) {
        println!("{}", e.name());
        self.abort();
        self.error_code = e as isize;
    }

    pub fn abort(&mut self) {
        self.s_stack.clear();
        self.f_stack.clear();
        self.error_code = Abort as isize;
        self.quit();
    }

    pub fn quit(&mut self) {
        self.r_stack.len = 0;
        self.input_buffer.clear();
        self.source_index = 0;
        self.instruction_pointer = 0;
        self.last_definition = 0;
        self.is_paused = false;
        self.interpret();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::test::Bencher;

    #[bench]
    fn bench_noop (b: &mut Bencher) {
        let vm = &mut VM::new();
        b.iter(|| vm.noop());
    }

    #[test]
    fn test_find() {
        let vm = &mut VM::new();
        assert_eq!(0usize, vm.find(""));
        assert_eq!(0usize, vm.find("word-not-exist"));
        assert_eq!(1usize, vm.find("noop"));
        assert_eq!(vm.error_code, 0);
    }

    #[bench]
    fn bench_find_word_not_exist(b: &mut Bencher) {
        let vm = &mut VM::new();
        b.iter(|| vm.find("unknown"));
    }

    #[bench]
    fn bench_find_word_at_beginning_of_wordlist(b: &mut Bencher) {
        let vm = &mut VM::new();
        b.iter(|| vm.find("noop"));
    }

    #[bench]
    fn bench_find_word_at_middle_of_wordlist(b: &mut Bencher) {
        let vm = &mut VM::new();
        b.iter(|| vm.find("branch"));
    }

    #[bench]
    fn bench_find_word_at_end_of_wordlist(b: &mut Bencher) {
        let vm = &mut VM::new();
        b.iter(|| vm.find("f<"));
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
        assert_eq!(vm.error_code, 0);
    }

    #[bench]
    fn bench_inner_interpreter_without_nest (b: &mut Bencher) {
        let vm = &mut VM::new();
        let idx = vm.find("noop");
        vm.compile_word(idx);
        vm.compile_word(idx);
        vm.compile_word(idx);
        vm.compile_word(idx);
        vm.compile_word(idx);
        vm.compile_word(idx);
        vm.compile_word(idx);
        b.iter(|| vm.inner_interpret(1));
    }

    #[test]
    fn test_drop() {
        let vm = &mut VM::new();
        vm.s_stack.push(1);
        vm.drop();
        assert!(vm.s_stack.len()==0);
        assert_eq!(vm.error_code, 0);
    }

    #[bench]
    fn bench_drop(b: &mut Bencher) {
        let vm = &mut VM::new();
        vm.s_stack.push(1);
        b.iter(|| {
            vm.drop();
            vm.s_stack.push(1);
        });
    }

    #[test]
    fn test_nip() {
        let vm = &mut VM::new();
        vm.s_stack.push(1);
        vm.s_stack.push(2);
        vm.nip();
        assert!(vm.s_stack.len()==1);
        assert!(vm.s_stack.last() == Some(2));
        assert_eq!(vm.error_code, 0);
    }

    #[bench]
    fn bench_nip(b: &mut Bencher) {
        let vm = &mut VM::new();
        vm.s_stack.push(1);
        vm.s_stack.push(1);
        b.iter(|| {
            vm.nip();
            vm.s_stack.push(1);
        });
    }

    #[test]
    fn test_swap () {
        let vm = &mut VM::new();
        vm.s_stack.push(1);
        vm.s_stack.push(2);
        vm.swap();
        assert_eq!(vm.s_stack.len(), 2);
        assert_eq!(vm.s_stack.pop(), Some(1));
        assert_eq!(vm.s_stack.pop(), Some(2));
        assert_eq!(vm.error_code, 0);
    }

    #[bench]
    fn bench_swap (b: &mut Bencher) {
        let vm = &mut VM::new();
        vm.s_stack.push(1);
        vm.s_stack.push(2);
        b.iter(|| vm.swap());
    }

    #[test]
    fn test_dup () {
        let vm = &mut VM::new();
        vm.s_stack.push(1);
        vm.dup();
        assert_eq!(vm.s_stack.len(), 2);
        assert_eq!(vm.s_stack.pop(), Some(1));
        assert_eq!(vm.s_stack.pop(), Some(1));
        assert_eq!(vm.error_code, 0);
    }

    #[bench]
    fn bench_dup (b: &mut Bencher) {
        let vm = &mut VM::new();
        vm.s_stack.push(1);
        b.iter(|| {
            vm.dup();
            vm.s_stack.pop();
        });
    }

    #[test]
    fn test_over () {
        let vm = &mut VM::new();
        vm.s_stack.push(1);
        vm.s_stack.push(2);
        vm.over();
        assert_eq!(vm.s_stack.len(), 3);
        assert_eq!(vm.s_stack.pop(), Some(1));
        assert_eq!(vm.s_stack.pop(), Some(2));
        assert_eq!(vm.s_stack.pop(), Some(1));
        assert_eq!(vm.error_code, 0);
    }

    #[bench]
    fn bench_over (b: &mut Bencher) {
        let vm = &mut VM::new();
        vm.s_stack.push(1);
        vm.s_stack.push(2);
        b.iter(|| {
            vm.over();
            vm.s_stack.pop();
        });
    }

    #[test]
    fn test_rot () {
        let vm = &mut VM::new();
        vm.s_stack.push(1);
        vm.s_stack.push(2);
        vm.s_stack.push(3);
        vm.rot();
        assert_eq!(vm.s_stack.len(), 3);
        assert_eq!(vm.s_stack.pop(), Some(1));
        assert_eq!(vm.s_stack.pop(), Some(3));
        assert_eq!(vm.s_stack.pop(), Some(2));
        assert_eq!(vm.error_code, 0);
    }

    #[bench]
    fn bench_rot (b: &mut Bencher) {
        let vm = &mut VM::new();
        vm.s_stack.push(1);
        vm.s_stack.push(2);
        vm.s_stack.push(3);
        b.iter(|| vm.rot());
    }

    #[test]
    fn test_2drop () {
        let vm = &mut VM::new();
        vm.s_stack.push(1);
        vm.s_stack.push(2);
        vm.two_drop();
        assert!(vm.s_stack.len()==0);
        assert_eq!(vm.error_code, 0);
    }

    #[bench]
    fn bench_2drop (b: &mut Bencher) {
        let vm = &mut VM::new();
        b.iter(|| {
            vm.s_stack.push(1);
            vm.s_stack.push(2);
            vm.two_drop();
        });
    }

    #[test]
    fn test_2dup () {
        let vm = &mut VM::new();
        vm.s_stack.push(1);
        vm.s_stack.push(2);
        vm.two_dup();
        assert_eq!(vm.s_stack.len(), 4);
        assert_eq!(vm.s_stack.pop(), Some(2));
        assert_eq!(vm.s_stack.pop(), Some(1));
        assert_eq!(vm.s_stack.pop(), Some(2));
        assert_eq!(vm.s_stack.pop(), Some(1));
        assert_eq!(vm.error_code, 0);
    }

    #[bench]
    fn bench_2dup (b: &mut Bencher) {
        let vm = &mut VM::new();
        vm.s_stack.push(1);
        vm.s_stack.push(2);
        b.iter(|| {
            vm.two_dup();
            vm.two_drop();
        });
    }

    #[test]
    fn test_2swap () {
        let vm = &mut VM::new();
        vm.s_stack.push(1);
        vm.s_stack.push(2);
        vm.s_stack.push(3);
        vm.s_stack.push(4);
        vm.two_swap();
        assert_eq!(vm.s_stack.len(), 4);
        assert_eq!(vm.s_stack.pop(), Some(2));
        assert_eq!(vm.s_stack.pop(), Some(1));
        assert_eq!(vm.s_stack.pop(), Some(4));
        assert_eq!(vm.s_stack.pop(), Some(3));
        assert_eq!(vm.error_code, 0);
    }

    #[bench]
    fn bench_2swap (b: &mut Bencher) {
        let vm = &mut VM::new();
        vm.s_stack.push(1);
        vm.s_stack.push(2);
        vm.s_stack.push(3);
        vm.s_stack.push(4);
        b.iter(|| vm.two_swap());
    }

    #[bench]
    fn bench_2over (b: &mut Bencher) {
        let vm = &mut VM::new();
        vm.s_stack.push(1);
        vm.s_stack.push(2);
        vm.s_stack.push(3);
        vm.s_stack.push(4);
        b.iter(|| {
            vm.two_over();
            vm.two_drop();
        });
    }

    #[test]
    fn test_one_plus() {
        let vm = &mut VM::new();
        vm.s_stack.push(1);
        vm.one_plus();
        assert_eq!(vm.s_stack.len(), 1);
        assert_eq!(vm.s_stack.pop(), Some(2));
        assert_eq!(vm.error_code, 0);
    }

    #[bench]
    fn bench_one_plus(b: &mut Bencher) {
        let vm = &mut VM::new();
        vm.s_stack.push(0);
        b.iter(|| {
            vm.one_plus();
        });
    }

    #[test]
    fn test_one_minus() {
        let vm = &mut VM::new();
        vm.s_stack.push(2);
        vm.one_minus();
        assert_eq!(vm.s_stack.len(), 1);
        assert_eq!(vm.s_stack.pop(), Some(1));
        assert_eq!(vm.error_code, 0);
    }

    #[bench]
    fn bench_one_minus(b: &mut Bencher) {
        let vm = &mut VM::new();
        vm.s_stack.push(0);
        b.iter(|| {
            vm.one_minus();
        });
    }

    #[test]
    fn test_minus() {
        let vm = &mut VM::new();
        vm.s_stack.push(5);
        vm.s_stack.push(7);
        vm.minus();
        assert_eq!(vm.s_stack.len(), 1);
        assert_eq!(vm.s_stack.pop(), Some(-2));
        assert_eq!(vm.error_code, 0);
    }

    #[bench]
    fn bench_minus(b: &mut Bencher) {
        let vm = &mut VM::new();
        vm.s_stack.push(0);
        b.iter(|| {
            vm.dup();
            vm.minus();
        });
    }

    #[test]
    fn test_plus() {
        let vm = &mut VM::new();
        vm.s_stack.push(5);
        vm.s_stack.push(7);
        vm.plus();
        assert_eq!(vm.s_stack.len(), 1);
        assert_eq!(vm.s_stack.pop(), Some(12));
        assert_eq!(vm.error_code, 0);
    }

    #[bench]
    fn bench_plus(b: &mut Bencher) {
        let vm = &mut VM::new();
        vm.s_stack.push(1);
        b.iter(|| {
            vm.dup();
            vm.plus();
        });
    }

    #[test]
    fn test_star () {
        let vm = &mut VM::new();
        vm.s_stack.push(5);
        vm.s_stack.push(7);
        vm.star();
        assert_eq!(vm.s_stack.len(), 1);
        assert_eq!(vm.s_stack.pop(), Some(35));
        assert_eq!(vm.error_code, 0);
    }

    #[bench]
    fn bench_star(b: &mut Bencher) {
        let vm = &mut VM::new();
        vm.s_stack.push(1);
        b.iter(|| {
            vm.dup();
            vm.star();
        });
    }

    #[test]
    fn test_slash () {
        let vm = &mut VM::new();
        vm.s_stack.push(30);
        vm.s_stack.push(7);
        vm.slash();
        assert_eq!(vm.s_stack.len(), 1);
        assert_eq!(vm.s_stack.pop(), Some(4));
        assert_eq!(vm.error_code, 0);
    }

    #[bench]
    fn bench_slash(b: &mut Bencher) {
        let vm = &mut VM::new();
        vm.s_stack.push(1);
        b.iter(|| {
            vm.dup();
            vm.slash();
        });
    }

    #[test]
    fn test_mod () {
        let vm = &mut VM::new();
        vm.s_stack.push(30);
        vm.s_stack.push(7);
        vm.p_mod();
        assert_eq!(vm.s_stack.len(), 1);
        assert_eq!(vm.s_stack.pop(), Some(2));
        assert_eq!(vm.error_code, 0);
    }

    #[bench]
    fn bench_mod(b: &mut Bencher) {
        let vm = &mut VM::new();
        vm.s_stack.push(1);
        vm.s_stack.push(2);
        b.iter(|| {
            vm.p_mod();
            vm.s_stack.push(2);
        });
    }

    #[test]
    fn test_slash_mod () {
        let vm = &mut VM::new();
        vm.s_stack.push(30);
        vm.s_stack.push(7);
        vm.slash_mod();
        assert_eq!(vm.s_stack.len(), 2);
        assert_eq!(vm.s_stack.pop(), Some(4));
        assert_eq!(vm.s_stack.pop(), Some(2));
        assert_eq!(vm.error_code, 0);
    }

    #[bench]
    fn bench_slash_mod(b: &mut Bencher) {
        let vm = &mut VM::new();
        vm.s_stack.push2(1, 2);
        b.iter(|| {
            vm.slash_mod();
            vm.drop();
            vm.s_stack.push(2)
        });
    }

    #[test]
    fn test_abs () {
        let vm = &mut VM::new();
        vm.s_stack.push(-30);
        vm.abs();
        assert_eq!(vm.s_stack.len(), 1);
        assert_eq!(vm.s_stack.pop(), Some(30));
        assert_eq!(vm.error_code, 0);
    }

    #[test]
    fn test_negate () {
        let vm = &mut VM::new();
        vm.s_stack.push(30);
        vm.negate();
        assert_eq!(vm.s_stack.len(), 1);
        assert_eq!(vm.s_stack.pop(), Some(-30));
        assert_eq!(vm.error_code, 0);
    }

    #[test]
    fn test_zero_less () {
        let vm = &mut VM::new();
        vm.s_stack.push(-1);
        vm.zero_less();
        assert_eq!(vm.s_stack.len(), 1);
        assert_eq!(vm.s_stack.pop(), Some(-1));
        vm.s_stack.push(0);
        vm.zero_less();
        assert_eq!(vm.s_stack.len(), 1);
        assert_eq!(vm.s_stack.pop(), Some(0));
        assert_eq!(vm.error_code, 0);
    }

    #[test]
    fn test_zero_equals () {
        let vm = &mut VM::new();
        vm.s_stack.push(0);
        vm.zero_equals();
        assert_eq!(vm.s_stack.len(), 1);
        assert_eq!(vm.s_stack.pop(), Some(-1));
        vm.s_stack.push(-1);
        vm.zero_equals();
        assert_eq!(vm.s_stack.len(), 1);
        assert_eq!(vm.s_stack.pop(), Some(0));
        vm.s_stack.push(1);
        vm.zero_equals();
        assert_eq!(vm.s_stack.len(), 1);
        assert_eq!(vm.s_stack.pop(), Some(0));
        assert_eq!(vm.error_code, 0);
    }

    #[test]
    fn test_zero_greater () {
        let vm = &mut VM::new();
        vm.s_stack.push(1);
        vm.zero_greater();
        assert_eq!(vm.s_stack.len(), 1);
        assert_eq!(vm.s_stack.pop(), Some(-1));
        vm.s_stack.push(0);
        vm.zero_greater();
        assert_eq!(vm.s_stack.len(), 1);
        assert_eq!(vm.s_stack.pop(), Some(0));
        assert_eq!(vm.error_code, 0);
    }

    #[test]
    fn test_zero_not_equals () {
        let vm = &mut VM::new();
        vm.s_stack.push(0);
        vm.zero_not_equals();
        assert_eq!(vm.s_stack.len(), 1);
        assert_eq!(vm.s_stack.pop(), Some(0));
        vm.s_stack.push(-1);
        vm.zero_not_equals();
        assert_eq!(vm.s_stack.len(), 1);
        assert_eq!(vm.s_stack.pop(), Some(-1));
        vm.s_stack.push(1);
        vm.zero_not_equals();
        assert_eq!(vm.s_stack.len(), 1);
        assert_eq!(vm.s_stack.pop(), Some(-1));
        assert_eq!(vm.error_code, 0);
    }

    #[test]
    fn test_less_than () {
        let vm = &mut VM::new();
        vm.s_stack.push(-1);
        vm.s_stack.push(0);
        vm.less_than();
        assert_eq!(vm.s_stack.len(), 1);
        assert_eq!(vm.s_stack.pop(), Some(-1));
        vm.s_stack.push(0);
        vm.s_stack.push(0);
        vm.less_than();
        assert_eq!(vm.s_stack.len(), 1);
        assert_eq!(vm.s_stack.pop(), Some(0));
        assert_eq!(vm.error_code, 0);
    }

    #[test]
    fn test_equals () {
        let vm = &mut VM::new();
        vm.s_stack.push(0);
        vm.s_stack.push(0);
        vm.equals();
        assert_eq!(vm.s_stack.len(), 1);
        assert_eq!(vm.s_stack.pop(), Some(-1));
        vm.s_stack.push(-1);
        vm.s_stack.push(0);
        vm.equals();
        assert_eq!(vm.s_stack.len(), 1);
        assert_eq!(vm.s_stack.pop(), Some(0));
        vm.s_stack.push(1);
        vm.s_stack.push(0);
        vm.equals();
        assert_eq!(vm.s_stack.len(), 1);
        assert_eq!(vm.s_stack.pop(), Some(0));
        assert_eq!(vm.error_code, 0);
    }

    #[test]
    fn test_greater_than () {
        let vm = &mut VM::new();
        vm.s_stack.push(1);
        vm.s_stack.push(0);
        vm.greater_than();
        assert_eq!(vm.s_stack.len(), 1);
        assert_eq!(vm.s_stack.pop(), Some(-1));
        vm.s_stack.push(0);
        vm.s_stack.push(0);
        vm.greater_than();
        assert_eq!(vm.s_stack.len(), 1);
        assert_eq!(vm.s_stack.pop(), Some(0));
        assert_eq!(vm.error_code, 0);
    }

    #[test]
    fn test_not_equals () {
        let vm = &mut VM::new();
        vm.s_stack.push(0);
        vm.s_stack.push(0);
        vm.not_equals();
        assert_eq!(vm.s_stack.len(), 1);
        assert_eq!(vm.s_stack.pop(), Some(0));
        vm.s_stack.push(-1);
        vm.s_stack.push(0);
        vm.not_equals();
        assert_eq!(vm.s_stack.len(), 1);
        assert_eq!(vm.s_stack.pop(), Some(-1));
        vm.s_stack.push(1);
        vm.s_stack.push(0);
        vm.not_equals();
        assert_eq!(vm.s_stack.len(), 1);
        assert_eq!(vm.s_stack.pop(), Some(-1));
        assert_eq!(vm.error_code, 0);
    }

    #[test]
    fn test_between () {
        let vm = &mut VM::new();
        vm.s_stack.push(1);
        vm.s_stack.push(1);
        vm.s_stack.push(2);
        vm.between();
        assert_eq!(vm.s_stack.len(), 1);
        assert_eq!(vm.s_stack.pop(), Some(-1));
        assert_eq!(vm.error_code, 0);
        vm.s_stack.push(1);
        vm.s_stack.push(0);
        vm.s_stack.push(1);
        vm.between();
        assert_eq!(vm.s_stack.len(), 1);
        assert_eq!(vm.s_stack.pop(), Some(-1));
        assert_eq!(vm.error_code, 0);
        vm.s_stack.push(0);
        vm.s_stack.push(1);
        vm.s_stack.push(2);
        vm.between();
        assert_eq!(vm.s_stack.len(), 1);
        assert_eq!(vm.s_stack.pop(), Some(0));
        assert_eq!(vm.error_code, 0);
        vm.s_stack.push(3);
        vm.s_stack.push(1);
        vm.s_stack.push(2);
        vm.between();
        assert_eq!(vm.s_stack.len(), 1);
        assert_eq!(vm.s_stack.pop(), Some(0));
        assert_eq!(vm.error_code, 0);
    }

    #[test]
    fn test_invert () {
        let vm = &mut VM::new();
        vm.s_stack.push(707);
        vm.invert();
        assert_eq!(vm.s_stack.len(), 1);
        assert_eq!(vm.s_stack.pop(), Some(-708));
        assert_eq!(vm.error_code, 0);
    }

    #[test]
    fn test_and () {
        let vm = &mut VM::new();
        vm.s_stack.push(707);
        vm.s_stack.push(007);
        vm.and();
        assert_eq!(vm.s_stack.len(), 1);
        assert_eq!(vm.s_stack.pop(), Some(3));
        assert_eq!(vm.error_code, 0);
    }

    #[test]
    fn test_or () {
        let vm = &mut VM::new();
        vm.s_stack.push(707);
        vm.s_stack.push(07);
        vm.or();
        assert_eq!(vm.s_stack.len(), 1);
        assert_eq!(vm.s_stack.pop(), Some(711));
        assert_eq!(vm.error_code, 0);
    }

    #[test]
    fn test_xor () {
        let vm = &mut VM::new();
        vm.s_stack.push(707);
        vm.s_stack.push(07);
        vm.xor();
        assert_eq!(vm.s_stack.len(), 1);
        assert_eq!(vm.s_stack.pop(), Some(708));
        assert_eq!(vm.error_code, 0);
    }

    #[test]
    fn test_parse_word () {
        let vm = &mut VM::new();
        vm.set_source("hello world\t\r\n\"");
        vm.parse_word();
        assert_eq!(vm.last_token, "hello");
        assert_eq!(vm.source_index, 6);
        vm.parse_word();
        assert_eq!(vm.last_token, "world");
        assert_eq!(vm.source_index, 12);
        vm.parse_word();
        assert_eq!(vm.last_token, "\"");
        assert_eq!(vm.error_code, 0);
    }

    #[test]
    fn test_evaluate () {
        let vm = &mut VM::new();
        vm.set_source("false true dup 1+ 2 -3");
        vm.evaluate();
        assert_eq!(vm.s_stack.len(), 5);
        assert_eq!(vm.s_stack.pop(), Some(-3));
        assert_eq!(vm.s_stack.pop(), Some(2));
        assert_eq!(vm.s_stack.pop(), Some(0));
        assert_eq!(vm.s_stack.pop(), Some(-1));
        assert_eq!(vm.s_stack.pop(), Some(0));
        assert_eq!(vm.error_code, 0);
    }

    #[bench]
    fn bench_compile_words_at_beginning_of_wordlist (b: &mut Bencher) {
        let vm = &mut VM::new();
        b.iter(|| {
            vm.set_source(": main noop noop noop noop noop noop noop noop ;");
            vm.evaluate();
            vm.s_stack.clear();
        });
    }

    #[bench]
    fn bench_compile_words_at_middle_of_wordlist(b: &mut Bencher) {
        let vm = &mut VM::new();
        b.iter(|| {
            vm.set_source(": main here here here here here here here here ;");
            vm.evaluate();
            vm.s_stack.clear();
        });
    }

    #[test]
    fn test_evaluate_f64 () {
        let vm = &mut VM::new();
        vm.set_source("1.0 2.5");
        vm.evaluate();
        assert_eq!(vm.f_stack.len(), 2);
        assert!(0.99999 < vm.f_stack[0]);
        assert!(vm.f_stack[0] < 1.00001);
        assert!(2.49999 < vm.f_stack[1]);
        assert!(vm.f_stack[1] < 2.50001);
        assert_eq!(vm.error_code, 0);
    }

    #[test]
    fn test_colon_and_semi_colon() {
        let vm = &mut VM::new();
        vm.set_source(": 2+3 2 3 + ; 2+3");
        vm.evaluate();
        assert_eq!(vm.s_stack.len(), 1);
        assert_eq!(vm.s_stack.pop(), Some(5));
        assert_eq!(vm.error_code, 0);
    }

    #[test]
    fn test_constant () {
        let vm = &mut VM::new();
        vm.set_source("5 constant x x x");
        vm.evaluate();
        assert_eq!(vm.s_stack.len(), 2);
        assert_eq!(vm.s_stack.pop(), Some(5));
        assert_eq!(vm.s_stack.pop(), Some(5));
        assert_eq!(vm.error_code, 0);
    }

    #[test]
    fn test_variable_and_store_fetch () {
        let vm = &mut VM::new();
        vm.set_source("variable x  x @  3 x !  x @");
        vm.evaluate();
        assert_eq!(vm.s_stack.len(), 2);
        assert_eq!(vm.s_stack.pop(), Some(3));
        assert_eq!(vm.s_stack.pop(), Some(0));
        assert_eq!(vm.error_code, 0);
    }

    #[test]
    fn test_execute () {
        let vm = &mut VM::new();
        vm.set_source("1 2  ' swap execute");
        vm.evaluate();
        assert_eq!(vm.s_stack.len(), 2);
        assert_eq!(vm.s_stack.pop(), Some(1));
        assert_eq!(vm.s_stack.pop(), Some(2));
        assert_eq!(vm.error_code, 0);
    }

    #[test]
    fn test_here_comma_comple_interpret () {
        let vm = &mut VM::new();
        vm.set_source("here 1 , 2 , ] noop noop [ here");
        vm.evaluate();
        assert_eq!(vm.s_stack.len(), 2);
        assert_eq!(vm.s_stack.pop(), Some(5));
        assert_eq!(vm.s_stack.pop(), Some(1));
        assert_eq!(vm.s_heap, [1, 1, 2, 1, 1]);
        assert_eq!(vm.error_code, 0);
    }

    #[test]
    fn test_to_r_r_fetch_r_from () {
        let vm = &mut VM::new();
        vm.set_source(": t 3 >r 2 r@ + r> + ; t");
        vm.evaluate();
        assert_eq!(vm.s_stack.len(), 1);
        assert_eq!(vm.s_stack.pop(), Some(8));
        assert_eq!(vm.error_code, 0);
    }

    #[bench]
    fn bench_to_r_r_fetch_r_from (b: &mut Bencher) {
        let vm = &mut VM::new();
        vm.set_source(": main 3 >r r@ drop r> drop ;");
        vm.evaluate();
        vm.set_source("' main");
        vm.evaluate();
        b.iter(|| {
            vm.dup();
            vm.execute();
            vm.inner();
        });
    }

    #[test]
    fn test_two_to_r_two_r_fetch_two_r_from () {
        let vm = &mut VM::new();
        vm.set_source(": t 1 2 2>r 2r@ + 2r> - * ; t");
        vm.evaluate();
        assert_eq!(vm.s_stack.len(), 1);
        assert_eq!(vm.s_stack.pop(), Some(-3));
        assert_eq!(vm.error_code, 0);
    }

    #[bench]
    fn bench_two_to_r_two_r_fetch_two_r_from (b: &mut Bencher) {
        let vm = &mut VM::new();
        vm.set_source(": main 1 2 2>r 2r@ 2drop 2r> 2drop ;");
        vm.evaluate();
        vm.set_source("' main");
        vm.evaluate();
        b.iter(|| {
            vm.dup();
            vm.execute();
            vm.inner();
        });
    }

    #[test]
    fn test_if_else_then () {
        let vm = &mut VM::new();
        vm.set_source(": t1 0 if true else false then ; t1");
        vm.evaluate();
        assert_eq!(vm.s_stack.len(), 1);
        assert_eq!(vm.s_stack.pop(), Some(0));
        vm.set_source(": t2 1 if true else false then ; t2");
        vm.evaluate();
        assert_eq!(vm.s_stack.len(), 1);
        assert_eq!(vm.s_stack.pop(), Some(-1));
        assert_eq!(vm.error_code, 0);
    }

    #[test]
    fn test_begin_again () {
        let vm = &mut VM::new();
        vm.set_source(": t1 0 begin 1+ dup 3 = if exit then again ; t1");
        vm.evaluate();
        assert_eq!(vm.s_stack.len(), 1);
        assert_eq!(vm.s_stack.pop(), Some(3));
        assert_eq!(vm.error_code, 0);
    }

    #[test]
    fn test_begin_while_repeat () {
        let vm = &mut VM::new();
        vm.set_source(": t1 0 begin 1+ dup 3 <> while repeat ; t1");
        vm.evaluate();
        assert_eq!(vm.s_stack.len(), 1);
        assert_eq!(vm.s_stack.pop(), Some(3));
        assert_eq!(vm.error_code, 0);
    }

    #[test]
    fn test_backlash () {
        let vm = &mut VM::new();
        vm.set_source("1 2 3 \\ 5 6 7");
        vm.evaluate();
        assert_eq!(vm.s_stack.len(), 3);
        assert_eq!(vm.s_stack.pop(), Some(3));
        assert_eq!(vm.s_stack.pop(), Some(2));
        assert_eq!(vm.s_stack.pop(), Some(1));
        assert_eq!(vm.error_code, 0);
    }

    #[test]
    fn test_quit () {
        let vm = &mut VM::new();
        vm.set_source("1 2 3 quit 5 6 7");
        vm.evaluate();
        assert_eq!(vm.s_stack.len(), 3);
        assert_eq!(vm.s_stack.pop(), Some(3));
        assert_eq!(vm.s_stack.pop(), Some(2));
        assert_eq!(vm.s_stack.pop(), Some(1));
        assert_eq!(vm.input_buffer.len(), 0);
        assert_eq!(vm.error_code, 0);
    }

    #[test]
    fn test_abort () {
        let vm = &mut VM::new();
        vm.set_source("1 2 3 abort 5 6 7");
        vm.evaluate();
        assert_eq!(vm.s_stack.len(), 0);
        assert_eq!(vm.error_code, -1);
    }

    #[test]
    fn test_fconstant () {
        let vm = &mut VM::new();
        vm.set_source("1.1 fconstant x x x");
        vm.evaluate();
        assert_eq!(vm.f_stack, [1.1, 1.1]);
        assert_eq!(vm.error_code, 0);
    }

    #[test]
    fn test_fvariable_and_fstore_ffetch () {
        let vm = &mut VM::new();
        vm.set_source("fvariable fx  fx f@  3.3 fx f!  fx f@");
        vm.evaluate();
        assert_eq!(vm.f_stack, [0.0, 3.3]);
        assert_eq!(vm.error_code, 0);
    }

    #[test]
    fn test_fabs () {
        let vm = &mut VM::new();
        vm.set_source("-3.14 fabs");
        vm.evaluate();
        assert_eq!(vm.f_stack.len(), 1);
        assert!(match vm.f_stack.pop() {
            Some(t) => {
                t > 3.13999 && t < 3.14001
            },
            None => false
        });
        assert_eq!(vm.error_code, 0);
    }

    #[test]
    fn test_fsin () {
        let vm = &mut VM::new();
        vm.set_source("3.14 fsin");
        vm.evaluate();
        assert_eq!(vm.f_stack.len(), 1);
        assert!(match vm.f_stack.pop() {
            Some(t) => {
                t > 0.0015925 && t < 0.0015927
            },
            None => false
        });
        assert_eq!(vm.error_code, 0);
    }

    #[test]
    fn test_fcos () {
        let vm = &mut VM::new();
        vm.set_source("3.0 fcos");
        vm.evaluate();
        assert_eq!(vm.f_stack.len(), 1);
        assert!(match vm.f_stack.pop() {
            Some(t) => {
                t > -0.989993 && t < -0.989991
            },
            None => false
        });
        assert_eq!(vm.error_code, 0);
    }

    #[test]
    fn test_ftan () {
        let vm = &mut VM::new();
        vm.set_source("3.0 ftan");
        vm.evaluate();
        assert_eq!(vm.f_stack.len(), 1);
        assert!(match vm.f_stack.pop() {
            Some(t) => {
                t > -0.142547 && t < -0.142545
            },
            None => false
        });
        assert_eq!(vm.error_code, 0);
    }

    #[test]
    fn test_fasin () {
        let vm = &mut VM::new();
        vm.set_source("0.3 fasin");
        vm.evaluate();
        assert_eq!(vm.f_stack.len(), 1);
        assert!(match vm.f_stack.pop() {
            Some(t) => {
                t > 0.304691 && t < 0.304693
            },
            None => false
        });
        assert_eq!(vm.error_code, 0);
    }

    #[test]
    fn test_facos () {
        let vm = &mut VM::new();
        vm.set_source("0.3 facos");
        vm.evaluate();
        assert_eq!(vm.f_stack.len(), 1);
        assert!(match vm.f_stack.pop() {
            Some(t) => {
                t > 1.266102 && t < 1.266104
            },
            None => false
        });
        assert_eq!(vm.error_code, 0);
    }

    #[test]
    fn test_fatan () {
        let vm = &mut VM::new();
        vm.set_source("0.3 fatan");
        vm.evaluate();
        assert_eq!(vm.f_stack.len(), 1);
        assert!(match vm.f_stack.pop() {
            Some(t) => {
                t > 0.291455 && t < 0.291457
            },
            None => false
        });
        assert_eq!(vm.error_code, 0);
    }

    #[test]
    fn test_fatan2 () {
        let vm = &mut VM::new();
        vm.set_source("3.0 4.0 fatan2");
        vm.evaluate();
        assert_eq!(vm.f_stack.len(), 1);
        assert!(match vm.f_stack.pop() {
            Some(t) => {
                t > 0.643500  && t < 0.643502
            },
            None => false
        });
        assert_eq!(vm.error_code, 0);
    }

    #[test]
    fn test_fsqrt () {
        let vm = &mut VM::new();
        vm.set_source("0.3 fsqrt");
        vm.evaluate();
        assert_eq!(vm.f_stack.len(), 1);
        assert!(match vm.f_stack.pop() {
            Some(t) => {
                t > 0.547721 && t < 0.547723 
            },
            None => false
        });
        assert_eq!(vm.error_code, 0);
    }

    #[test]
    fn test_fdrop() {
        let vm = &mut VM::new();
        vm.f_stack.push(1.0);
        vm.fdrop();
        assert_eq!(vm.f_stack, []);
        assert_eq!(vm.error_code, 0);
    }

    #[test]
    fn test_fnip() {
        let vm = &mut VM::new();
        vm.f_stack.push(1.0);
        vm.f_stack.push(2.0);
        vm.fnip();
        assert_eq!(vm.f_stack, [2.0]);
        assert_eq!(vm.error_code, 0);
    }

    #[test]
    fn test_fswap () {
        let vm = &mut VM::new();
        vm.f_stack.push(1.0);
        vm.f_stack.push(2.0);
        vm.fswap();
        assert_eq!(vm.f_stack, [2.0,1.0]);
        assert_eq!(vm.error_code, 0);
    }

    #[test]
    fn test_fdup () {
        let vm = &mut VM::new();
        vm.f_stack.push(1.0);
        vm.fdup();
        assert_eq!(vm.f_stack, [1.0, 1.0]);
        assert_eq!(vm.error_code, 0);
    }

    #[test]
    fn test_fover () {
        let vm = &mut VM::new();
        vm.f_stack.push(1.0);
        vm.f_stack.push(2.0);
        vm.fover();
        assert_eq!(vm.f_stack, [1.0,2.0,1.0]);
        assert_eq!(vm.error_code, 0);
    }

    #[test]
    fn test_frot () {
        let vm = &mut VM::new();
        vm.f_stack.push(1.0);
        vm.f_stack.push(2.0);
        vm.f_stack.push(3.0);
        vm.frot();
        assert_eq!(vm.f_stack, [2.0, 3.0, 1.0]);
        assert_eq!(vm.error_code, 0);
    }

    #[test]
    fn test_fplus_fminus_fstar_fslash () {
        let vm = &mut VM::new();
        vm.set_source("9.0 10.0 f+ 11.0 f- 12.0 f* 13.0 f/");
        vm.evaluate();
        assert_eq!(vm.f_stack.len(), 1);
        assert!(match vm.f_stack.pop() {
            Some(t) => {
                t > 7.384614 && t < 7.384616
            },
            None => false
        });
        assert_eq!(vm.error_code, 0);
    }

    #[test]
    fn test_f_zero_less_than () {
        let vm = &mut VM::new();
        vm.set_source("0.0 f0<   0.1 f0<   -0.1 f0<");
        vm.evaluate();
        assert_eq!(vm.s_stack.len(), 3);
        assert_eq!(vm.s_stack.pop(), Some(-1));
        assert_eq!(vm.s_stack.pop(), Some(0));
        assert_eq!(vm.s_stack.pop(), Some(0));
        assert_eq!(vm.f_stack, []);
        assert_eq!(vm.error_code, 0);
    }

    #[test]
    fn test_f_zero_equals () {
        let vm = &mut VM::new();
        vm.set_source("0.0 f0=   0.1 f0=   -0.1 f0=");
        vm.evaluate();
        assert_eq!(vm.s_stack.len(), 3);
        assert_eq!(vm.s_stack.pop(), Some(0));
        assert_eq!(vm.s_stack.pop(), Some(0));
        assert_eq!(vm.s_stack.pop(), Some(-1));
        assert_eq!(vm.f_stack, []);
        assert_eq!(vm.error_code, 0);
    }

    #[test]
    fn test_f_less_than () {
        let vm = &mut VM::new();
        vm.set_source("0.0 0.0 f<   0.1 0.0 f<   -0.1 0.0 f<");
        vm.evaluate();
        assert_eq!(vm.s_stack.len(), 3);
        assert_eq!(vm.s_stack.pop(), Some(-1));
        assert_eq!(vm.s_stack.pop(), Some(0));
        assert_eq!(vm.s_stack.pop(), Some(0));
        assert_eq!(vm.f_stack, []);
        assert_eq!(vm.error_code, 0);
    }

    #[test]
    fn test_fproximate () {
        let vm = &mut VM::new();
        vm.set_source("0.1 0.1 0.0 f~   0.1 0.10000000001 0.0 f~");
        vm.evaluate();
        assert_eq!(vm.s_stack.len(), 2);
        assert_eq!(vm.s_stack.pop(), Some(0));
        assert_eq!(vm.s_stack.pop(), Some(-1));
        assert_eq!(vm.f_stack, []);
        assert_eq!(vm.error_code, 0);
        vm.s_stack.clear();
        vm.set_source("0.1 0.1 0.001 f~   0.1 0.109 0.01 f~   0.1 0.111  0.01 f~");
        vm.evaluate();
        assert_eq!(vm.s_stack.len(), 3);
        assert_eq!(vm.s_stack.pop(), Some(0));
        assert_eq!(vm.s_stack.pop(), Some(-1));
        assert_eq!(vm.s_stack.pop(), Some(-1));
        assert_eq!(vm.f_stack, []);
        assert_eq!(vm.error_code, 0);
        vm.s_stack.clear();
        vm.set_source("0.1 0.1 -0.001 f~   0.1 0.109 -0.1 f~   0.1 0.109  -0.01 f~");
        vm.evaluate();
        assert_eq!(vm.s_stack.len(), 3);
        assert_eq!(vm.s_stack.pop(), Some(0));
        assert_eq!(vm.s_stack.pop(), Some(-1));
        assert_eq!(vm.s_stack.pop(), Some(-1));
        assert_eq!(vm.f_stack, []);
        assert_eq!(vm.error_code, 0);
        vm.s_stack.clear();
    }

    #[test]
    fn test_n_to_f () {
        let vm = &mut VM::new();
        vm.set_source("0 n>f -1 n>f 1 n>f");
        vm.evaluate();
        assert_eq!(vm.f_stack, [0.0, -1.0, 1.0]);
        assert_eq!(vm.error_code, 0);
    }

    #[bench]
    fn bench_fib(b: &mut Bencher) {
        let vm = &mut VM::new();
        vm.set_source(": fib dup 2 < if drop 1 else dup 1- recurse swap 2 - recurse + then ;");
        vm.evaluate();
        vm.set_source(": main 7 fib drop ;");
        vm.evaluate();
        vm.set_source("' main");
        vm.evaluate();
        b.iter(|| {
            vm.dup();
            vm.execute();
            vm.inner();
        });
    }
}
