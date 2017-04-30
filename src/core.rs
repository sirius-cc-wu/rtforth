extern crate libc;

extern "C" {
    fn memset(s: *mut libc::c_void, c: libc::uint32_t, n: libc::size_t) -> *mut libc::c_void;
}

use {TRUE, FALSE};
use std::process;
use std::mem;
use std::ptr::{self, Unique};
use std::fmt;
use std::fmt::Write;
use std::str::FromStr;
use std::slice;
use std::ascii::AsciiExt;
use std::result;
use jitmem::{self, DataSpace};
use exception::Exception::{self, Abort, UnexpectedEndOfFile, UndefinedWord, StackOverflow,
                           StackUnderflow, ReturnStackUnderflow, ReturnStackOverflow,
                           FloatingPointStackOverflow, UnsupportedOperation,
                           InterpretingACompileOnlyWord, ControlStructureMismatch};

pub type Result = result::Result<(), Exception>;

// Word
pub struct Word<Target> {
    symbol: Symbol,
    is_immediate: bool,
    is_compile_only: bool,
    hidden: bool,
    dfa: usize,
    action: fn(&mut Target),
}

impl<Target> Word<Target> {
    pub fn new(symbol: Symbol, action: fn(&mut Target), dfa: usize) -> Word<Target> {
        Word {
            symbol: symbol,
            is_immediate: false,
            is_compile_only: false,
            hidden: false,
            dfa: dfa,
            action: action,
        }
    }

    pub fn symbol(&self) -> Symbol {
        self.symbol
    }

    pub fn is_immediate(&self) -> bool {
        self.is_immediate
    }

    pub fn set_immediate(&mut self, flag: bool) {
        self.is_immediate = flag;
    }

    pub fn is_compile_only(&self) -> bool {
        self.is_compile_only
    }

    pub fn set_compile_only(&mut self, flag: bool) {
        self.is_compile_only = flag;
    }

    pub fn is_hidden(&self) -> bool {
        self.hidden
    }

    pub fn set_hidden(&mut self, flag: bool) {
        self.hidden = flag;
    }

    pub fn dfa(&self) -> usize {
        self.dfa
    }

    pub fn action(&self) -> fn(&mut Target) {
        self.action
    }
}

pub struct Stack<T> {
    pub inner: Unique<T>,
    pub cap: usize,
    pub len: usize,
}

impl<T> Stack<T> {
    pub fn with_capacity(cap: usize) -> Self {
        assert!(cap > 0 && cap <= 2048, "Invalid stack capacity");
        let align = mem::align_of::<isize>();
        let elem_size = mem::size_of::<isize>();
        let size_in_bytes = cap * elem_size;
        unsafe {
            let mut ptr = mem::uninitialized();
            libc::posix_memalign(&mut ptr, align, size_in_bytes);
            if ptr.is_null() {
                panic!("Cannot allocate memory.");
            }
            libc::mprotect(ptr, size_in_bytes, libc::PROT_READ | libc::PROT_WRITE);
            memset(ptr, 0x00, size_in_bytes);
            Stack {
                inner: Unique::new(ptr as *mut _),
                cap: cap,
                len: 0,
            }
        }
    }

    pub fn push(&mut self, v: T) -> Result {
        if self.len >= self.cap {
            Err(StackOverflow)
        } else {
            unsafe {
                ptr::write(self.inner.offset(self.len as isize), v);
            }
            self.len += 1;
            Ok(())
        }
    }

    pub fn pop(&mut self) -> result::Result<T, Exception> {
        if self.len < 1 {
            Err(StackUnderflow)
        } else {
            self.len -= 1;
            unsafe { Ok(ptr::read(self.inner.offset(self.len as isize))) }
        }
    }

    pub fn push2(&mut self, v1: T, v2: T) -> Result {
        if self.len + 2 > self.cap {
            Err(StackOverflow)
        } else {
            unsafe {
                ptr::write(self.inner.offset(self.len as isize), v1);
                ptr::write(self.inner.offset((self.len + 1) as isize), v2);
            }
            self.len += 2;
            Ok(())
        }
    }

    pub fn push3(&mut self, v1: T, v2: T, v3: T) -> Result {
        if self.len + 3 > self.cap {
            Err(StackOverflow)
        } else {
            unsafe {
                ptr::write(self.inner.offset(self.len as isize), v1);
                ptr::write(self.inner.offset((self.len + 1) as isize), v2);
                ptr::write(self.inner.offset((self.len + 2) as isize), v3);
            }
            self.len += 3;
            Ok(())
        }
    }

    pub fn pop2(&mut self) -> result::Result<(T, T), Exception> {
        if self.len < 2 {
            Err(StackUnderflow)
        } else {
            self.len -= 2;
            unsafe {
                Ok((ptr::read(self.inner.offset(self.len as isize)),
                    ptr::read(self.inner.offset((self.len + 1) as isize))))
            }
        }
    }

    pub fn pop3(&mut self) -> result::Result<(T, T, T), Exception> {
        if self.len < 3 {
            Err(StackUnderflow)
        } else {
            self.len -= 3;
            unsafe {
                Ok((ptr::read(self.inner.offset(self.len as isize)),
                    ptr::read(self.inner.offset((self.len + 1) as isize)),
                    ptr::read(self.inner.offset((self.len + 2) as isize))))
            }
        }
    }

    pub fn last(&self) -> Option<T> {
        if self.len == 0 {
            None
        } else {
            unsafe { Some(ptr::read(self.inner.offset((self.len - 1) as isize))) }
        }
    }

    pub fn get(&self, pos: usize) -> Option<T> {
        if pos >= self.len {
            None
        } else {
            unsafe { Some(ptr::read(self.inner.offset(pos as isize))) }
        }
    }

    pub fn clear(&mut self) {
        self.len = 0;
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn is_full(&self) -> bool {
        self.len >= self.cap
    }

    pub fn space_left(&self) -> usize {
        self.cap - self.len
    }

    /// # Safety
    /// Because the implementer (me) is still learning Rust, it is uncertain if as_slice is safe.
    pub fn as_slice(&self) -> &[T] {
        unsafe { slice::from_raw_parts(self.inner.get(), self.len) }
    }
}

impl<T: fmt::Display> fmt::Debug for Stack<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match write!(f, "<{}> ", self.len()) {
            Ok(_) => {
                for i in 0..(self.len() - 1) {
                    let v = unsafe { ptr::read(self.inner.offset(i as isize)) };
                    match write!(f, "{} ", v) {
                        Ok(_) => {}
                        Err(e) => {
                            return Err(e);
                        }
                    }
                }
                Ok(())
            }
            Err(e) => Err(e),
        }
    }
}

pub struct ForwardReferences {
    pub idx_lit: usize,
    pub idx_flit: usize,
    pub idx_exit: usize,
    pub idx_zero_branch: usize,
    pub idx_branch: usize,
    pub idx_do: usize,
    pub idx_loop: usize,
    pub idx_plus_loop: usize,
    pub idx_s_quote: usize,
    pub idx_type: usize,
}

impl ForwardReferences {
    pub fn new() -> ForwardReferences {
        ForwardReferences {
            idx_lit: 0,
            idx_flit: 0,
            idx_exit: 0,
            idx_zero_branch: 0,
            idx_branch: 0,
            idx_do: 0,
            idx_loop: 0,
            idx_plus_loop: 0,
            idx_s_quote: 0,
            idx_type: 0,
        }
    }
}

pub struct State {
    pub is_compiling: bool,
    pub instruction_pointer: usize,
    word_pointer: usize,
    pub source_index: usize,
}

impl State {
    pub fn new() -> State {
        State {
            is_compiling: false,
            instruction_pointer: 0,
            word_pointer: 0,
            source_index: 0,
        }
    }

    /// Idle is the result of new and reset, means that VM has nothing to do.
    fn is_idle(&self) -> bool {
        self.instruction_pointer == 0
    }
    pub fn word_pointer(&self) -> usize {
        self.word_pointer
    }
}

#[derive(PartialEq, Clone, Copy)]
pub struct Symbol {
    id: usize,
}

impl Symbol {
    pub fn id(&self) -> usize {
        self.id
    }
}

pub trait Core: Sized {
    // Functions to access VM.
    fn last_error(&self) -> Option<Exception>;
    fn set_error(&mut self, e: Option<Exception>);
    fn handler(&self) -> usize;
    fn set_handler(&mut self, h: usize);
    fn data_space(&mut self) -> &mut DataSpace;
    fn data_space_const(&self) -> &DataSpace;
    /// Get `output_buffer`.
    fn output_buffer(&mut self) -> &mut Option<String>;
    /// Set `output_buffer` to `Some(buffer)`.
    fn set_output_buffer(&mut self, buffer: String);
    /// Get `input_buffer`.
    fn input_buffer(&mut self) -> &mut Option<String>;
    /// Set `input_buffer` to `Some(buffer)`.
    fn set_input_buffer(&mut self, buffer: String);
    fn last_token(&mut self) -> &mut Option<String>;
    fn set_last_token(&mut self, buffer: String);
    fn structure_depth(&self) -> usize;
    fn set_structure_depth(&mut self, d: usize);
    fn s_stack(&mut self) -> &mut Stack<isize>;
    fn r_stack(&mut self) -> &mut Stack<isize>;
    fn f_stack(&mut self) -> &mut Stack<f64>;
    fn symbols_mut(&mut self) -> &mut Vec<String>;
    fn symbols(&self) -> &Vec<String>;
    /// Last definition, 0 if last define fails.
    fn last_definition(&self) -> usize;
    fn set_last_definition(&mut self, n: usize);
    fn wordlist_mut(&mut self) -> &mut Vec<Word<Self>>;
    fn wordlist(&self) -> &Vec<Word<Self>>;
    fn state(&mut self) -> &mut State;
    fn references(&mut self) -> &mut ForwardReferences;

    /// Add core primitives to self.
    fn add_core(&mut self) {
        // Bytecodes
        self.add_primitive("noop", Core::noop); // j1, Ngaro, jx
        self.add_compile_only("exit", Core::exit); // j1, jx, eForth
        self.add_compile_only("halt", Core::halt); // rtForth
        self.add_compile_only("lit", Core::lit); // Ngaro, jx, eForth
        self.add_compile_only("flit", Core::flit);
        self.add_compile_only("_s\"", Core::p_s_quote);
        self.add_compile_only("branch", Core::branch); // j1, eForth
        self.add_compile_only("0branch", Core::zero_branch); // j1, eForth
        self.add_compile_only("_do", Core::_do); // jx
        self.add_compile_only("_loop", Core::p_loop); // jx
        self.add_compile_only("_+loop", Core::p_plus_loop); // jx
        self.add_compile_only("unloop", Core::unloop); // jx
        self.add_compile_only("leave", Core::leave); // jx
        self.add_compile_only("i", Core::p_i); // jx
        self.add_compile_only("j", Core::p_j); // jx
        self.add_compile_only(">r", Core::p_to_r); // j1, Ngaro, jx, eForth
        self.add_compile_only("r>", Core::r_from); // j1, Ngaro, jx, eForth
        self.add_compile_only("r@", Core::r_fetch); // j1, jx, eForth
        self.add_compile_only("2>r", Core::two_to_r); // jx
        self.add_compile_only("2r>", Core::two_r_from); // jx
        self.add_compile_only("2r@", Core::two_r_fetch); // jx

        self.add_primitive("execute", Core::execute); // jx, eForth
        self.add_primitive("dup", Core::dup); // j1, Ngaro, jx, eForth
        self.add_primitive("drop", Core::p_drop); // j1, Ngaro, jx, eForth
        self.add_primitive("swap", Core::swap); // j1, Ngaro, jx, eForth
        self.add_primitive("over", Core::over); // j1, jx, eForth
        self.add_primitive("nip", Core::nip); // j1, jx
        self.add_primitive("depth", Core::depth); // j1, jx
        self.add_primitive("0<", Core::zero_less); // eForth
        self.add_primitive("=", Core::equals); // j1, jx
        self.add_primitive("<", Core::less_than); // j1, jx
        self.add_primitive("invert", Core::invert); // j1, jx
        self.add_primitive("and", Core::and); // j1, Ngaro, jx, eForth
        self.add_primitive("or", Core::or); // j1, Ngaro, jx, eForth
        self.add_primitive("xor", Core::xor); // j1, Ngaro, jx, eForth
        self.add_primitive("lshift", Core::lshift); // jx, Ngaro
        self.add_primitive("rshift", Core::rshift); // jx
        self.add_primitive("arshift", Core::arshift); // jx, Ngaro
        self.add_primitive("1+", Core::one_plus); // Ngaro
        self.add_primitive("1-", Core::one_minus); // Ngaro, jx
        self.add_primitive("-", Core::minus); // Ngaro
        self.add_primitive("+", Core::plus); // j1, Ngaro, jx
        self.add_primitive("*", Core::star); // Ngaro
        self.add_primitive("/mod", Core::slash_mod); // Ngaro
        self.add_primitive("cell+", Core::cell_plus); // eForth
        self.add_primitive("cells", Core::cells); // eForth
        self.add_primitive("@", Core::fetch); // j1, jx, eForth
        self.add_primitive("!", Core::store); // j1, jx, eForth
        self.add_primitive("char+", Core::char_plus); // eForth
        self.add_primitive("chars", Core::chars); // eForth
        self.add_primitive("here", Core::here);
        self.add_primitive("allot", Core::allot);
        self.add_primitive("c@", Core::c_fetch);
        self.add_primitive("c!", Core::c_store);
        self.add_primitive("base", Core::base);

        // Immediate words
        self.add_immediate("(", Core::imm_paren);
        self.add_immediate("\\", Core::imm_backslash);
        self.add_immediate("[", Core::left_bracket);
        self.add_immediate_and_compile_only("[char]", Core::bracket_char);
        self.add_immediate_and_compile_only(";", Core::semicolon);
        self.add_immediate_and_compile_only("if", Core::imm_if);
        self.add_immediate_and_compile_only("else", Core::imm_else);
        self.add_immediate_and_compile_only("then", Core::imm_then);
        self.add_immediate_and_compile_only("begin", Core::imm_begin);
        self.add_immediate_and_compile_only("while", Core::imm_while);
        self.add_immediate_and_compile_only("repeat", Core::imm_repeat);
        self.add_immediate_and_compile_only("again", Core::imm_again);
        self.add_immediate_and_compile_only("recurse", Core::imm_recurse);
        self.add_immediate_and_compile_only("do", Core::imm_do);
        self.add_immediate_and_compile_only("loop", Core::imm_loop);
        self.add_immediate_and_compile_only("+loop", Core::imm_plus_loop);

        // More Primitives
        self.add_primitive("true", Core::p_true);
        self.add_primitive("false", Core::p_false);
        self.add_primitive("not", Core::zero_equals);
        self.add_primitive("0=", Core::zero_equals);
        self.add_primitive("0>", Core::zero_greater);
        self.add_primitive("0<>", Core::zero_not_equals);
        self.add_primitive(">", Core::greater_than);
        self.add_primitive("<>", Core::not_equals);
        self.add_primitive("rot", Core::rot);
        self.add_primitive("2dup", Core::two_dup);
        self.add_primitive("2drop", Core::two_drop);
        self.add_primitive("2swap", Core::two_swap);
        self.add_primitive("2over", Core::two_over);
        self.add_primitive("/", Core::slash);
        self.add_primitive("mod", Core::p_mod);
        self.add_primitive("abs", Core::abs);
        self.add_primitive("negate", Core::negate);
        self.add_primitive("between", Core::between);
        self.add_primitive("parse-word", Core::parse_word);
        self.add_primitive("char", Core::char);
        self.add_primitive("parse", Core::parse);
        self.add_primitive(":", Core::colon);
        self.add_primitive("constant", Core::constant);
        self.add_primitive("variable", Core::variable);
        self.add_primitive("create", Core::create);
        self.add_primitive("'", Core::tick);
        self.add_primitive("]", Core::right_bracket);
        self.add_primitive(",", Core::comma);
        self.add_primitive("marker", Core::marker);
        self.add_primitive("handler!", Core::handler_store);
        self.add_primitive("error?", Core::p_error_q);
        self.add_primitive("handle-error", Core::p_handle_error);
        self.add_primitive("reset", Core::reset);
        self.add_primitive("abort", Core::abort);
        self.add_primitive("bye", Core::bye);
        self.add_primitive("jit", Core::jit);
        self.add_primitive("compiling?", Core::p_compiling);
        self.add_primitive("token-empty?", Core::p_token_empty);
        self.add_primitive("compile-token", Core::compile_token);
        self.add_primitive("interpret-token", Core::interpret_token);

        self.references().idx_lit = self.find("lit").expect("lit undefined");
        self.references().idx_flit = self.find("flit").expect("flit undefined");
        self.references().idx_exit = self.find("exit").expect("exit undefined");
        self.references().idx_zero_branch = self.find("0branch").expect("0branch undefined");
        self.references().idx_branch = self.find("branch").expect("branch undefined");
        self.references().idx_do = self.find("_do").expect("_do undefined");
        self.references().idx_loop = self.find("_loop").expect("_loop undefined");
        self.references().idx_plus_loop = self.find("_+loop").expect("_+loop undefined");
        let idx_halt = self.find("halt").expect("halt undefined");
        self.data_space().put_u32(idx_halt as u32, 0);
    }

    fn push(&mut self, value: isize) {
        match self.s_stack().push(value) {
            Err(_) => self.abort_with(StackOverflow),
            Ok(()) => {}
        }
    }

    /// Add a primitive word to word list.
    fn add_primitive(&mut self, name: &str, action: fn(&mut Self)) {
        let symbol = self.new_symbol(name);
        let word = Word::new(symbol, action, self.data_space().len());
        let len = self.wordlist().len();
        self.set_last_definition(len);
        self.wordlist_mut().push(word);
    }

    /// Add an immediate word to word list.
    fn add_immediate(&mut self, name: &str, action: fn(&mut Self)) {
        self.add_primitive(name, action);
        let def = self.last_definition();
        self.wordlist_mut()[def].set_immediate(true);
    }

    /// Add a compile-only word to word list.
    fn add_compile_only(&mut self, name: &str, action: fn(&mut Self)) {
        self.add_primitive(name, action);
        let def = self.last_definition();
        self.wordlist_mut()[def].set_compile_only(true);
    }

    /// Add an immediate and compile-only word to word list.
    fn add_immediate_and_compile_only(&mut self, name: &str, action: fn(&mut Self)) {
        self.add_primitive(name, action);
        let def = self.last_definition();
        let w = &mut self.wordlist_mut()[def];
        w.set_immediate(true);
        w.set_compile_only(true);
    }

    /// Execute word at position `i`.
    fn execute_word(&mut self, i: usize) {
        self.state().word_pointer = i;
        (self.wordlist()[i].action())(self);
    }

    /// Find the word with name `name`.
    /// If not found returns zero.
    fn find(&mut self, name: &str) -> Option<usize> {
        for (i, word) in self.wordlist().iter().enumerate().rev() {
            if !word.is_hidden() && self.symbols()[word.symbol().id].eq_ignore_ascii_case(name) {
                return Some(i);
            }
        }
        None
    }

    fn find_symbol(&mut self, s: &str) -> Option<Symbol> {
        for (i, sym) in self.symbols().iter().enumerate().rev() {
            if sym.eq_ignore_ascii_case(s) {
                return Some(Symbol { id: i });
            }
        }
        None
    }

    fn new_symbol(&mut self, s: &str) -> Symbol {
        self.symbols_mut().push(s.to_string());
        Symbol { id: self.symbols().len() - 1 }
    }

    // ------------------
    // Inner interpreter
    // ------------------

    /// Evaluate a compiled program following self.state().instruction_pointer.
    /// Any exception causes termination of inner loop.
    #[inline(never)]
    fn run(&mut self) {
        let mut ip = self.state().instruction_pointer;
        while 0 < ip && ip < self.data_space().len() {
            let w = self.data_space().get_i32(ip) as usize;
            self.state().instruction_pointer += mem::size_of::<i32>();
            self.execute_word(w);
            ip = self.state().instruction_pointer;
        }
    }

    // ---------
    // Compiler
    // ---------

    fn compile_word(&mut self, word_index: usize) {
        self.data_space().compile_i32(word_index as i32);
    }

    /// Compile integer `i`.
    fn compile_integer(&mut self, i: isize) {
        let idx = self.references().idx_lit as i32;
        self.data_space().compile_i32(idx);
        self.data_space().compile_i32(i as i32);
    }

    /// Compile float 'f'.
    fn compile_float(&mut self, f: f64) {
        let idx_flit = self.references().idx_flit;
        self.data_space().compile_i32(idx_flit as i32);
        self.data_space().compile_f64(f);
    }

    // -----------
    // Evaluation
    // -----------

    fn left_bracket(&mut self) {
        self.state().is_compiling = false;
    }

    fn right_bracket(&mut self) {
        self.state().is_compiling = true;
    }

    /// copy content of `s` to `input_buffer` and set `source_index` to 0.
    fn set_source(&mut self, s: &str) {
        let mut buffer = self.input_buffer().take().unwrap();
        buffer.clear();
        buffer.push_str(s);
        self.state().source_index = 0;
        self.set_input_buffer(buffer);
    }

    /// Run-time: ( "ccc" -- )
    ///
    /// Parse word delimited by white space, skipping leading white spaces.
    fn parse_word(&mut self) {
        let mut last_token = self.last_token().take().unwrap();
        last_token.clear();
        let input_buffer = self.input_buffer().take().unwrap();
        if self.state().source_index < input_buffer.len() {
            let source = &input_buffer[self.state().source_index..input_buffer.len()];
            let mut cnt = 0;
            for ch in source.chars() {
                cnt = cnt + 1;
                match ch {
                    '\t' | '\n' | '\r' | ' ' => {
                        if !last_token.is_empty() {
                            break;
                        }
                    }
                    _ => last_token.push(ch),
                };
            }
            self.state().source_index = self.state().source_index + cnt;
        }
        self.set_last_token(last_token);
        self.set_input_buffer(input_buffer);
    }

    /// Run-time: ( "&lt;spaces&gt;name" -- char)
    ///
    /// Skip leading space delimiters. Parse name delimited by a space.
    /// Put the value of its first character onto the stack.
    fn char(&mut self) {
        self.parse_word();
        if self.last_error().is_some() {
            return;
        }
        let last_token = self.last_token().take().unwrap();
        match last_token.chars().nth(0) {
            Some(c) => self.push(c as isize),
            None => self.abort_with(UnexpectedEndOfFile),
        }
        self.set_last_token(last_token);
    }

    /// Compilation: ( "&lt;spaces&gt;name" -- )
    ///
    /// Skip leading space delimiters. Parse name delimited by a space.
    /// Append the run-time semantics given below to the current definition.
    ///
    /// Run-time: ( -- char )
    ///
    /// Place `char`, the value of the first character of name, on the stack.
    fn bracket_char(&mut self) {
        self.char();
        if self.last_error().is_some() {
            return;
        }
        match self.s_stack().pop() {
            Ok(ch) => {
                self.compile_integer(ch);
            }
            Err(_) => self.abort_with(StackUnderflow),
        }
    }

    /// Run-time: ( char "ccc&lt;char&gt;" -- )
    ///
    /// Parse ccc delimited by the delimiter char.
    fn parse(&mut self) {
        let input_buffer = self.input_buffer().take().unwrap();
        match self.s_stack().pop() {
            Ok(v) => {
                let mut last_token = self.last_token().take().unwrap();
                last_token.clear();
                {
                    let source = &input_buffer[self.state().source_index..input_buffer.len()];
                    let mut cnt = 0;
                    for ch in source.chars() {
                        cnt = cnt + 1;
                        if ch as isize == v {
                            break;
                        } else {
                            last_token.push(ch);
                        }
                    }
                    self.state().source_index = self.state().source_index + cnt;
                }
                self.set_last_token(last_token);
                self.set_input_buffer(input_buffer);
            }
            Err(_) => {
                self.set_input_buffer(input_buffer);
                self.abort_with(StackUnderflow);
            }
        }
    }

    fn imm_paren(&mut self) {
        match self.s_stack().push(')' as isize) {
            Err(_) => self.abort_with(StackOverflow),
            Ok(()) => { self.parse(); }
        }
    }

    fn imm_backslash(&mut self) {
        self.state().source_index = match *self.input_buffer() {
            Some(ref buf) => buf.len(),
            None => 0,
        };
    }

    fn compile_token(&mut self) {
        let last_token = self.last_token().take().unwrap();
        match self.find(&last_token) {
            Some(found_index) => {
                self.set_last_token(last_token);
                if !self.wordlist()[found_index].is_immediate() {
                    self.compile_word(found_index);
                } else {
                    self.execute_word(found_index);
                }
            }
            None => {
                let mut done = false;
                self.set_error(None);
                self.evaluate_integer(&last_token);
                match self.last_error() {
                    None => done = true,
                    Some(_) => {
                        self.set_error(None);
                        self.evaluate_float(&last_token);
                        if self.last_error().is_none() {
                            done = true;
                        }
                    }
                }
                if !done {
                    match self.output_buffer().as_mut() {
                        Some(buf) => {
                            write!(buf, "{} ", &last_token).unwrap();
                        }
                        None => {}
                    }
                    self.abort_with(UndefinedWord);
                }
                self.set_last_token(last_token);
            }
        }
    }

    fn interpret_token(&mut self) {
        let last_token = self.last_token().take().unwrap();
        match self.find(&last_token) {
            Some(found_index) => {
                self.set_last_token(last_token);
                if self.wordlist()[found_index].is_compile_only() {
                    self.abort_with(InterpretingACompileOnlyWord);
                } else {
                    self.execute_word(found_index);
                }
            }
            None => {
                let mut done = false;
                self.set_error(None);
                self.evaluate_integer(&last_token);
                match self.last_error() {
                    None => done = true,
                    Some(_) => {
                        self.set_error(None);
                        self.evaluate_float(&last_token);
                        if self.last_error().is_none() {
                            done = true;
                        }
                    }
                }
                if !done {
                    match self.output_buffer().as_mut() {
                        Some(buf) => {
                            write!(buf, "{} ", &last_token).unwrap();
                        }
                        None => {}
                    }
                    self.abort_with(UndefinedWord);
                }
                self.set_last_token(last_token);
            }
        }
    }

    fn p_compiling(&mut self) {
        let value = if self.state().is_compiling {
            TRUE
        } else {
            FALSE
        };
        self.push(value);
    }

    fn p_token_empty(&mut self) {
        let value = match self.last_token().as_ref() {
            Some(ref t) => if t.is_empty() { TRUE } else { FALSE },
            None => TRUE,
        };
        self.push(value);
    }

    fn evaluate(&mut self) {
        loop {
            self.parse_word();
            match self.last_token().as_ref() {
                Some(t) => {
                    if t.is_empty() {
                        return;
                    }
                }
                None => {}
            }
            if self.state().is_compiling {
                self.compile_token();
            } else {
                self.interpret_token();
            }
            self.run();
            if self.last_error().is_some() {
                break;
            }
        }
    }

    fn base(&mut self) {
        let base_addr = self.data_space().system_variables().base_addr();
        self.push(base_addr as isize);
    }

    fn evaluate_integer(&mut self, token: &str) {
        let base_addr = self.data_space().system_variables().base_addr();
        let base = self.data_space().get_isize(base_addr);
        match isize::from_str_radix(token, base as u32) {
            Ok(t) => {
                if self.state().is_compiling {
                    self.compile_integer(t);
                } else {
                    self.push(t);
                }
            }
            Err(_) => self.set_error(Some(UnsupportedOperation)),
        }
    }

    /// Evaluate float.
    fn evaluate_float(&mut self, token: &str) {
        match FromStr::from_str(token) {
            Ok(t) => {
                if self.references().idx_flit == 0 {
                    self.set_error(Some(UnsupportedOperation));
                } else {
                    if self.state().is_compiling {
                        self.compile_float(t);
                    } else {
                        if let Err(_) = self.f_stack().push(t) {
                            self.set_error(Some(FloatingPointStackOverflow));
                        }
                    }
                }
            }
            Err(_) => self.set_error(Some(UnsupportedOperation)),
        }
    }

    // -----------------------
    // High level definitions
    // -----------------------

    fn nest(&mut self) {
        if self.r_stack().is_full() {
            self.abort_with(ReturnStackOverflow);
        } else {
            unsafe {
                ptr::write(self.r_stack().inner.offset(self.r_stack().len as isize),
                           self.state().instruction_pointer as isize);
            }
            self.r_stack().len += 1;
            let wp = self.state().word_pointer;
            self.state().instruction_pointer = self.wordlist()[wp].dfa();
        }
    }

    fn p_var(&mut self) {
        let wp = self.state().word_pointer;
        let dfa = self.wordlist()[wp].dfa() as isize;
        self.push(dfa);
    }

    fn p_const(&mut self) {
        let wp = self.state().word_pointer;
        let dfa = self.wordlist()[wp].dfa();
        let value = self.data_space().get_i32(dfa) as isize;
        self.push(value);
    }

    fn p_fvar(&mut self) {
        let wp = self.state().word_pointer;
        let dfa = self.wordlist()[wp].dfa() as isize;
        self.push(dfa);
    }

    fn define(&mut self, action: fn(&mut Self)) {
        self.parse_word();
        let mut last_token = self.last_token().take().unwrap();
        last_token.make_ascii_lowercase();
        if let Some(_) = self.find(&last_token) {
            match self.output_buffer().as_mut() {
                Some(buf) => {
                    write!(buf, "Redefining {}", last_token).unwrap();
                }
                None => {}
            }
        }
        if last_token.is_empty() {
            self.set_last_definition(0);
            self.set_last_token(last_token);
            self.abort_with(UnexpectedEndOfFile);
        } else {
            let symbol = self.new_symbol(&last_token);
            let word = Word::new(symbol, action, self.data_space().len());
            let len = self.wordlist().len();
            self.set_last_definition(len);
            self.wordlist_mut().push(word);
            self.set_last_token(last_token);
        }
    }

    fn colon(&mut self) {
        self.define(Core::nest);
        if self.last_error().is_none() {
            let def = self.last_definition();
            self.wordlist_mut()[def].set_hidden(true);
            let depth = self.s_stack().len;
            self.set_structure_depth(depth);
            self.right_bracket();
        }
    }

    fn semicolon(&mut self) {
        if self.last_definition() != 0 {
            let depth = self.s_stack().len;
            if depth != self.structure_depth() {
                self.abort_with(ControlStructureMismatch);
            } else {
                let idx = self.references().idx_exit as i32;
                self.data_space().compile_i32(idx);
                let def = self.last_definition();
                self.wordlist_mut()[def].set_hidden(false);
            }
        }
        self.left_bracket();
    }

    fn create(&mut self) {
        self.define(Core::p_var);
    }

    fn variable(&mut self) {
        self.define(Core::p_var);
        if self.last_error().is_none() {
            self.data_space().compile_i32(0);
        }
    }

    fn constant(&mut self) {
        match self.s_stack().pop() {
            Ok(v) => {
                self.define(Core::p_const);
                if self.last_error().is_none() {
                    self.data_space().compile_i32(v as i32);
                }
            }
            Err(_) => self.abort_with(StackUnderflow),
        }
    }

    fn unmark(&mut self) {
        let wp = self.state().word_pointer;
        let dfa;
        let symbol;
        {
            let w = &self.wordlist()[wp];
            dfa = w.dfa();
            symbol = w.symbol();
        }
        self.data_space().truncate(dfa);
        self.wordlist_mut().truncate(wp);
        self.symbols_mut().truncate(symbol.id);
    }

    fn marker(&mut self) {
        self.define(Core::unmark);
    }

    // --------
    // Control
    // --------)

    fn branch(&mut self) {
        let ip = self.state().instruction_pointer;
        self.state().instruction_pointer = self.data_space().get_i32(ip) as usize;
    }

    fn zero_branch(&mut self) {
        match self.s_stack().pop() {
            Ok(v) => {
                if v == 0 {
                    self.branch();
                } else {
                    self.state().instruction_pointer += mem::size_of::<i32>();
                }
            }
            Err(_) => self.abort_with(StackUnderflow),
        }
    }

    /// ( n1|u1 n2|u2 -- ) ( R: -- loop-sys )
    ///
    /// Set up loop control parameters with index `n2`|`u2` and limit `n1`|`u1`. An
    /// ambiguous condition exists if `n1`|`u1` and `n2`|`u2` are not both the same
    /// type.  Anything already on the return stack becomes unavailable until
    /// the loop-control parameters are discarded.
    fn _do(&mut self) {
        let ip = self.state().instruction_pointer as isize;
        match self.r_stack().push(ip) {
            Err(_) => self.abort_with(ReturnStackOverflow),
            Ok(()) => {
                self.state().instruction_pointer += mem::size_of::<i32>();
                self.two_to_r();
            }
        }
    }

    /// Run-time: ( -- ) ( R:  loop-sys1 --  | loop-sys2 )
    ///
    /// An ambiguous condition exists if the loop control parameters are
    /// unavailable. Add one to the loop index. If the loop index is then equal
    /// to the loop limit, discard the loop parameters and continue execution
    /// immediately following the loop. Otherwise continue execution at the
    /// beginning of the loop.
    fn p_loop(&mut self) {
        match self.r_stack().pop2() {
            Ok((rn, rt)) => {
                if rt + 1 < rn {
                    if let Err(e) = self.r_stack().push2(rn, rt + 1) {
                        self.abort_with(e);
                        return;
                    }
                    self.branch();
                } else {
                    match self.r_stack().pop() {
                        Ok(_) => {
                            self.state().instruction_pointer += mem::size_of::<i32>();
                        }
                        Err(_) => self.abort_with(ReturnStackUnderflow),
                    }
                }
            }
            Err(_) => self.abort_with(ReturnStackUnderflow),
        }
    }

    /// Run-time: ( n -- ) ( R: loop-sys1 -- | loop-sys2 )
    ///
    /// An ambiguous condition exists if the loop control parameters are
    /// unavailable. Add `n` to the loop index. If the loop index did not cross
    /// the boundary between the loop limit minus one and the loop limit,
    /// continue execution at the beginning of the loop. Otherwise, discard the
    /// current loop control parameters and continue execution immediately
    /// following the loop.
    fn p_plus_loop(&mut self) {
        match self.r_stack().pop2() {
            Ok((rn, rt)) => {
                match self.s_stack().pop() {
                    Ok(t) => {
                        if rt + t < rn {
                            if let Err(e) = self.r_stack().push2(rn, rt + t) {
                                self.abort_with(e);
                                return;
                            }
                            self.branch();
                        } else {
                            match self.r_stack().pop() {
                                Ok(_) => {
                                    self.state().instruction_pointer += mem::size_of::<i32>();
                                }
                                Err(_) => self.abort_with(ReturnStackUnderflow),
                            }
                        }
                    }
                    Err(_) => self.abort_with(StackUnderflow),
                }
            }
            Err(_) => self.abort_with(ReturnStackUnderflow),
        }
    }

    /// Run-time: ( -- ) ( R: loop-sys -- )
    ///
    /// Discard the loop-control parameters for the current nesting level. An
    /// `UNLOOP` is required for each nesting level before the definition may be
    /// `EXIT`ed. An ambiguous condition exists if the loop-control parameters
    /// are unavailable.
    fn unloop(&mut self) {
        match self.r_stack().pop3() {
            Ok(_) => {}
            Err(_) => self.abort_with(ReturnStackUnderflow),
        }
    }

    fn leave(&mut self) {
        match self.r_stack().pop3() {
            Ok((third, _, _)) => {
                self.state().instruction_pointer = self.data_space().get_i32(third as usize) as
                                                   usize;
            }
            Err(_) => self.abort_with(ReturnStackUnderflow),
        }
    }

    fn p_i(&mut self) {
        match self.r_stack().last() {
            Some(i) => self.push(i),
            None => self.abort_with(ReturnStackUnderflow),
        }
    }

    fn p_j(&mut self) {
        let pos = self.r_stack().len() - 4;
        match self.r_stack().get(pos) {
            Some(j) => self.push(j),
            None => self.abort_with(ReturnStackUnderflow),
        }
    }

    fn imm_if(&mut self) {
        let idx = self.references().idx_zero_branch as i32;
        self.data_space().compile_i32(idx);
        self.data_space().compile_i32(0);
        self.here();
    }

    fn imm_else(&mut self) {
        match self.s_stack().pop() {
            Ok(if_part) => {
                let idx = self.references().idx_branch as i32;
                self.data_space().compile_i32(idx);
                self.data_space().compile_i32(0);
                self.here();
                let here = self.data_space().len();
                self.data_space()
                    .put_i32(here as i32,
                             (if_part - mem::size_of::<i32>() as isize) as usize);
            }
            Err(_) => self.abort_with(ControlStructureMismatch),
        }
    }

    fn imm_then(&mut self) {
        match self.s_stack().pop() {
            Ok(branch_part) => {
                let here = self.data_space().len();
                self.data_space()
                    .put_i32(here as i32,
                             (branch_part - mem::size_of::<i32>() as isize) as usize);
            }
            Err(_) => self.abort_with(ControlStructureMismatch),
        }
    }

    fn imm_begin(&mut self) {
        self.here();
    }

    fn imm_while(&mut self) {
        let idx = self.references().idx_zero_branch as i32;
        self.data_space().compile_i32(idx);
        self.data_space().compile_i32(0);
        self.here();
    }

    fn imm_repeat(&mut self) {
        match self.s_stack().pop2() {
            Ok((begin_part, while_part)) => {
                let idx = self.references().idx_branch as i32;
                self.data_space().compile_i32(idx);
                self.data_space().compile_i32(begin_part as i32);
                let here = self.data_space().len();
                self.data_space()
                    .put_i32(here as i32,
                             (while_part - mem::size_of::<i32>() as isize) as usize);
            }
            Err(_) => self.abort_with(ControlStructureMismatch),
        }
    }

    fn imm_again(&mut self) {
        match self.s_stack().pop() {
            Ok(begin_part) => {
                let idx = self.references().idx_branch as i32;
                self.data_space().compile_i32(idx);
                self.data_space().compile_i32(begin_part as i32);
            }
            Err(_) => self.abort_with(ControlStructureMismatch),
        }
    }

    fn imm_recurse(&mut self) {
        let last = self.wordlist().len() - 1;
        self.data_space().compile_u32(last as u32);
    }

    /// Execution: ( -- a-ddr )
    ///
    /// Append the run-time semantics of `_do` to the current definition.
    /// The semantics are incomplete until resolved by `LOOP` or `+LOOP`.
    fn imm_do(&mut self) {
        let idx = self.references().idx_do as i32;
        self.data_space().compile_i32(idx);
        self.data_space().compile_i32(0);
        self.here();
    }

    /// Run-time: ( a-addr -- )
    ///
    /// Append the run-time semantics of `_LOOP` to the current definition.
    /// Resolve the destination of all unresolved occurrences of `LEAVE` between
    /// the location given by do-sys and the next location for a transfer of
    /// control, to execute the words following the `LOOP`.
    fn imm_loop(&mut self) {
        match self.s_stack().pop() {
            Ok(do_part) => {
                let idx = self.references().idx_loop as i32;
                self.data_space().compile_i32(idx);
                self.data_space().compile_i32(do_part as i32);
                let here = self.data_space().len();
                self.data_space()
                    .put_i32(here as i32,
                             (do_part - mem::size_of::<i32>() as isize) as usize);
            }
            Err(_) => self.abort_with(ControlStructureMismatch),
        }
    }

    /// Run-time: ( a-addr -- )
    ///
    /// Append the run-time semantics of `_+LOOP` to the current definition.
    /// Resolve the destination of all unresolved occurrences of `LEAVE` between
    /// the location given by do-sys and the next location for a transfer of
    /// control, to execute the words following `+LOOP`.
    fn imm_plus_loop(&mut self) {
        match self.s_stack().pop() {
            Ok(do_part) => {
                let idx = self.references().idx_plus_loop as i32;
                self.data_space().compile_i32(idx);
                self.data_space().compile_i32(do_part as i32);
                let here = self.data_space().len();
                self.data_space()
                    .put_i32(here as i32,
                             (do_part - mem::size_of::<i32>() as isize) as usize);
            }
            Err(_) => self.abort_with(ControlStructureMismatch),
        }
    }

    // -----------
    // Primitives
    // -----------

    /// Run-time: ( -- )
    ///
    /// No operation
    fn noop(&mut self) {
        // Do nothing
    }

    /// Run-time: ( -- true )
    ///
    /// Return a true flag, a single-cell value with all bits set.
    fn p_true(&mut self) {
        self.push(TRUE);
    }

    /// Run-time: ( -- false )
    ///
    /// Return a false flag.
    fn p_false(&mut self) {
        self.push(FALSE);
    }

    /// Run-time: (c-addr1 -- c-addr2 )
    ///
    ///Add the size in address units of a character to `c-addr1`, giving `c-addr2`.
    fn char_plus(&mut self) {
        match self.s_stack().pop() {
            Ok(v) => self.push(v + mem::size_of::<u8>() as isize),
            Err(_) => self.abort_with(StackUnderflow),
        }
    }

    /// Run-time: (n1 -- n2 )
    ///
    /// `n2` is the size in address units of `n1` characters.
    fn chars(&mut self) {
        match self.s_stack().pop() {
            Ok(v) => self.push(v * mem::size_of::<u8>() as isize),
            Err(_) => self.abort_with(StackUnderflow),
        }
    }


    /// Run-time: (a-addr1 -- a-addr2 )
    ///
    /// Add the size in address units of a cell to `a-addr1`, giving `a-addr2`.
    fn cell_plus(&mut self) {
        match self.s_stack().pop() {
            Ok(v) => self.push(v + mem::size_of::<i32>() as isize),
            Err(_) => self.abort_with(StackUnderflow),
        }
    }

    /// Run-time: (n1 -- n2 )
    ///
    /// `n2` is the size in address units of `n1` cells.
    fn cells(&mut self) {
        match self.s_stack().pop() {
            Ok(v) => self.push(v * mem::size_of::<i32>() as isize),
            Err(_) => self.abort_with(StackUnderflow),
        }
    }

    fn lit(&mut self) {
        if self.s_stack().is_full() {
            self.abort_with(StackOverflow);
        } else {
            unsafe {
                let ip = self.state().instruction_pointer;
                let v = self.data_space().get_i32(ip) as isize;
                ptr::write(self.s_stack()
                               .inner
                               .offset((self.s_stack().len) as isize),
                           v);
            }
            self.s_stack().len += 1;
            self.state().instruction_pointer = self.state().instruction_pointer +
                                               mem::size_of::<i32>();
        }
    }

    fn flit(&mut self) {
        let ip = self.state().instruction_pointer as usize;
        let v = self.data_space().get_f64(ip);
        match self.f_stack().push(v) {
            Err(_) => self.abort_with(FloatingPointStackOverflow),
            Ok(()) => {
                self.state().instruction_pointer = self.state().instruction_pointer +
                                                   mem::size_of::<f64>();
            }
        }
    }

    /// Runtime of S"
    fn p_s_quote(&mut self) {
        let ip = self.state().instruction_pointer;
        let cnt = self.data_space().get_i32(ip);
        let addr = self.state().instruction_pointer + mem::size_of::<i32>();
        match self.s_stack().push2(addr as isize, cnt as isize) {
            Err(_) => self.abort_with(StackOverflow),
            Ok(()) => {
                self.state().instruction_pointer = self.state().instruction_pointer +
                                                   mem::size_of::<i32>() +
                                                   cnt as usize;
            }
        }
    }

    fn swap(&mut self) {
        if self.s_stack().len < 2 {
            self.abort_with(StackUnderflow);
        } else {
            unsafe {
                let t = ptr::read(self.s_stack()
                                      .inner
                                      .offset((self.s_stack().len - 1) as isize));
                ptr::write(self.s_stack()
                               .inner
                               .offset((self.s_stack().len - 1) as isize),
                           ptr::read(self.s_stack()
                                         .inner
                                         .offset((self.s_stack().len - 2) as isize)));
                ptr::write(self.s_stack()
                               .inner
                               .offset((self.s_stack().len - 2) as isize),
                           t);
            }
        }
    }

    fn dup(&mut self) {
        if self.s_stack().len < 1 {
            self.abort_with(StackUnderflow);
        } else if self.s_stack().is_full() {
            self.abort_with(StackOverflow);
        } else {
            unsafe {
                ptr::write(self.s_stack()
                               .inner
                               .offset((self.s_stack().len) as isize),
                           ptr::read(self.s_stack()
                                         .inner
                                         .offset((self.s_stack().len - 1) as isize)));
                self.s_stack().len += 1;
            }
        }
    }

    fn p_drop(&mut self) {
        if self.s_stack().len < 1 {
            self.abort_with(StackUnderflow);
        } else {
            self.s_stack().len -= 1;
        }
    }

    fn nip(&mut self) {
        if self.s_stack().len < 2 {
            self.abort_with(StackUnderflow);
        } else {
            unsafe {
                self.s_stack().len -= 1;
                ptr::write(self.s_stack()
                               .inner
                               .offset((self.s_stack().len - 1) as isize),
                           ptr::read(self.s_stack()
                                         .inner
                                         .offset((self.s_stack().len) as isize)));
            }
        }
    }

    fn over(&mut self) {
        if self.s_stack().len < 2 {
            self.abort_with(StackUnderflow);
        } else if self.s_stack().is_full() {
            self.abort_with(StackOverflow);
        } else {
            unsafe {
                ptr::write(self.s_stack()
                               .inner
                               .offset((self.s_stack().len) as isize),
                           ptr::read(self.s_stack()
                                         .inner
                                         .offset((self.s_stack().len - 2) as isize)));
                self.s_stack().len += 1;
            }
        }
    }

    fn rot(&mut self) {
        if self.s_stack().len < 3 {
            self.abort_with(StackUnderflow);
        } else {
            unsafe {
                let t = ptr::read(self.s_stack()
                                      .inner
                                      .offset((self.s_stack().len - 1) as isize));
                let n = ptr::read(self.s_stack()
                                      .inner
                                      .offset((self.s_stack().len - 2) as isize));
                ptr::write(self.s_stack()
                               .inner
                               .offset((self.s_stack().len - 1) as isize),
                           ptr::read(self.s_stack()
                                         .inner
                                         .offset((self.s_stack().len - 3) as isize)));
                ptr::write(self.s_stack()
                               .inner
                               .offset((self.s_stack().len - 2) as isize),
                           t);
                ptr::write(self.s_stack()
                               .inner
                               .offset((self.s_stack().len - 3) as isize),
                           n);
            }
        }
    }

    fn two_drop(&mut self) {
        if self.s_stack().len < 2 {
            self.abort_with(StackUnderflow);
        } else {
            self.s_stack().len -= 2;
        }
    }

    fn two_dup(&mut self) {
        if self.s_stack().len < 2 {
            self.abort_with(StackUnderflow);
        } else if self.s_stack().len + 2 > self.s_stack().cap {
            self.abort_with(StackOverflow);
        } else {
            unsafe {
                self.s_stack().len += 2;
                ptr::write(self.s_stack()
                               .inner
                               .offset((self.s_stack().len - 1) as isize),
                           ptr::read(self.s_stack()
                                         .inner
                                         .offset((self.s_stack().len - 3) as isize)));
                ptr::write(self.s_stack()
                               .inner
                               .offset((self.s_stack().len - 2) as isize),
                           ptr::read(self.s_stack()
                                         .inner
                                         .offset((self.s_stack().len - 4) as isize)));
            }
        }
    }

    fn two_swap(&mut self) {
        if self.s_stack().len < 4 {
            self.abort_with(StackUnderflow);
        } else {
            unsafe {
                let t = ptr::read(self.s_stack()
                                      .inner
                                      .offset((self.s_stack().len - 1) as isize));
                let n = ptr::read(self.s_stack()
                                      .inner
                                      .offset((self.s_stack().len - 2) as isize));
                ptr::write(self.s_stack()
                               .inner
                               .offset((self.s_stack().len - 1) as isize),
                           ptr::read(self.s_stack()
                                         .inner
                                         .offset((self.s_stack().len - 3) as isize)));
                ptr::write(self.s_stack()
                               .inner
                               .offset((self.s_stack().len - 2) as isize),
                           ptr::read(self.s_stack()
                                         .inner
                                         .offset((self.s_stack().len - 4) as isize)));
                ptr::write(self.s_stack()
                               .inner
                               .offset((self.s_stack().len - 3) as isize),
                           t);
                ptr::write(self.s_stack()
                               .inner
                               .offset((self.s_stack().len - 4) as isize),
                           n);
            }
        }
    }

    fn two_over(&mut self) {
        if self.s_stack().len < 4 {
            self.abort_with(StackUnderflow);
        } else if self.s_stack().len + 2 > self.s_stack().cap {
            self.abort_with(StackOverflow);
        } else {
            unsafe {
                self.s_stack().len += 2;
                ptr::write(self.s_stack()
                               .inner
                               .offset((self.s_stack().len - 1) as isize),
                           ptr::read(self.s_stack()
                                         .inner
                                         .offset((self.s_stack().len - 5) as isize)));
                ptr::write(self.s_stack()
                               .inner
                               .offset((self.s_stack().len - 2) as isize),
                           ptr::read(self.s_stack()
                                         .inner
                                         .offset((self.s_stack().len - 6) as isize)));
            }
        }
    }

    fn depth(&mut self) {
        let len = self.s_stack().len;
        self.push(len as isize);
    }

    fn one_plus(&mut self) {
        if self.s_stack().len < 1 {
            self.abort_with(StackUnderflow);
        } else {
            unsafe {
                ptr::write(self.s_stack()
                               .inner
                               .offset((self.s_stack().len - 1) as isize),
                           ptr::read(self.s_stack()
                                         .inner
                                         .offset((self.s_stack().len - 1) as isize))
                                   .wrapping_add(1));
            }
        }
    }

    fn one_minus(&mut self) {
        if self.s_stack().len < 1 {
            self.abort_with(StackUnderflow);
        } else {
            unsafe {
                ptr::write(self.s_stack()
                               .inner
                               .offset((self.s_stack().len - 1) as isize),
                           ptr::read(self.s_stack()
                                         .inner
                                         .offset((self.s_stack().len - 1) as isize)) -
                           1);
            }
        }
    }

    fn plus(&mut self) {
        if self.s_stack().len < 2 {
            self.abort_with(StackUnderflow);
        } else {
            unsafe {
                self.s_stack().len -= 1;
                ptr::write(self.s_stack()
                               .inner
                               .offset((self.s_stack().len - 1) as isize),
                           ptr::read(self.s_stack()
                                         .inner
                                         .offset((self.s_stack().len - 1) as isize)) +
                           ptr::read(self.s_stack()
                                         .inner
                                         .offset((self.s_stack().len) as isize)));
            }
        }
    }

    fn minus(&mut self) {
        if self.s_stack().len < 2 {
            self.abort_with(StackUnderflow);
        } else {
            unsafe {
                self.s_stack().len -= 1;
                ptr::write(self.s_stack()
                               .inner
                               .offset((self.s_stack().len - 1) as isize),
                           ptr::read(self.s_stack()
                                         .inner
                                         .offset((self.s_stack().len - 1) as isize)) -
                           ptr::read(self.s_stack()
                                         .inner
                                         .offset((self.s_stack().len) as isize)));
            }
        }
    }

    fn star(&mut self) {
        if self.s_stack().len < 2 {
            self.abort_with(StackUnderflow);
        } else {
            unsafe {
                self.s_stack().len -= 1;
                ptr::write(self.s_stack()
                               .inner
                               .offset((self.s_stack().len - 1) as isize),
                           ptr::read(self.s_stack()
                                         .inner
                                         .offset((self.s_stack().len - 1) as isize)) *
                           ptr::read(self.s_stack()
                                         .inner
                                         .offset((self.s_stack().len) as isize)));
            }
        }
    }

    fn slash(&mut self) {
        if self.s_stack().len < 2 {
            self.abort_with(StackUnderflow);
        } else {
            unsafe {
                self.s_stack().len -= 1;
                ptr::write(self.s_stack()
                               .inner
                               .offset((self.s_stack().len - 1) as isize),
                           ptr::read(self.s_stack()
                                         .inner
                                         .offset((self.s_stack().len - 1) as isize)) /
                           ptr::read(self.s_stack()
                                         .inner
                                         .offset((self.s_stack().len) as isize)));
            }
        }
    }

    fn p_mod(&mut self) {
        if self.s_stack().len < 2 {
            self.abort_with(StackUnderflow);
        } else {
            unsafe {
                self.s_stack().len -= 1;
                ptr::write(self.s_stack()
                               .inner
                               .offset((self.s_stack().len - 1) as isize),
                           ptr::read(self.s_stack()
                                         .inner
                                         .offset((self.s_stack().len - 1) as isize)) %
                           ptr::read(self.s_stack()
                                         .inner
                                         .offset((self.s_stack().len) as isize)));
            }
        }
    }

    fn slash_mod(&mut self) {
        if self.s_stack().len < 2 {
            self.abort_with(StackUnderflow);
        } else {
            unsafe {
                let t = ptr::read(self.s_stack()
                                      .inner
                                      .offset((self.s_stack().len - 1) as isize));
                let n = ptr::read(self.s_stack()
                                      .inner
                                      .offset((self.s_stack().len - 2) as isize));
                ptr::write(self.s_stack()
                               .inner
                               .offset((self.s_stack().len - 2) as isize),
                           n % t);
                ptr::write(self.s_stack()
                               .inner
                               .offset((self.s_stack().len - 1) as isize),
                           n / t);
            }
        }
    }

    fn abs(&mut self) {
        match self.s_stack().pop() {
            Ok(t) => self.push(t.abs()),
            Err(_) => self.abort_with(StackUnderflow),
        }
    }

    fn negate(&mut self) {
        match self.s_stack().pop() {
            Ok(t) => self.push(-t),
            Err(_) => self.abort_with(StackUnderflow),
        }
    }

    fn zero_less(&mut self) {
        match self.s_stack().pop() {
            Ok(t) => self.push(if t < 0 { TRUE } else { FALSE }),
            Err(_) => self.abort_with(StackUnderflow),
        }
    }

    fn zero_equals(&mut self) {
        match self.s_stack().pop() {
            Ok(t) => self.push(if t == 0 { TRUE } else { FALSE }),
            Err(_) => self.abort_with(StackUnderflow),
        }
    }

    fn zero_greater(&mut self) {
        match self.s_stack().pop() {
            Ok(t) => self.push(if t > 0 { TRUE } else { FALSE }),
            Err(_) => self.abort_with(StackUnderflow),
        }
    }

    fn zero_not_equals(&mut self) {
        match self.s_stack().pop() {
            Ok(t) => self.push(if t == 0 { FALSE } else { TRUE }),
            Err(_) => self.abort_with(StackUnderflow),
        }
    }

    fn equals(&mut self) {
        match self.s_stack().pop2() {
            Ok((n, t)) => self.push(if t == n { TRUE } else { FALSE }),
            Err(_) => self.abort_with(StackUnderflow),
        }
    }

    fn less_than(&mut self) {
        match self.s_stack().pop2() {
            Ok((n, t)) => self.push(if n < t { TRUE } else { FALSE }),
            Err(_) => self.abort_with(StackUnderflow),
        }
    }

    fn greater_than(&mut self) {
        match self.s_stack().pop2() {
            Ok((n, t)) => self.push(if n > t { TRUE } else { FALSE }),
            Err(_) => self.abort_with(StackUnderflow),
        }
    }

    fn not_equals(&mut self) {
        match self.s_stack().pop2() {
            Ok((n, t)) => self.push(if n == t { FALSE } else { TRUE }),
            Err(_) => self.abort_with(StackUnderflow),
        }
    }

    fn between(&mut self) {
        match self.s_stack().pop3() {
            Ok((x1, x2, x3)) => self.push(if x2 <= x1 && x1 <= x3 { TRUE } else { FALSE }),
            Err(_) => self.abort_with(StackUnderflow),
        }
    }

    fn invert(&mut self) {
        match self.s_stack().pop() {
            Ok(t) => self.push(!t),
            Err(_) => self.abort_with(StackUnderflow),
        }
    }

    fn and(&mut self) {
        match self.s_stack().pop2() {
            Ok((n, t)) => self.push(t & n),
            Err(_) => self.abort_with(StackUnderflow),
        }
    }

    fn or(&mut self) {
        match self.s_stack().pop2() {
            Ok((n, t)) => self.push(t | n),
            Err(_) => self.abort_with(StackUnderflow),
        }
    }

    fn xor(&mut self) {
        match self.s_stack().pop2() {
            Ok((n, t)) => self.push(t ^ n),
            Err(_) => self.abort_with(StackUnderflow),
        }
    }

    /// Run-time: ( x1 u -- x2 )
    ///
    /// Perform a logical left shift of `u` bit-places on `x1`, giving `x2`. Put
    /// zeroes into the least significant bits vacated by the shift. An
    /// ambiguous condition exists if `u` is greater than or equal to the number
    /// of bits in a cell.
    fn lshift(&mut self) {
        match self.s_stack().pop2() {
            Ok((n, t)) => self.push(n << t),
            Err(_) => self.abort_with(StackUnderflow),
        }
    }

    /// Run-time: ( x1 u -- x2 )
    ///
    /// Perform a logical right shift of `u` bit-places on `x1`, giving `x2`. Put
    /// zeroes into the most significant bits vacated by the shift. An
    /// ambiguous condition exists if `u` is greater than or equal to the number
    /// of bits in a cell.
    fn rshift(&mut self) {
        match self.s_stack().pop2() {
            Ok((n, t)) => self.push((n as usize >> t) as isize),
            Err(_) => self.abort_with(StackUnderflow),
        }
    }

    /// Run-time: ( x1 u -- x2 )
    ///
    /// Perform a arithmetic right shift of `u` bit-places on `x1`, giving `x2`. Put
    /// zeroes into the most significant bits vacated by the shift. An
    /// ambiguous condition exists if `u` is greater than or equal to the number
    /// of bits in a cell.
    fn arshift(&mut self) {
        match self.s_stack().pop2() {
            Ok((n, t)) => self.push(n >> t),
            Err(_) => self.abort_with(StackUnderflow),
        }
    }

    /// Interpretation: Interpretation semantics for this word are undefined.
    ///
    /// Execution: ( -- ) ( R: nest-sys -- )
    /// Return control to the calling definition specified by `nest-sys`.
    /// Before executing `EXIT` within a do-loop, a program shall discard the
    /// loop-control parameters by executing `UNLOOP`.
    ///
    /// TODO: `UNLOOP`
    fn exit(&mut self) {
        if self.r_stack().len == 0 {
            self.abort_with(ReturnStackUnderflow);
        } else {
            self.r_stack().len -= 1;
            unsafe {
                self.state().instruction_pointer =
                    ptr::read(self.r_stack().inner.offset(self.r_stack().len as isize)) as usize;
            }
        }
    }

    /// Run-time: ( a-addr -- x )
    ///
    /// `x` is the value stored at `a-addr`.
    fn fetch(&mut self) {
        match self.s_stack().pop() {
            Ok(t) => {
                let value = self.data_space().get_i32(t as usize) as isize;
                self.push(value);
            }
            Err(_) => self.abort_with(StackUnderflow),
        }
    }

    /// Run-time: ( x a-addr -- )
    ///
    /// Store `x` at `a-addr`.
    fn store(&mut self) {
        match self.s_stack().pop2() {
            Ok((n, t)) => {
                self.data_space().put_i32(n as i32, t as usize);
            }
            Err(_) => self.abort_with(StackUnderflow),
        }
    }

    /// Run-time: ( c-addr -- char )
    ///
    /// Fetch the character stored at `c-addr`. When the cell size is greater than
    /// character size, the unused high-order bits are all zeroes.
    fn c_fetch(&mut self) {
        match self.s_stack().pop() {
            Ok(t) => {
                let value = self.data_space().get_u8(t as usize) as isize;
                self.push(value);
            }
            Err(_) => self.abort_with(StackUnderflow),
        }
    }

    /// Run-time: ( char c-addr -- )
    ///
    /// Store `char` at `c-addr`. When character size is smaller than cell size,
    /// only the number of low-order bits corresponding to character size are
    /// transferred.
    fn c_store(&mut self) {
        match self.s_stack().pop2() {
            Ok((n, t)) => {
                self.data_space().put_u8(n as u8, t as usize);
            }
            Err(_) => self.abort_with(StackUnderflow),
        }
    }

    /// Run-time: ( "<spaces>name" -- xt )
    ///
    /// Skip leading space delimiters. Parse name delimited by a space. Find
    /// `name` and return `xt`, the execution token for name. An ambiguous
    /// condition exists if name is not found.
    fn tick(&mut self) {
        self.parse_word();
        let last_token = self.last_token().take().unwrap();
        if last_token.is_empty() {
            self.abort_with(UnexpectedEndOfFile);
        } else {
            match self.find(&last_token) {
                Some(found_index) => self.push(found_index as isize),
                None => self.abort_with(UndefinedWord),
            }
        }
        self.set_last_token(last_token);
    }

    /// Run-time: ( i*x xt -- j*x )
    ///
    /// Remove `xt` from the stack and perform the semantics identified by it.
    /// Other stack effects are due to the word `EXECUTE`d.
    fn execute(&mut self) {
        match self.s_stack().pop() {
            Ok(t) => {
                self.execute_word(t as usize);
            }
            Err(_) => self.abort_with(StackUnderflow),
        }
    }

    /// Run-time: ( -- addr )
    ///
    /// `addr` is the data-space pointer.
    fn here(&mut self) {
        let len = self.data_space().len() as isize;
        self.push(len);
    }

    /// Run-time: ( n -- )
    ///
    /// If `n` is greater than zero, reserve n address units of data space. If `n`
    /// is less than zero, release `|n|` address units of data space. If `n` is
    /// zero, leave the data-space pointer unchanged.
    fn allot(&mut self) {
        match self.s_stack().pop() {
            Ok(v) => {
                self.data_space().allot(v);
            }
            Err(_) => self.abort_with(StackUnderflow),
        }
    }

    /// Run-time: ( x -- )
    ///
    /// Reserve one cell of data space and store `x` in the cell. If the
    /// data-space pointer is aligned when `,` begins execution, it will remain
    /// aligned when `,` finishes execution. An ambiguous condition exists if the
    /// data-space pointer is not aligned prior to execution of `,`.
    fn comma(&mut self) {
        match self.s_stack().pop() {
            Ok(v) => {
                self.data_space().compile_i32(v as i32);
            }
            Err(_) => self.abort_with(StackUnderflow),
        }
    }

    fn p_to_r(&mut self) {
        match self.s_stack().pop() {
            Ok(v) => {
                if self.r_stack().is_full() {
                    self.abort_with(ReturnStackOverflow);
                } else {
                    unsafe {
                        ptr::write(self.r_stack().inner.offset(self.r_stack().len as isize), v);
                    }
                    self.r_stack().len += 1;
                }
            }
            Err(_) => self.abort_with(StackUnderflow),
        }
    }

    fn r_from(&mut self) {
        if self.r_stack().len == 0 {
            self.abort_with(ReturnStackUnderflow);
        } else if self.s_stack().is_full() {
            self.abort_with(StackOverflow);
        } else {
            self.r_stack().len -= 1;
            unsafe {
                let r0 = self.r_stack().inner.offset(self.r_stack().len as isize);
                self.push(ptr::read(r0));
            }
        }
    }

    fn r_fetch(&mut self) {
        if self.r_stack().len == 0 {
            self.abort_with(ReturnStackUnderflow);
        } else if self.s_stack().is_full() {
            self.abort_with(StackOverflow);
        } else {
            unsafe {
                let r1 = self.r_stack()
                    .inner
                    .offset((self.r_stack().len - 1) as isize);
                self.push(ptr::read(r1));
            }
        }
    }

    fn two_to_r(&mut self) {
        match self.s_stack().pop2() {
            Ok((n, t)) => {
                if self.r_stack().space_left() < 2 {
                    self.abort_with(ReturnStackOverflow);
                } else {
                    unsafe {
                        ptr::write(self.r_stack().inner.offset(self.r_stack().len as isize), n);
                        ptr::write(self.r_stack()
                                       .inner
                                       .offset((self.r_stack().len + 1) as isize),
                                   t);
                    }
                    self.r_stack().len += 2;
                }
            }
            Err(_) => self.abort_with(StackUnderflow),
        }
    }

    fn two_r_from(&mut self) {
        // TODO: check overflow.
        if self.r_stack().len < 2 {
            self.abort_with(ReturnStackUnderflow);
        } else {
            self.r_stack().len -= 2;
            unsafe {
                let r0 = self.r_stack().inner.offset(self.r_stack().len as isize);
                self.push(ptr::read(r0));
                let r1 = self.r_stack()
                    .inner
                    .offset((self.r_stack().len + 1) as isize);
                self.push(ptr::read(r1));
            }
        }
    }

    fn two_r_fetch(&mut self) {
        if self.r_stack().len < 2 {
            self.abort_with(ReturnStackUnderflow);
        } else {
            unsafe {
                let r2 = self.r_stack()
                    .inner
                    .offset((self.r_stack().len - 2) as isize);
                self.push(ptr::read(r2));
                let r1 = self.r_stack()
                    .inner
                    .offset((self.r_stack().len - 1) as isize);
                self.push(ptr::read(r1));
            }
        }
    }

    // ----------------
    // Error handlling
    // ----------------

    fn handler_store(&mut self) {
        match self.s_stack().pop() {
            Ok(t) => {
                self.set_handler(t as usize);
            }
            Err(_) => {
                self.abort_with(StackUnderflow);
            }
        }
    }

    fn p_error_q(&mut self) {
        let value = if self.last_error().is_some() {
            TRUE
        } else {
            FALSE
        };
        self.push(value);
    }

    fn p_handle_error(&mut self) {
        match self.last_error() {
            Some(e) => {
                self.clear_stacks();
                self.set_error(None);
                match self.output_buffer().as_mut() {
                    Some(buf) => {
                        write!(buf, "{} ", e.description()).unwrap();
                    }
                    None => {}
                }
            }
            None => {}
        }
    }

    /// Clear data and floating point stacks.
    /// Called by VM's client upon Abort.
    fn clear_stacks(&mut self) {
        self.s_stack().clear();
        self.f_stack().clear();
    }

    /// Reset VM, do not clear data stack and floating point stack.
    /// Called by VM's client upon Quit.
    fn reset(&mut self) {
        self.r_stack().len = 0;
        if let Some(ref mut buf) = *self.input_buffer() {
            buf.clear()
        }
        self.state().source_index = 0;
        self.left_bracket();
        self.set_error(None);
    }

    /// Abort the inner loop with an exception, reset VM and clears stacks.
    fn abort_with(&mut self, e: Exception) {
        self.clear_stacks();
        self.set_error(Some(e));
        let h = self.handler();
        self.execute_word(h);
    }

    /// Abort the inner loop with an exception, reset VM and clears stacks.
    fn abort(&mut self) {
        self.abort_with(Abort);
    }

    fn halt(&mut self) {
        self.state().instruction_pointer = 0;
    }

    fn bye(&mut self) {
        process::exit(0);
    }

    /// Jit
    fn jit(&mut self) {
        self.push(jitmem::jit_3()() as isize);
    }
}

#[cfg(test)]
mod tests {
    extern crate test;
    use super::Core;
    use vm::VM;
    use self::test::Bencher;
    use std::mem;
    use exception::Exception::{Abort, StackUnderflow, InterpretingACompileOnlyWord, UndefinedWord,
                               UnexpectedEndOfFile, ControlStructureMismatch, ReturnStackUnderflow};
    use loader::HasLoader;

    #[bench]
    fn bench_noop(b: &mut Bencher) {
        let vm = &mut VM::new(16);
        b.iter(|| vm.noop());
    }

    #[test]
    fn test_find() {
        let vm = &mut VM::new(16);
        assert!(vm.find("").is_none());
        assert!(vm.find("word-not-exist").is_none());
        vm.find("noop").expect("noop not found");
    }

    #[bench]
    fn bench_find_word_not_exist(b: &mut Bencher) {
        let vm = &mut VM::new(16);
        b.iter(|| vm.find("unknown"));
    }

    #[bench]
    fn bench_find_word_at_beginning_of_wordlist(b: &mut Bencher) {
        let vm = &mut VM::new(16);
        b.iter(|| vm.find("noop"));
    }

    #[bench]
    fn bench_find_word_at_end_of_wordlist(b: &mut Bencher) {
        let vm = &mut VM::new(16);
        b.iter(|| vm.find("bye"));
    }

    #[bench]
    fn bench_inner_interpreter_without_nest(b: &mut Bencher) {
        let vm = &mut VM::new(16);
        let ip = vm.data_space().len();
        let idx = vm.find("noop").expect("noop not exists");
        vm.compile_word(idx);
        vm.compile_word(idx);
        vm.compile_word(idx);
        vm.compile_word(idx);
        vm.compile_word(idx);
        vm.compile_word(idx);
        vm.compile_word(idx);
        b.iter(|| {
                   vm.state().instruction_pointer = ip;
                   vm.run();
               });
    }

    #[test]
    fn test_drop() {
        let vm = &mut VM::new(16);
        vm.p_drop();
        assert_eq!(vm.last_error(), Some(StackUnderflow));
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(1).unwrap();
        vm.p_drop();
        assert!(vm.s_stack().is_empty());
        assert!(vm.last_error().is_none());
    }

    #[bench]
    fn bench_drop(b: &mut Bencher) {
        let vm = &mut VM::new(16);
        vm.s_stack().push(1).unwrap();
        b.iter(|| {
                   vm.p_drop();
                   vm.s_stack().push(1).unwrap();
               });
    }

    #[test]
    fn test_nip() {
        let vm = &mut VM::new(16);
        vm.nip();
        assert_eq!(vm.last_error(), Some(StackUnderflow));
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(1).unwrap();
        vm.nip();
        assert_eq!(vm.last_error(), Some(StackUnderflow));
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(1).unwrap();
        vm.s_stack().push(2).unwrap();
        vm.nip();
        assert!(vm.last_error().is_none());
        assert!(vm.s_stack().len() == 1);
        assert!(vm.s_stack().last() == Some(2));
    }

    #[bench]
    fn bench_nip(b: &mut Bencher) {
        let vm = &mut VM::new(16);
        vm.s_stack().push(1).unwrap();
        vm.s_stack().push(1).unwrap();
        b.iter(|| {
                   vm.nip();
                   vm.s_stack().push(1).unwrap();
               });
    }

    #[test]
    fn test_swap() {
        let vm = &mut VM::new(16);
        vm.swap();
        assert_eq!(vm.last_error(), Some(StackUnderflow));
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(1).unwrap();
        vm.swap();
        assert_eq!(vm.last_error(), Some(StackUnderflow));
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(1).unwrap();
        vm.s_stack().push(2).unwrap();
        vm.swap();
        assert!(vm.last_error().is_none());
        assert_eq!(vm.s_stack().len(), 2);
        assert_eq!(vm.s_stack().pop(), Ok(1));
        assert_eq!(vm.s_stack().pop(), Ok(2));
    }

    #[bench]
    fn bench_swap(b: &mut Bencher) {
        let vm = &mut VM::new(16);
        vm.s_stack().push(1).unwrap();
        vm.s_stack().push(2).unwrap();
        b.iter(|| vm.swap());
    }

    #[test]
    fn test_dup() {
        let vm = &mut VM::new(16);
        vm.dup();
        assert_eq!(vm.last_error(), Some(StackUnderflow));
        vm.reset();
        vm.s_stack().push(1).unwrap();
        vm.dup();
        assert!(vm.last_error().is_none());
        assert_eq!(vm.s_stack().len(), 2);
        assert_eq!(vm.s_stack().pop(), Ok(1));
        assert_eq!(vm.s_stack().pop(), Ok(1));
    }

    #[bench]
    fn bench_dup(b: &mut Bencher) {
        let vm = &mut VM::new(16);
        vm.s_stack().push(1).unwrap();
        b.iter(|| {
                   vm.dup();
                   vm.s_stack().pop().unwrap();
               });
    }

    #[test]
    fn test_over() {
        let vm = &mut VM::new(16);
        vm.over();
        assert_eq!(vm.last_error(), Some(StackUnderflow));
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(1).unwrap();
        vm.over();
        assert_eq!(vm.last_error(), Some(StackUnderflow));
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(1).unwrap();
        vm.s_stack().push(2).unwrap();
        vm.over();
        assert!(vm.last_error().is_none());
        assert_eq!(vm.s_stack().len(), 3);
        assert_eq!(vm.s_stack().pop(), Ok(1));
        assert_eq!(vm.s_stack().pop(), Ok(2));
        assert_eq!(vm.s_stack().pop(), Ok(1));
    }

    #[bench]
    fn bench_over(b: &mut Bencher) {
        let vm = &mut VM::new(16);
        vm.s_stack().push(1).unwrap();
        vm.s_stack().push(2).unwrap();
        b.iter(|| {
                   vm.over();
                   vm.s_stack().pop().unwrap();
               });
    }

    #[test]
    fn test_rot() {
        let vm = &mut VM::new(16);
        vm.rot();
        assert_eq!(vm.last_error(), Some(StackUnderflow));
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(1).unwrap();
        vm.rot();
        assert_eq!(vm.last_error(), Some(StackUnderflow));
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(1).unwrap();
        vm.s_stack().push(2).unwrap();
        vm.rot();
        assert_eq!(vm.last_error(), Some(StackUnderflow));
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(1).unwrap();
        vm.s_stack().push(2).unwrap();
        vm.s_stack().push(3).unwrap();
        vm.rot();
        assert!(vm.last_error().is_none());
        assert_eq!(vm.s_stack().len(), 3);
        assert_eq!(vm.s_stack().pop(), Ok(1));
        assert_eq!(vm.s_stack().pop(), Ok(3));
        assert_eq!(vm.s_stack().pop(), Ok(2));
    }

    #[bench]
    fn bench_rot(b: &mut Bencher) {
        let vm = &mut VM::new(16);
        vm.s_stack().push(1).unwrap();
        vm.s_stack().push(2).unwrap();
        vm.s_stack().push(3).unwrap();
        b.iter(|| vm.rot());
    }

    #[test]
    fn test_2drop() {
        let vm = &mut VM::new(16);
        vm.two_drop();
        assert_eq!(vm.last_error(), Some(StackUnderflow));
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(1).unwrap();
        vm.two_drop();
        assert_eq!(vm.last_error(), Some(StackUnderflow));
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(1).unwrap();
        vm.s_stack().push(2).unwrap();
        vm.two_drop();
        assert!(vm.last_error().is_none());
        assert!(vm.s_stack().is_empty());
    }

    #[bench]
    fn bench_2drop(b: &mut Bencher) {
        let vm = &mut VM::new(16);
        b.iter(|| {
                   vm.s_stack().push(1).unwrap();
                   vm.s_stack().push(2).unwrap();
                   vm.two_drop();
               });
    }

    #[test]
    fn test_2dup() {
        let vm = &mut VM::new(16);
        vm.two_dup();
        assert_eq!(vm.last_error(), Some(StackUnderflow));
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(1).unwrap();
        vm.two_dup();
        assert_eq!(vm.last_error(), Some(StackUnderflow));
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(1).unwrap();
        vm.s_stack().push(2).unwrap();
        vm.two_dup();
        assert!(vm.last_error().is_none());
        assert_eq!(vm.s_stack().len(), 4);
        assert_eq!(vm.s_stack().pop(), Ok(2));
        assert_eq!(vm.s_stack().pop(), Ok(1));
        assert_eq!(vm.s_stack().pop(), Ok(2));
        assert_eq!(vm.s_stack().pop(), Ok(1));
    }

    #[bench]
    fn bench_2dup(b: &mut Bencher) {
        let vm = &mut VM::new(16);
        vm.s_stack().push(1).unwrap();
        vm.s_stack().push(2).unwrap();
        b.iter(|| {
                   vm.two_dup();
                   vm.two_drop();
               });
    }

    #[test]
    fn test_2swap() {
        let vm = &mut VM::new(16);
        vm.two_swap();
        assert_eq!(vm.last_error(), Some(StackUnderflow));
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(1).unwrap();
        vm.two_swap();
        assert_eq!(vm.last_error(), Some(StackUnderflow));
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(1).unwrap();
        vm.s_stack().push(2).unwrap();
        vm.two_swap();
        assert_eq!(vm.last_error(), Some(StackUnderflow));
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(1).unwrap();
        vm.s_stack().push(2).unwrap();
        vm.s_stack().push(3).unwrap();
        vm.two_swap();
        assert_eq!(vm.last_error(), Some(StackUnderflow));
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(1).unwrap();
        vm.s_stack().push(2).unwrap();
        vm.s_stack().push(3).unwrap();
        vm.s_stack().push(4).unwrap();
        vm.two_swap();
        assert!(vm.last_error().is_none());
        assert_eq!(vm.s_stack().len(), 4);
        assert_eq!(vm.s_stack().pop(), Ok(2));
        assert_eq!(vm.s_stack().pop(), Ok(1));
        assert_eq!(vm.s_stack().pop(), Ok(4));
        assert_eq!(vm.s_stack().pop(), Ok(3));
    }

    #[bench]
    fn bench_2swap(b: &mut Bencher) {
        let vm = &mut VM::new(16);
        vm.s_stack().push(1).unwrap();
        vm.s_stack().push(2).unwrap();
        vm.s_stack().push(3).unwrap();
        vm.s_stack().push(4).unwrap();
        b.iter(|| vm.two_swap());
    }

    #[test]
    fn test_2over() {
        let vm = &mut VM::new(16);
        vm.two_over();
        assert_eq!(vm.last_error(), Some(StackUnderflow));
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(1).unwrap();
        vm.two_over();
        assert_eq!(vm.last_error(), Some(StackUnderflow));
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(1).unwrap();
        vm.s_stack().push(2).unwrap();
        vm.two_over();
        assert_eq!(vm.last_error(), Some(StackUnderflow));
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(1).unwrap();
        vm.s_stack().push(2).unwrap();
        vm.s_stack().push(3).unwrap();
        vm.two_over();
        assert_eq!(vm.last_error(), Some(StackUnderflow));
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(1).unwrap();
        vm.s_stack().push(2).unwrap();
        vm.s_stack().push(3).unwrap();
        vm.s_stack().push(4).unwrap();
        vm.two_over();
        assert!(vm.last_error().is_none());
        assert_eq!(vm.s_stack().len(), 6);
        assert_eq!(vm.s_stack().as_slice(), [1, 2, 3, 4, 1, 2]);
    }

    #[bench]
    fn bench_2over(b: &mut Bencher) {
        let vm = &mut VM::new(16);
        vm.s_stack().push(1).unwrap();
        vm.s_stack().push(2).unwrap();
        vm.s_stack().push(3).unwrap();
        vm.s_stack().push(4).unwrap();
        b.iter(|| {
                   vm.two_over();
                   vm.two_drop();
               });
    }

    #[test]
    fn test_depth() {
        let vm = &mut VM::new(16);
        vm.depth();
        vm.depth();
        vm.depth();
        assert_eq!(vm.s_stack().as_slice(), [0, 1, 2]);
    }

    #[test]
    fn test_one_plus() {
        let vm = &mut VM::new(16);
        vm.one_plus();
        assert_eq!(vm.last_error(), Some(StackUnderflow));
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(1).unwrap();
        vm.one_plus();
        assert!(vm.last_error().is_none());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), Ok(2));
    }

    #[bench]
    fn bench_one_plus(b: &mut Bencher) {
        let vm = &mut VM::new(16);
        vm.s_stack().push(0).unwrap();
        b.iter(|| { vm.one_plus(); });
    }

    #[test]
    fn test_one_minus() {
        let vm = &mut VM::new(16);
        vm.one_minus();
        assert_eq!(vm.last_error(), Some(StackUnderflow));
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(2).unwrap();
        vm.one_minus();
        assert!(vm.last_error().is_none());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), Ok(1));
    }

    #[bench]
    fn bench_one_minus(b: &mut Bencher) {
        let vm = &mut VM::new(16);
        vm.s_stack().push(0).unwrap();
        b.iter(|| { vm.one_minus(); });
    }

    #[test]
    fn test_minus() {
        let vm = &mut VM::new(16);
        vm.minus();
        assert_eq!(vm.last_error(), Some(StackUnderflow));
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(5).unwrap();
        vm.minus();
        assert_eq!(vm.last_error(), Some(StackUnderflow));
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(5).unwrap();
        vm.s_stack().push(7).unwrap();
        vm.minus();
        assert_eq!(vm.last_error(), None);
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), Ok(-2));
    }

    #[bench]
    fn bench_minus(b: &mut Bencher) {
        let vm = &mut VM::new(16);
        vm.s_stack().push(0).unwrap();
        b.iter(|| {
                   vm.dup();
                   vm.minus();
               });
    }

    #[test]
    fn test_plus() {
        let vm = &mut VM::new(16);
        vm.plus();
        assert_eq!(vm.last_error(), Some(StackUnderflow));
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(5).unwrap();
        vm.plus();
        assert_eq!(vm.last_error(), Some(StackUnderflow));
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(5).unwrap();
        vm.s_stack().push(7).unwrap();
        vm.plus();
        assert!(vm.last_error().is_none());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), Ok(12));
    }

    #[bench]
    fn bench_plus(b: &mut Bencher) {
        let vm = &mut VM::new(16);
        vm.s_stack().push(1).unwrap();
        b.iter(|| {
                   vm.dup();
                   vm.plus();
               });
    }

    #[test]
    fn test_star() {
        let vm = &mut VM::new(16);
        vm.star();
        assert_eq!(vm.last_error(), Some(StackUnderflow));
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(5).unwrap();
        vm.star();
        assert_eq!(vm.last_error(), Some(StackUnderflow));
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(5).unwrap();
        vm.s_stack().push(7).unwrap();
        vm.star();
        assert!(vm.last_error().is_none());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), Ok(35));
    }

    #[bench]
    fn bench_star(b: &mut Bencher) {
        let vm = &mut VM::new(16);
        vm.s_stack().push(1).unwrap();
        b.iter(|| {
                   vm.dup();
                   vm.star();
               });
    }

    #[test]
    fn test_slash() {
        let vm = &mut VM::new(16);
        vm.slash();
        assert_eq!(vm.last_error(), Some(StackUnderflow));
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(30).unwrap();
        vm.slash();
        assert_eq!(vm.last_error(), Some(StackUnderflow));
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(30).unwrap();
        vm.s_stack().push(7).unwrap();
        vm.slash();
        assert!(vm.last_error().is_none());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), Ok(4));
    }

    #[bench]
    fn bench_slash(b: &mut Bencher) {
        let vm = &mut VM::new(16);
        vm.s_stack().push(1).unwrap();
        b.iter(|| {
                   vm.dup();
                   vm.slash();
               });
    }

    #[test]
    fn test_mod() {
        let vm = &mut VM::new(16);
        vm.p_mod();
        assert_eq!(vm.last_error(), Some(StackUnderflow));
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(30).unwrap();
        vm.p_mod();
        assert_eq!(vm.last_error(), Some(StackUnderflow));
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(30).unwrap();
        vm.s_stack().push(7).unwrap();
        vm.p_mod();
        assert!(vm.last_error().is_none());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), Ok(2));
    }

    #[bench]
    fn bench_mod(b: &mut Bencher) {
        let vm = &mut VM::new(16);
        vm.s_stack().push(1).unwrap();
        vm.s_stack().push(2).unwrap();
        b.iter(|| {
                   vm.p_mod();
                   vm.s_stack().push(2).unwrap();
               });
    }

    #[test]
    fn test_slash_mod() {
        let vm = &mut VM::new(16);
        vm.slash_mod();
        assert_eq!(vm.last_error(), Some(StackUnderflow));
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(30).unwrap();
        vm.slash_mod();
        assert_eq!(vm.last_error(), Some(StackUnderflow));
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(30).unwrap();
        vm.s_stack().push(7).unwrap();
        vm.slash_mod();
        assert!(vm.last_error().is_none());
        assert_eq!(vm.s_stack().len(), 2);
        assert_eq!(vm.s_stack().pop(), Ok(4));
        assert_eq!(vm.s_stack().pop(), Ok(2));
    }

    #[bench]
    fn bench_slash_mod(b: &mut Bencher) {
        let vm = &mut VM::new(16);
        vm.s_stack().push2(1, 2).unwrap();
        b.iter(|| {
                   vm.slash_mod();
                   vm.p_drop();
                   vm.s_stack().push(2).unwrap();
               });
    }

    #[test]
    fn test_abs() {
        let vm = &mut VM::new(16);
        vm.abs();
        assert_eq!(vm.last_error(), Some(StackUnderflow));
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(-30).unwrap();
        vm.abs();
        assert!(vm.last_error().is_none());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), Ok(30));
    }

    #[test]
    fn test_negate() {
        let vm = &mut VM::new(16);
        vm.negate();
        assert_eq!(vm.last_error(), Some(StackUnderflow));
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(30).unwrap();
        vm.negate();
        assert!(vm.last_error().is_none());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), Ok(-30));
    }

    #[test]
    fn test_zero_less() {
        let vm = &mut VM::new(16);
        vm.zero_less();
        assert_eq!(vm.last_error(), Some(StackUnderflow));
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(-1).unwrap();
        vm.zero_less();
        assert!(vm.last_error().is_none());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), Ok(-1));
        vm.s_stack().push(0).unwrap();
        vm.zero_less();
        assert!(vm.last_error().is_none());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), Ok(0));
    }

    #[test]
    fn test_zero_equals() {
        let vm = &mut VM::new(16);
        vm.zero_equals();
        assert_eq!(vm.last_error(), Some(StackUnderflow));
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(0).unwrap();
        vm.zero_equals();
        assert!(vm.last_error().is_none());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), Ok(-1));
        vm.s_stack().push(-1).unwrap();
        vm.zero_equals();
        assert!(vm.last_error().is_none());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), Ok(0));
        vm.s_stack().push(1).unwrap();
        vm.zero_equals();
        assert!(vm.last_error().is_none());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), Ok(0));
    }

    #[test]
    fn test_zero_greater() {
        let vm = &mut VM::new(16);
        vm.zero_greater();
        assert_eq!(vm.last_error(), Some(StackUnderflow));
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(1).unwrap();
        vm.zero_greater();
        assert!(vm.last_error().is_none());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), Ok(-1));
        vm.s_stack().push(0).unwrap();
        vm.zero_greater();
        assert!(vm.last_error().is_none());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), Ok(0));
    }

    #[test]
    fn test_zero_not_equals() {
        let vm = &mut VM::new(16);
        vm.zero_not_equals();
        assert_eq!(vm.last_error(), Some(StackUnderflow));
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(0).unwrap();
        vm.zero_not_equals();
        assert!(vm.last_error().is_none());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), Ok(0));
        vm.s_stack().push(-1).unwrap();
        vm.zero_not_equals();
        assert!(vm.last_error().is_none());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), Ok(-1));
        vm.s_stack().push(1).unwrap();
        vm.zero_not_equals();
        assert!(vm.last_error().is_none());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), Ok(-1));
    }

    #[test]
    fn test_less_than() {
        let vm = &mut VM::new(16);
        vm.less_than();
        assert_eq!(vm.last_error(), Some(StackUnderflow));
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(-1).unwrap();
        vm.less_than();
        assert_eq!(vm.last_error(), Some(StackUnderflow));
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(-1).unwrap();
        vm.s_stack().push(0).unwrap();
        vm.less_than();
        assert!(vm.last_error().is_none());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), Ok(-1));
        vm.s_stack().push(0).unwrap();
        vm.s_stack().push(0).unwrap();
        vm.less_than();
        assert!(vm.last_error().is_none());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), Ok(0));
    }

    #[test]
    fn test_equals() {
        let vm = &mut VM::new(16);
        vm.equals();
        assert_eq!(vm.last_error(), Some(StackUnderflow));
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(0).unwrap();
        vm.equals();
        assert_eq!(vm.last_error(), Some(StackUnderflow));
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(0).unwrap();
        vm.s_stack().push(0).unwrap();
        vm.equals();
        assert!(vm.last_error().is_none());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), Ok(-1));
        vm.s_stack().push(-1).unwrap();
        vm.s_stack().push(0).unwrap();
        vm.equals();
        assert!(vm.last_error().is_none());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), Ok(0));
        vm.s_stack().push(1).unwrap();
        vm.s_stack().push(0).unwrap();
        vm.equals();
        assert!(vm.last_error().is_none());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), Ok(0));
    }

    #[test]
    fn test_greater_than() {
        let vm = &mut VM::new(16);
        vm.greater_than();
        assert_eq!(vm.last_error(), Some(StackUnderflow));
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(1).unwrap();
        vm.greater_than();
        assert_eq!(vm.last_error(), Some(StackUnderflow));
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(1).unwrap();
        vm.s_stack().push(0).unwrap();
        vm.greater_than();
        assert!(vm.last_error().is_none());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), Ok(-1));
        vm.s_stack().push(0).unwrap();
        vm.s_stack().push(0).unwrap();
        vm.greater_than();
        assert!(vm.last_error().is_none());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), Ok(0));
    }

    #[test]
    fn test_not_equals() {
        let vm = &mut VM::new(16);
        vm.not_equals();
        assert_eq!(vm.last_error(), Some(StackUnderflow));
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(0).unwrap();
        vm.not_equals();
        assert_eq!(vm.last_error(), Some(StackUnderflow));
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(0).unwrap();
        vm.s_stack().push(0).unwrap();
        vm.not_equals();
        assert!(vm.last_error().is_none());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), Ok(0));
        vm.s_stack().push(-1).unwrap();
        vm.s_stack().push(0).unwrap();
        vm.not_equals();
        assert!(vm.last_error().is_none());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), Ok(-1));
        vm.s_stack().push(1).unwrap();
        vm.s_stack().push(0).unwrap();
        vm.not_equals();
        assert!(vm.last_error().is_none());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), Ok(-1));
    }

    #[test]
    fn test_between() {
        let vm = &mut VM::new(16);
        vm.between();
        assert_eq!(vm.last_error(), Some(StackUnderflow));
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(1).unwrap();
        vm.between();
        assert_eq!(vm.last_error(), Some(StackUnderflow));
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(1).unwrap();
        vm.s_stack().push(1).unwrap();
        vm.between();
        assert_eq!(vm.last_error(), Some(StackUnderflow));
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(1).unwrap();
        vm.s_stack().push(1).unwrap();
        vm.s_stack().push(2).unwrap();
        vm.between();
        assert!(vm.last_error().is_none());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), Ok(-1));
        vm.s_stack().push(1).unwrap();
        vm.s_stack().push(0).unwrap();
        vm.s_stack().push(1).unwrap();
        vm.between();
        assert!(vm.last_error().is_none());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), Ok(-1));
        vm.s_stack().push(0).unwrap();
        vm.s_stack().push(1).unwrap();
        vm.s_stack().push(2).unwrap();
        vm.between();
        assert!(vm.last_error().is_none());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), Ok(0));
        vm.s_stack().push(3).unwrap();
        vm.s_stack().push(1).unwrap();
        vm.s_stack().push(2).unwrap();
        vm.between();
        assert!(vm.last_error().is_none());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), Ok(0));
    }

    #[test]
    fn test_invert() {
        let vm = &mut VM::new(16);
        vm.invert();
        assert_eq!(vm.last_error(), Some(StackUnderflow));
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(707).unwrap();
        vm.invert();
        assert!(vm.last_error().is_none());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), Ok(-708));
    }

    #[test]
    fn test_and() {
        let vm = &mut VM::new(16);
        vm.and();
        assert_eq!(vm.last_error(), Some(StackUnderflow));
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(707).unwrap();
        vm.and();
        assert_eq!(vm.last_error(), Some(StackUnderflow));
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(707).unwrap();
        vm.s_stack().push(007).unwrap();
        vm.and();
        assert!(vm.last_error().is_none());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), Ok(3));
    }

    #[test]
    fn test_or() {
        let vm = &mut VM::new(16);
        vm.or();
        assert_eq!(vm.last_error(), Some(StackUnderflow));
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(707).unwrap();
        vm.or();
        assert_eq!(vm.last_error(), Some(StackUnderflow));
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(707).unwrap();
        vm.s_stack().push(07).unwrap();
        vm.or();
        assert!(vm.last_error().is_none());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), Ok(711));
    }

    #[test]
    fn test_xor() {
        let vm = &mut VM::new(16);
        vm.xor();
        assert_eq!(vm.last_error(), Some(StackUnderflow));
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(707).unwrap();
        vm.xor();
        assert_eq!(vm.last_error(), Some(StackUnderflow));
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(707).unwrap();
        vm.s_stack().push(07).unwrap();
        vm.xor();
        assert!(vm.last_error().is_none());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), Ok(708));
    }

    #[test]
    fn test_lshift() {
        let vm = &mut VM::new(16);
        vm.lshift();
        assert_eq!(vm.last_error(), Some(StackUnderflow));
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(1).unwrap();
        vm.lshift();
        assert_eq!(vm.last_error(), Some(StackUnderflow));
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(1).unwrap();
        vm.s_stack().push(1).unwrap();
        vm.lshift();
        assert!(vm.last_error().is_none());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), Ok(2));
        vm.s_stack().push(1).unwrap();
        vm.s_stack().push(2).unwrap();
        vm.lshift();
        assert!(vm.last_error().is_none());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), Ok(4));
    }

    #[test]
    fn test_rshift() {
        let vm = &mut VM::new(16);
        vm.rshift();
        assert_eq!(vm.last_error(), Some(StackUnderflow));
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(8).unwrap();
        vm.rshift();
        assert_eq!(vm.last_error(), Some(StackUnderflow));
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(8).unwrap();
        vm.s_stack().push(1).unwrap();
        vm.rshift();
        assert!(vm.last_error().is_none());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), Ok(4));
        vm.s_stack().push(-1).unwrap();
        vm.s_stack().push(1).unwrap();
        vm.rshift();
        assert!(vm.last_error().is_none());
        assert_eq!(vm.s_stack().len(), 1);
        assert!(vm.s_stack().pop().unwrap() > 0);
    }

    #[test]
    fn test_arshift() {
        let vm = &mut VM::new(16);
        vm.arshift();
        assert_eq!(vm.last_error(), Some(StackUnderflow));
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(8).unwrap();
        vm.arshift();
        assert_eq!(vm.last_error(), Some(StackUnderflow));
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(8).unwrap();
        vm.s_stack().push(1).unwrap();
        vm.arshift();
        assert!(vm.last_error().is_none());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), Ok(4));
        vm.s_stack().push(-8).unwrap();
        vm.s_stack().push(1).unwrap();
        vm.arshift();
        assert!(vm.last_error().is_none());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), Ok(-4));
    }

    #[test]
    fn test_parse_word() {
        let vm = &mut VM::new(16);
        vm.set_source("hello world\t\r\n\"");
        vm.parse_word();
        assert_eq!(vm.last_token().clone().unwrap(), "hello");
        assert_eq!(vm.state().source_index, 6);
        vm.parse_word();
        assert_eq!(vm.last_token().clone().unwrap(), "world");
        assert_eq!(vm.state().source_index, 12);
        vm.parse_word();
        assert_eq!(vm.last_token().clone().unwrap(), "\"");
    }

    #[test]
    fn test_evaluate() {
        let vm = &mut VM::new(16);
        // >r
        vm.set_source(">r");
        vm.evaluate();
        assert_eq!(vm.last_error(), Some(InterpretingACompileOnlyWord));
        vm.reset();
        vm.clear_stacks();
        // drop
        vm.set_source("drop");
        vm.evaluate();
        assert_eq!(vm.last_error(), Some(StackUnderflow));
        vm.reset();
        vm.clear_stacks();
        // error in colon definition: 4drop
        vm.set_source(": 4drop drop drop drop drop ; 4drop");
        vm.evaluate();
        assert_eq!(vm.last_error(), Some(StackUnderflow));
        vm.reset();
        vm.clear_stacks();
        // undefined word
        vm.set_source("xdrop");
        vm.evaluate();
        assert_eq!(vm.last_error(), Some(UndefinedWord));
        vm.reset();
        vm.clear_stacks();
        // false true dup 1+ 2 -3
        vm.set_source("false true dup 1+ 2 -3");
        vm.evaluate();
        assert!(vm.last_error().is_none());
        assert_eq!(vm.s_stack().len(), 5);
        assert_eq!(vm.s_stack().pop(), Ok(-3));
        assert_eq!(vm.s_stack().pop(), Ok(2));
        assert_eq!(vm.s_stack().pop(), Ok(0));
        assert_eq!(vm.s_stack().pop(), Ok(-1));
        assert_eq!(vm.s_stack().pop(), Ok(0));
    }

    #[bench]
    fn bench_compile_words_at_beginning_of_wordlist(b: &mut Bencher) {
        let vm = &mut VM::new(16);
        b.iter(|| {
            vm.set_source("marker empty : main noop noop noop noop noop noop noop noop ; empty");
            vm.evaluate();
            vm.s_stack().clear();
        });
    }

    #[bench]
    fn bench_compile_words_at_end_of_wordlist(b: &mut Bencher) {
        let vm = &mut VM::new(16);
        b.iter(|| {
                   vm.set_source("marker empty : main bye bye bye bye bye bye bye bye ; empty");
                   vm.evaluate();
                   vm.s_stack().clear();
               });
    }

    #[test]
    fn test_colon_and_semi_colon() {
        let vm = &mut VM::new(16);
        // :
        vm.set_source(":");
        vm.evaluate();
        assert_eq!(vm.last_error(), Some(UnexpectedEndOfFile));
        vm.reset();
        vm.clear_stacks();
        // : 2+3 2 3 + ; 2+3
        vm.set_source(": 2+3 2 3 + ; 2+3");
        vm.evaluate();
        assert!(vm.last_error().is_none());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), Ok(5));
    }

    #[test]
    fn test_constant() {
        let vm = &mut VM::new(16);
        // constant x
        vm.set_source("constant");
        vm.evaluate();
        assert_eq!(vm.last_error(), Some(StackUnderflow));
        vm.reset();
        vm.clear_stacks();
        // 5 constant x x x
        vm.set_source("5 constant x x x");
        vm.evaluate();
        assert!(vm.last_error().is_none());
        assert_eq!(vm.s_stack().len(), 2);
        assert_eq!(vm.s_stack().pop(), Ok(5));
        assert_eq!(vm.s_stack().pop(), Ok(5));
    }

    #[test]
    fn test_variable_and_store_fetch() {
        let vm = &mut VM::new(16);
        // @
        vm.set_source("@");
        vm.evaluate();
        assert_eq!(vm.last_error(), Some(StackUnderflow));
        vm.reset();
        vm.clear_stacks();
        // !
        vm.set_source("!");
        vm.evaluate();
        assert_eq!(vm.last_error(), Some(StackUnderflow));
        vm.reset();
        vm.clear_stacks();
        // variable x x !
        vm.set_source("variable x x !");
        vm.evaluate();
        assert_eq!(vm.last_error(), Some(StackUnderflow));
        vm.reset();
        vm.clear_stacks();
        // variable x  x @  3 x !  x @
        vm.set_source("variable x  x @  3 x !  x @");
        vm.evaluate();
        assert!(vm.last_error().is_none());
        assert_eq!(vm.s_stack().len(), 2);
        assert_eq!(vm.s_stack().pop(), Ok(3));
        assert_eq!(vm.s_stack().pop(), Ok(0));
    }

    #[test]
    fn test_char_plus_and_chars() {
        let vm = &mut VM::new(16);
        vm.char_plus();
        assert_eq!(vm.last_error(), Some(StackUnderflow));
        vm.reset();
        vm.chars();
        assert_eq!(vm.last_error(), Some(StackUnderflow));
        vm.reset();
        // 2 char+  9 chars
        vm.set_source("2 char+  9 chars");
        vm.evaluate();
        assert!(vm.last_error().is_none());
        assert_eq!(vm.s_stack().as_slice(), [3, 9]);
    }

    #[test]
    fn test_cell_plus_and_cells() {
        let vm = &mut VM::new(16);
        vm.cell_plus();
        assert_eq!(vm.last_error(), Some(StackUnderflow));
        vm.reset();
        vm.cells();
        assert_eq!(vm.last_error(), Some(StackUnderflow));
        vm.reset();
        vm.set_source("2 cell+  9 cells");
        vm.evaluate();
        assert!(vm.last_error().is_none());
        assert_eq!(vm.s_stack().as_slice(), [6, 36]);
    }

    #[test]
    fn test_tick() {
        let vm = &mut VM::new(16);
        // '
        vm.tick();
        assert_eq!(vm.last_error(), Some(UnexpectedEndOfFile));
        vm.reset();
        // ' xdrop
        vm.set_source("' xdrop");
        vm.evaluate();
        assert_eq!(vm.last_error(), Some(UndefinedWord));
        vm.reset();
        vm.clear_stacks();
        // ' drop
        vm.set_source("' drop");
        vm.evaluate();
        assert!(vm.last_error().is_none());
        assert_eq!(vm.s_stack().len(), 1);
    }

    #[test]
    fn test_execute() {
        let vm = &mut VM::new(16);
        // execute
        vm.execute();
        assert_eq!(vm.last_error(), Some(StackUnderflow));
        vm.reset();
        // ' drop execute
        vm.set_source("' drop");
        vm.execute();
        assert_eq!(vm.last_error(), Some(StackUnderflow));
        vm.reset();
        vm.clear_stacks();
        // 1 2  ' swap execute
        vm.set_source("1 2  ' swap execute");
        vm.evaluate();
        assert!(vm.last_error().is_none());
        assert_eq!(vm.s_stack().len(), 2);
        assert_eq!(vm.s_stack().pop(), Ok(1));
        assert_eq!(vm.s_stack().pop(), Ok(2));
    }

    #[test]
    fn test_here_allot() {
        let vm = &mut VM::new(16);
        // allot
        vm.allot();
        assert_eq!(vm.last_error(), Some(StackUnderflow));
        vm.reset();
        // here 2 cells allot here -
        vm.set_source("here 2 cells allot here -");
        vm.evaluate();
        assert!(vm.last_error().is_none());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(),
                   Ok(-((mem::size_of::<i32>() * 2) as isize)));
    }

    #[test]
    fn test_here_comma_compile_interpret() {
        let vm = &mut VM::new(16);
        vm.comma();
        assert_eq!(vm.last_error(), Some(StackUnderflow));
        vm.reset();
        // here 1 , 2 , ] lit exit [ here
        let here = vm.data_space().len();
        vm.set_source("here 1 , 2 , ] lit exit [ here");
        vm.evaluate();
        assert!(vm.last_error().is_none());
        assert_eq!(vm.s_stack().len(), 2);
        match vm.s_stack().pop2() {
            Ok((n, t)) => {
                assert_eq!(t - n, 4 * mem::size_of::<u32>() as isize);
            }
            Err(_) => {
                assert!(false);
            }
        }
        let idx_halt = vm.find("halt").expect("halt undefined");
        assert_eq!(vm.data_space().get_i32(0), idx_halt as i32);
        assert_eq!(vm.data_space().get_i32(here + 0), 1);
        assert_eq!(vm.data_space().get_i32(here + 4), 2);
        assert_eq!(vm.data_space().get_i32(here + 8),
                   vm.references().idx_lit as i32);
        assert_eq!(vm.data_space().get_i32(here + 12),
                   vm.references().idx_exit as i32);
    }

    #[test]
    fn test_to_r_r_fetch_r_from() {
        let vm = &mut VM::new(16);
        vm.set_source(": t 3 >r 2 r@ + r> + ; t");
        vm.evaluate();
        assert!(vm.last_error().is_none());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), Ok(8));
    }

    #[bench]
    fn bench_to_r_r_fetch_r_from(b: &mut Bencher) {
        let vm = &mut VM::new(16);
        vm.set_source(": main 3 >r r@ drop r> drop ;");
        vm.evaluate();
        vm.set_source("' main");
        vm.evaluate();
        b.iter(|| {
                   vm.dup();
                   vm.execute();
                   vm.run();
               });
    }

    #[test]
    fn test_two_to_r_two_r_fetch_two_r_from() {
        let vm = &mut VM::new(16);
        vm.set_source(": t 1 2 2>r 2r@ + 2r> - * ; t");
        vm.evaluate();
        assert!(vm.last_error().is_none());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), Ok(-3));
    }

    #[bench]
    fn bench_two_to_r_two_r_fetch_two_r_from(b: &mut Bencher) {
        let vm = &mut VM::new(16);
        vm.set_source(": main 1 2 2>r 2r@ 2drop 2r> 2drop ;");
        vm.evaluate();
        vm.set_source("' main");
        vm.evaluate();
        b.iter(|| {
                   vm.dup();
                   vm.execute();
                   vm.run();
               });
    }

    #[test]
    fn test_if_else_then() {
        let vm = &mut VM::new(16);
        // : t5 if ; t5
        vm.set_source(": t5 if ;");
        vm.evaluate();
        assert_eq!(vm.last_error(), Some(ControlStructureMismatch));
        vm.reset();
        vm.clear_stacks();
        // : t3 else then ; t3
        vm.set_source(": t3 else then ;");
        vm.evaluate();
        assert_eq!(vm.last_error(), Some(ControlStructureMismatch));
        vm.reset();
        vm.clear_stacks();
        // : t4 then ; t4
        vm.set_source(": t4 then ; t4");
        vm.evaluate();
        assert_eq!(vm.last_error(), Some(ControlStructureMismatch));
        vm.reset();
        vm.clear_stacks();
        // : t1 0 if true else false then ; t1
        vm.set_source(": t1 0 if true else false then ; t1");
        vm.evaluate();
        assert!(vm.last_error().is_none());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), Ok(0));
        // : t2 1 if true else false then ; t2
        vm.set_source(": t2 1 if true else false then ; t2");
        vm.evaluate();
        assert!(vm.last_error().is_none());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), Ok(-1));
    }

    #[test]
    fn test_begin_again() {
        let vm = &mut VM::new(16);
        // : t3 begin ;
        vm.set_source(": t3 begin ;");
        vm.evaluate();
        assert_eq!(vm.last_error(), Some(ControlStructureMismatch));
        vm.reset();
        vm.clear_stacks();
        // : t2 again ;
        vm.set_source(": t2 again ;");
        vm.evaluate();
        assert_eq!(vm.last_error(), Some(ControlStructureMismatch));
        vm.reset();
        vm.clear_stacks();
        // : t1 0 begin 1+ dup 3 = if exit then again ; t1
        vm.set_source(": t1 0 begin 1+ dup 3 = if exit then again ; t1");
        vm.evaluate();
        assert_eq!(vm.last_error(), None);
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), Ok(3));
    }

    #[test]
    fn test_begin_while_repeat() {
        let vm = &mut VM::new(16);
        // : t1 begin ;
        vm.set_source(": t1 begin ;");
        vm.evaluate();
        assert_eq!(vm.last_error(), Some(ControlStructureMismatch));
        vm.reset();
        vm.clear_stacks();
        // : t2 while ;
        vm.set_source(": t2 while ;");
        vm.evaluate();
        assert_eq!(vm.last_error(), Some(ControlStructureMismatch));
        vm.reset();
        vm.clear_stacks();
        // : t3 repeat ;
        vm.set_source(": t3 repeat ;");
        vm.evaluate();
        assert_eq!(vm.last_error(), Some(ControlStructureMismatch));
        vm.reset();
        vm.clear_stacks();
        // : t4 begin while ;
        vm.set_source(": t4 begin while ;");
        vm.evaluate();
        assert_eq!(vm.last_error(), Some(ControlStructureMismatch));
        vm.reset();
        vm.clear_stacks();
        // : t5 begin repeat ;
        vm.set_source(": t5 begin repeat ;");
        vm.evaluate();
        assert_eq!(vm.last_error(), Some(ControlStructureMismatch));
        vm.reset();
        vm.clear_stacks();
        // : t6 while repeat ;
        vm.set_source(": t6 while repeat ;");
        vm.evaluate();
        assert_eq!(vm.last_error(), Some(ControlStructureMismatch));
        vm.reset();
        vm.clear_stacks();
        // : t7 0 begin 1+ dup 3 <> while repeat ; t1
        vm.set_source(": t7 0 begin 1+ dup 3 <> while repeat ; t7");
        vm.evaluate();
        assert_eq!(vm.last_error(), None);
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), Ok(3));
    }

    #[test]
    fn test_backslash() {
        let vm = &mut VM::new(16);
        vm.set_source("1 2 3 \\ 5 6 7");
        vm.evaluate();
        assert!(vm.last_error().is_none());
        assert_eq!(vm.s_stack().len(), 3);
        assert_eq!(vm.s_stack().pop(), Ok(3));
        assert_eq!(vm.s_stack().pop(), Ok(2));
        assert_eq!(vm.s_stack().pop(), Ok(1));
    }

    #[test]
    fn test_marker_unmark() {
        let vm = &mut VM::new(16);
        let symbols_len = vm.symbols().len();
        let wordlist_len = vm.wordlist().len();
        vm.set_source("here marker empty empty here =");
        vm.evaluate();
        assert!(vm.last_error().is_none());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), Ok(-1));
        assert_eq!(vm.symbols().len(), symbols_len);
        assert_eq!(vm.wordlist().len(), wordlist_len);
    }

    #[test]
    fn test_abort() {
        let vm = &mut VM::new(16);
        vm.set_source("1 2 3 abort 5 6 7");
        vm.evaluate();
        assert_eq!(vm.last_error(), Some(Abort));
        assert_eq!(vm.s_stack().len(), 0);
    }

    #[test]
    fn test_do_loop() {
        let vm = &mut VM::new(16);
        // : t1 do ;
        vm.set_source(": t1 do ;");
        vm.evaluate();
        assert_eq!(vm.last_error(), Some(ControlStructureMismatch));
        vm.reset();
        vm.clear_stacks();
        // : t2 loop ;
        vm.set_source(": t2 loop ;");
        vm.evaluate();
        assert_eq!(vm.last_error(), Some(ControlStructureMismatch));
        vm.reset();
        vm.clear_stacks();
        // : main 1 5 0 do 1+ loop ;  main
        vm.set_source(": main 1 5 0 do 1+ loop ;  main");
        vm.evaluate();
        assert!(vm.last_error().is_none());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), Ok(6));
    }

    #[test]
    fn test_do_unloop_exit_loop() {
        let vm = &mut VM::new(16);
        // : t1 unloop ;
        vm.set_source(": t1 unloop ; t1");
        vm.evaluate();
        assert_eq!(vm.last_error(), Some(ReturnStackUnderflow));
        vm.reset();
        vm.clear_stacks();
        // : main 1 5 0 do 1+ dup 3 = if unloop exit then loop ;  main
        vm.set_source(": main 1 5 0 do 1+ dup 3 = if unloop exit then loop ;  main");
        vm.evaluate();
        assert!(vm.last_error().is_none());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), Ok(3));
    }

    #[test]
    fn test_do_plus_loop() {
        let vm = &mut VM::new(16);
        // : t1 +loop ;
        vm.set_source(": t1 +loop ;");
        vm.evaluate();
        assert_eq!(vm.last_error(), Some(ControlStructureMismatch));
        vm.reset();
        vm.clear_stacks();
        // : t2 5 0 do +loop ;
        vm.set_source(": t2 5 0 do +loop ; t2");
        vm.evaluate();
        assert_eq!(vm.last_error(), Some(StackUnderflow));
        vm.clear_stacks();
        vm.reset();
        // : t3 1 5 0 do 1+ 2 +loop ;  main
        vm.set_source(": t3 1 5 0 do 1+ 2 +loop ;  t3");
        vm.evaluate();
        assert_eq!(vm.last_error(), None);
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), Ok(4));
        // : t4 1 6 0 do 1+ 2 +loop ;  t4
        vm.set_source(": t4 1 6 0 do 1+ 2 +loop ;  t4");
        vm.evaluate();
        assert!(vm.last_error().is_none());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), Ok(4));
    }

    #[test]
    fn test_do_leave_loop() {
        let vm = &mut VM::new(16);
        // : t1 leave ;
        vm.set_source(": t1 leave ;  t1");
        vm.evaluate();
        assert_eq!(vm.last_error(), Some(ReturnStackUnderflow));
        vm.reset();
        vm.clear_stacks();
        // : main 1 5 0 do 1+ dup 3 = if drop 88 leave then loop 9 ;  main
        vm.set_source(": main 1 5 0 do 1+ dup 3 = if drop 88 leave then loop 9 ;  main");
        vm.evaluate();
        assert_eq!(vm.last_error(), None);
        assert_eq!(vm.s_stack().len(), 2);
        assert_eq!(vm.s_stack().pop2(), Ok((88, 9)));
    }

    #[test]
    fn test_do_i_loop() {
        let vm = &mut VM::new(16);
        // : main 3 0 do i loop ;  main
        vm.set_source(": main 3 0 do i loop ;  main");
        vm.evaluate();
        assert_eq!(vm.last_error(), None);
        assert_eq!(vm.s_stack().len(), 3);
        assert_eq!(vm.s_stack().pop3(), Ok((0, 1, 2)));
    }

    #[test]
    fn test_do_i_j_loop() {
        let vm = &mut VM::new(16);
        vm.set_source(": main 6 4 do 3 1 do i j * loop loop ;  main");
        vm.evaluate();
        assert_eq!(vm.last_error(), None);
        assert_eq!(vm.s_stack().len(), 4);
        assert_eq!(vm.s_stack().as_slice(), [4, 8, 5, 10]);
    }

    #[bench]
    fn bench_fib(b: &mut Bencher) {
        let vm = &mut VM::new(16);
        vm.set_source(": fib dup 2 < if drop 1 else dup 1- recurse swap 2 - recurse + then ;");
        vm.evaluate();
        assert!(vm.last_error().is_none());
        vm.set_source(": main 7 fib drop ;");
        vm.evaluate();
        vm.set_source("' main");
        vm.evaluate();
        b.iter(|| {
            vm.dup();
            vm.execute();
            vm.run();
            match vm.last_error() {
                Some(_) => assert!(false),
                None => assert!(true),
            };
        });
    }

    #[bench]
    fn bench_repeat(b: &mut Bencher) {
        let vm = &mut VM::new(16);
        vm.set_source(": bench 0 begin over over > while 1 + repeat drop drop ;");
        vm.evaluate();
        vm.set_source(": main 8000 bench ;");
        vm.evaluate();
        vm.set_source("' main");
        vm.evaluate();
        b.iter(|| {
            vm.dup();
            vm.execute();
            vm.run();
            match vm.last_error() {
                Some(_) => assert!(false),
                None => assert!(true),
            };
        });
    }

    #[bench]
    fn bench_sieve(b: &mut Bencher) {
        let vm = &mut VM::new(16);
        vm.load("./lib.fs");
        assert_eq!(vm.last_error(), None);
        vm.set_source("CREATE FLAGS 8190 ALLOT   VARIABLE EFLAG");
        vm.evaluate();
        assert_eq!(vm.last_error(), None);
        vm.set_source("
            : PRIMES  ( -- n )  FLAGS 8190 1 FILL  0 3  EFLAG @ FLAGS
                DO   I C@
                    IF  DUP I + DUP EFLAG @ <
                        IF    EFLAG @ SWAP
                            DO  0 I C! DUP  +LOOP
                        ELSE  DROP  THEN  SWAP 1+ SWAP
                    THEN  2 +
                LOOP  DROP ;
        ");
        vm.evaluate();
        assert_eq!(vm.last_error(), None);
        vm.set_source("
            : BENCHMARK  0 1 0 DO  PRIMES NIP  LOOP ;
        ");
        vm.evaluate();
        assert_eq!(vm.last_error(), None);
        vm.set_source("
            : MAIN
                FLAGS 8190 + EFLAG !
                BENCHMARK DROP
            ;
        ");
        vm.evaluate();
        assert_eq!(vm.last_error(), None);
        vm.set_source("' main");
        vm.evaluate();
        b.iter(|| {
            vm.dup();
            vm.execute();
            vm.run();
            match vm.last_error() {
                Some(_) => assert!(false),
                None => assert!(true),
            };
        });
    }
}
