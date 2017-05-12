extern crate libc;

use {TRUE, FALSE};
use std::process;
use std::mem;
use std::ops::{Index, IndexMut};
use std::fmt::{self, Display};
use std::fmt::Write;
use std::str::FromStr;
use std::ascii::AsciiExt;
use std::result;
use jitmem::{self, DataSpace};
use exception::Exception::{self, Abort, UnexpectedEndOfFile, UndefinedWord, StackOverflow,
                           StackUnderflow, ReturnStackUnderflow, ReturnStackOverflow,
                           FloatingPointStackOverflow, FloatingPointStackUnderflow,
                           UnsupportedOperation, InterpretingACompileOnlyWord,
                           ControlStructureMismatch, InvalidMemoryAddress, DivisionByZero};

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

pub struct Stack<T: Default> {
    pub inner: [T; 256],
    pub len: u8,
    pub canary: T,
}

impl<T: Default + Copy + PartialEq + Display> Stack<T> {
    pub fn new(canary: T) -> Self {
        let mut result = Stack {
            inner: [T::default(); 256],
            len: 0,
            canary: canary,
        };
        result.reset();
        result
    }

    pub fn reset(&mut self) {
        self.len = 0;
        for i in 32..256 {
            self.inner[i] = self.canary;
        }
    }

    pub fn underflow(&self) -> bool {
        (self.inner[255] != self.canary) || (self.len > 128)
    }

    pub fn overflow(&self) -> bool {
        (self.inner[32] != self.canary) || (self.len > 32 && self.len <= 128)
    }

    pub fn push(&mut self, v: T) -> Result {
        let len = self.len.wrapping_add(1);
        self.len = len;
        self.inner[len.wrapping_sub(1) as usize] = v;
        Ok(())
    }

    pub fn pop(&mut self) -> T {
        let result = self.inner[self.len.wrapping_sub(1) as usize];
        self.len = self.len.wrapping_sub(1);
        result
    }

    pub fn push2(&mut self, v1: T, v2: T) -> Result {
        let len = self.len.wrapping_add(2);
        self.len = len;
        self.inner[self.len.wrapping_sub(2) as usize] = v1;
        self.inner[self.len.wrapping_sub(1) as usize] = v2;
        Ok(())
    }

    pub fn push3(&mut self, v1: T, v2: T, v3: T) -> Result {
        let len = self.len.wrapping_add(3);
        self.len = len;
        self.inner[self.len.wrapping_sub(3) as usize] = v1;
        self.inner[self.len.wrapping_sub(2) as usize] = v2;
        self.inner[self.len.wrapping_sub(1) as usize] = v3;
        Ok(())
    }

    pub fn pop2(&mut self) -> (T, T) {
        let result = (self.inner[self.len.wrapping_sub(2) as usize],
                         self.inner[self.len.wrapping_sub(1) as usize]);
        self.len = self.len.wrapping_sub(2);
        result
    }

    pub fn pop3(&mut self) -> (T, T, T) {
        let result = (self.inner[self.len.wrapping_sub(3) as usize],
                         self.inner[self.len.wrapping_sub(2) as usize],
                         self.inner[self.len.wrapping_sub(1) as usize]);
        self.len = self.len.wrapping_sub(3);
        result
    }

    pub fn last(&self) -> Option<T> {
        Some(self.inner[self.len.wrapping_sub(1) as usize])
    }

    pub fn get(&self, pos: u8) -> Option<T> {
        Some(self.inner[pos as usize])
    }

    pub fn len(&self) -> u8 {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// # Safety
    /// Because the implementer (me) is still learning Rust, it is uncertain if as_slice is safe.
    pub fn as_slice(&self) -> &[T] {
        &self.inner[..self.len as usize]
    }
}

impl Index<u8> for Stack<isize> {
    type Output = isize;
    fn index(&self, index: u8) -> &isize {
        &self.inner[index as usize]
    }
}

impl IndexMut<u8> for Stack<isize> {
    fn index_mut(&mut self, index: u8) -> &mut isize {
        &mut self.inner[index as usize]
    }
}

impl Index<u8> for Stack<f64> {
    type Output = f64;
    fn index(&self, index: u8) -> &f64 {
        &self.inner[index as usize]
    }
}

impl IndexMut<u8> for Stack<f64> {
    fn index_mut(&mut self, index: u8) -> &mut f64 {
        &mut self.inner[index as usize]
    }
}

impl fmt::Debug for Stack<isize> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match write!(f, "<{}> ", self.len()) {
            Ok(_) => {
                if self.len == 0 {
                } else {
                    for i in 0..self.len {
                        let v = self[i];
                        match write!(f, "{} ", v) {
                            Ok(_) => {}
                            Err(e) => {
                                return Err(e);
                            }
                        }
                    }

                }
                Ok(())
            }
            Err(e) => Err(e),
        }
    }
}

impl fmt::Debug for Stack<f64> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match write!(f, "<F {}> ", self.len()) {
            Ok(_) => {
                if self.len == 0 {
                } else {
                    for i in 0..self.len {
                        let v = self[i];
                        match write!(f, "{} ", v) {
                            Ok(_) => {}
                            Err(e) => {
                                return Err(e);
                            }
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
    fn s_stack(&mut self) -> &mut Stack<isize>;
    fn r_stack(&mut self) -> &mut Stack<isize>;
    fn c_stack(&mut self) -> &mut Stack<usize>;
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
        self.add_primitive("?stacks", Core::check_stacks); // j1, jx
        self.add_primitive("0<", Core::zero_less); // eForth
        self.add_primitive("=", Core::equals); // j1, jx
        self.add_primitive("<", Core::less_than); // j1, jx
        self.add_primitive("invert", Core::invert); // j1, jx
        self.add_primitive("and", Core::and); // j1, Ngaro, jx, eForth
        self.add_primitive("or", Core::or); // j1, Ngaro, jx, eForth
        self.add_primitive("xor", Core::xor); // j1, Ngaro, jx, eForth
        self.add_primitive("lshift", Core::lshift); // jx, Ngaro
        self.add_primitive("rshift", Core::rshift); // jx
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
        if i < self.wordlist().len() {
            (self.wordlist()[i].action())(self);
        } else {
            self.abort_with(UndefinedWord);
        }
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
        let mut buffer = self.input_buffer().take().expect("input buffer");
        buffer.clear();
        buffer.push_str(s);
        self.state().source_index = 0;
        self.set_input_buffer(buffer);
    }

    /// Run-time: ( "ccc" -- )
    ///
    /// Parse word delimited by white space, skipping leading white spaces.
    fn parse_word(&mut self) {
        let mut last_token = self.last_token().take().expect("token");
        last_token.clear();
        if let Some(input_buffer) = self.input_buffer().take() {
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
            self.set_input_buffer(input_buffer);
        }
        self.set_last_token(last_token);
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
        let last_token = self.last_token().take().expect("token");
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
        let ch = self.s_stack().pop();
        self.compile_integer(ch);
    }

    /// Run-time: ( char "ccc&lt;char&gt;" -- )
    ///
    /// Parse ccc delimited by the delimiter char.
    fn parse(&mut self) {
        let input_buffer = self.input_buffer().take().expect("input buffer");
        let v = self.s_stack().pop();
        let mut last_token = self.last_token().take().expect("token");
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

    fn imm_paren(&mut self) {
        match self.s_stack().push(')' as isize) {
            Err(_) => {
                self.abort_with(StackOverflow);
            }
            Ok(()) => {
                self.parse();
            }
        }
    }

    fn imm_backslash(&mut self) {
        self.state().source_index = match *self.input_buffer() {
            Some(ref buf) => buf.len(),
            None => 0,
        };
    }

    fn compile_token(&mut self) {
        let last_token = self.last_token().take().expect("token");
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
                            write!(buf, "{} ", &last_token).expect("write");
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
        let last_token = self.last_token().take().expect("last token");
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
                            write!(buf, "{} ", &last_token).expect("write");
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
                if self.last_error().is_some() {
                    break;
                }
            } else {
                self.interpret_token();
                if self.last_error().is_some() {
                    break;
                }
            }
            self.run();
            self.check_stacks();
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
        let rlen = self.r_stack().len.wrapping_add(1);
        self.r_stack().len = rlen;
        self.r_stack()[rlen.wrapping_sub(1)] = self.state().instruction_pointer as isize;
        let wp = self.state().word_pointer;
        self.state().instruction_pointer = self.wordlist()[wp].dfa();
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
        let mut last_token = self.last_token().take().expect("last token");
        last_token.make_ascii_lowercase();
        if let Some(_) = self.find(&last_token) {
            match self.output_buffer().as_mut() {
                Some(buf) => {
                    write!(buf, "Redefining {}", last_token).expect("write");
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
            self.right_bracket();
        }
    }

    fn semicolon(&mut self) {
        if self.last_definition() != 0 {
            if self.c_stack().len != 0 {
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
        let v = self.s_stack().pop();
        self.define(Core::p_const);
        if self.last_error().is_none() {
            self.data_space().compile_i32(v as i32);
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
        let v = self.s_stack().pop();
        if v == 0 {
            self.branch();
        } else {
            self.state().instruction_pointer += mem::size_of::<i32>();
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
        let (rn, rt) = self.r_stack().pop2();
        if rt + 1 < rn {
            if let Err(e) = self.r_stack().push2(rn, rt + 1) {
                self.abort_with(e);
                return;
            }
            self.branch();
        } else {
            let _ = self.r_stack().pop();
            self.state().instruction_pointer += mem::size_of::<i32>();
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
        let (rn, rt) = self.r_stack().pop2();
        let t = self.s_stack().pop();
        if rt + t < rn {
            if let Err(e) = self.r_stack().push2(rn, rt + t) {
                self.abort_with(e);
                return;
            }
            self.branch();
        } else {
            let _ = self.r_stack().pop();
            self.state().instruction_pointer += mem::size_of::<i32>();
        }
    }

    /// Run-time: ( -- ) ( R: loop-sys -- )
    ///
    /// Discard the loop-control parameters for the current nesting level. An
    /// `UNLOOP` is required for each nesting level before the definition may be
    /// `EXIT`ed. An ambiguous condition exists if the loop-control parameters
    /// are unavailable.
    fn unloop(&mut self) {
        let _ = self.r_stack().pop3();
    }

    fn leave(&mut self) {
        let (third, _, _) = self.r_stack().pop3();
        if self.r_stack().underflow() {
            self.abort_with(ReturnStackUnderflow);
            return;
        }
        self.state().instruction_pointer = self.data_space().get_i32(third as usize) as usize;
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
        let here = self.data_space().len();
        self.c_stack().push(here).expect("pushed");
    }

    fn imm_else(&mut self) {
        let if_part = self.c_stack().pop();
        if self.c_stack().underflow() {
            self.abort_with(ControlStructureMismatch);
        } else {
            let idx = self.references().idx_branch as i32;
            self.data_space().compile_i32(idx);
            self.data_space().compile_i32(0);
            let here = self.data_space().len();
            self.c_stack().push(here).expect("pushed");
            self.data_space()
                .put_i32(here as i32, (if_part - mem::size_of::<i32>()));
        }
    }

    fn imm_then(&mut self) {
        let branch_part = self.c_stack().pop();
        if self.c_stack().underflow() {
            self.abort_with(ControlStructureMismatch);
        } else {
            let here = self.data_space().len();
            self.data_space()
                .put_i32(here as i32, (branch_part - mem::size_of::<i32>()));
        }
    }

    fn imm_begin(&mut self) {
        let here = self.data_space().len();
        self.c_stack().push(here).expect("pushed");
    }

    fn imm_while(&mut self) {
        let idx = self.references().idx_zero_branch as i32;
        self.data_space().compile_i32(idx);
        self.data_space().compile_i32(0);
        let here = self.data_space().len();
        self.c_stack().push(here).expect("pushed");
    }

    fn imm_repeat(&mut self) {
        let (begin_part, while_part) = self.c_stack().pop2();
        if self.c_stack().underflow() {
            self.abort_with(ControlStructureMismatch);
        } else {
            let idx = self.references().idx_branch as i32;
            self.data_space().compile_i32(idx);
            self.data_space().compile_i32(begin_part as i32);
            let here = self.data_space().len();
            self.data_space()
                .put_i32(here as i32, (while_part - mem::size_of::<i32>()));
        }
    }

    fn imm_again(&mut self) {
        let begin_part = self.c_stack().pop();
        if self.c_stack().underflow() {
            self.abort_with(ControlStructureMismatch);
        } else {
            let idx = self.references().idx_branch as i32;
            self.data_space().compile_i32(idx);
            self.data_space().compile_i32(begin_part as i32);
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
        let here = self.data_space().len();
        self.c_stack().push(here).expect("pushed");
    }

    /// Run-time: ( a-addr -- )
    ///
    /// Append the run-time semantics of `_LOOP` to the current definition.
    /// Resolve the destination of all unresolved occurrences of `LEAVE` between
    /// the location given by do-sys and the next location for a transfer of
    /// control, to execute the words following the `LOOP`.
    fn imm_loop(&mut self) {
        let do_part = self.c_stack().pop();
        if self.c_stack().underflow() {
            self.abort_with(ControlStructureMismatch);
        } else {
            let idx = self.references().idx_loop as i32;
            self.data_space().compile_i32(idx);
            self.data_space().compile_i32(do_part as i32);
            let here = self.data_space().len();
            self.data_space()
                .put_i32(here as i32, (do_part - mem::size_of::<i32>()) as usize);
        }
    }

    /// Run-time: ( a-addr -- )
    ///
    /// Append the run-time semantics of `_+LOOP` to the current definition.
    /// Resolve the destination of all unresolved occurrences of `LEAVE` between
    /// the location given by do-sys and the next location for a transfer of
    /// control, to execute the words following `+LOOP`.
    fn imm_plus_loop(&mut self) {
        let do_part = self.c_stack().pop();
        if self.c_stack().underflow() {
            self.abort_with(ControlStructureMismatch);
        } else {
            let idx = self.references().idx_plus_loop as i32;
            self.data_space().compile_i32(idx);
            self.data_space().compile_i32(do_part as i32);
            let here = self.data_space().len();
            self.data_space()
                .put_i32(here as i32, (do_part - mem::size_of::<i32>()) as usize);
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
        let v = self.s_stack().pop();
        self.push(v + mem::size_of::<u8>() as isize);
    }

    /// Run-time: (n1 -- n2 )
    ///
    /// `n2` is the size in address units of `n1` characters.
    fn chars(&mut self) {
        let v = self.s_stack().pop();
        self.push(v * mem::size_of::<u8>() as isize);
    }


    /// Run-time: (a-addr1 -- a-addr2 )
    ///
    /// Add the size in address units of a cell to `a-addr1`, giving `a-addr2`.
    fn cell_plus(&mut self) {
        let v = self.s_stack().pop();
        self.push(v + mem::size_of::<i32>() as isize);
    }

    /// Run-time: (n1 -- n2 )
    ///
    /// `n2` is the size in address units of `n1` cells.
    fn cells(&mut self) {
        let v = self.s_stack().pop();
        self.push(v * mem::size_of::<i32>() as isize);
    }

    fn lit(&mut self) {
        let ip = self.state().instruction_pointer;
        let v = self.data_space().get_i32(ip) as isize;
        let slen = self.s_stack().len.wrapping_add(1);
        self.s_stack().len = slen;
        self.s_stack()[slen.wrapping_sub(1)] = v;
        self.state().instruction_pointer = self.state().instruction_pointer + mem::size_of::<i32>();
    }

    fn flit(&mut self) {
        let ip = self.state().instruction_pointer as usize;
        let v = self.data_space().get_f64(ip);
        let flen = self.f_stack().len.wrapping_add(1);
        self.f_stack().len = flen;
        self.f_stack()[flen.wrapping_sub(1)] = v;
        self.state().instruction_pointer = self.state().instruction_pointer + mem::size_of::<f64>();
    }

    /// Runtime of S"
    fn p_s_quote(&mut self) {
        let ip = self.state().instruction_pointer;
        let cnt = self.data_space().get_i32(ip);
        let addr = self.state().instruction_pointer + mem::size_of::<i32>();
        let slen = self.s_stack().len.wrapping_add(2);
        self.s_stack().len = slen;
        self.s_stack()[slen.wrapping_sub(1)] = cnt as isize;
        self.s_stack()[slen.wrapping_sub(2)] = addr as isize;
        self.state().instruction_pointer =
            self.state().instruction_pointer + mem::size_of::<i32>() + cnt as usize;
    }

    fn swap(&mut self) {
        let slen = self.s_stack().len;
        let t = self.s_stack()[slen.wrapping_sub(1)];
        let n = self.s_stack()[slen.wrapping_sub(2)];
        self.s_stack()[slen.wrapping_sub(1)] = n;
        self.s_stack()[slen.wrapping_sub(2)] = t;
    }

    fn dup(&mut self) {
        let slen = self.s_stack().len.wrapping_add(1);
        self.s_stack().len = slen;
        self.s_stack()[slen.wrapping_sub(1)] = self.s_stack()[slen.wrapping_sub(2)];
    }

    fn p_drop(&mut self) {
        let slen = self.s_stack().len.wrapping_sub(1);
        self.s_stack().len = slen;
    }

    fn nip(&mut self) {
        let slen = self.s_stack().len.wrapping_sub(1);
        let t = self.s_stack()[slen];
        self.s_stack().len = slen;
        self.s_stack()[slen.wrapping_sub(1)] = t;
    }

    fn over(&mut self) {
        let slen = self.s_stack().len.wrapping_add(1);
        self.s_stack().len = slen;
        self.s_stack()[slen.wrapping_sub(1)] = self.s_stack()[slen.wrapping_sub(3)];
    }

    fn rot(&mut self) {
        let slen = self.s_stack().len;
        let t = self.s_stack()[slen.wrapping_sub(1)];
        let n = self.s_stack()[slen.wrapping_sub(2)];
        self.s_stack()[slen.wrapping_sub(1)] = self.s_stack()[slen.wrapping_sub(3)];
        self.s_stack()[slen.wrapping_sub(2)] = t;
        self.s_stack()[slen.wrapping_sub(3)] = n;
    }

    fn two_drop(&mut self) {
        let slen = self.s_stack().len.wrapping_sub(2);
        self.s_stack().len = slen;
    }

    fn two_dup(&mut self) {
        let slen = self.s_stack().len.wrapping_add(2);
        self.s_stack().len = slen;
        self.s_stack()[slen.wrapping_sub(1)] = self.s_stack()[slen.wrapping_sub(3)];
        self.s_stack()[slen.wrapping_sub(2)] = self.s_stack()[slen.wrapping_sub(4)];
    }

    fn two_swap(&mut self) {
        let slen = self.s_stack().len;
        let t = self.s_stack()[slen.wrapping_sub(1)];
        let n = self.s_stack()[slen.wrapping_sub(2)];
        self.s_stack()[slen.wrapping_sub(1)] = self.s_stack()[slen.wrapping_sub(3)];
        self.s_stack()[slen.wrapping_sub(2)] = self.s_stack()[slen.wrapping_sub(4)];
        self.s_stack()[slen.wrapping_sub(3)] = t;
        self.s_stack()[slen.wrapping_sub(4)] = n;
    }

    fn two_over(&mut self) {
        let slen = self.s_stack().len.wrapping_add(2);
        self.s_stack().len = slen;
        self.s_stack()[slen.wrapping_sub(1)] = self.s_stack()[slen.wrapping_sub(5)];
        self.s_stack()[slen.wrapping_sub(2)] = self.s_stack()[slen.wrapping_sub(6)];
    }

    fn depth(&mut self) {
        let len = self.s_stack().len;
        self.push(len as isize);
    }

    fn one_plus(&mut self) {
        let slen = self.s_stack().len;
        let t = self.s_stack()[slen.wrapping_sub(1)];
        self.s_stack()[slen.wrapping_sub(1)] = t.wrapping_add(1);
    }

    fn one_minus(&mut self) {
        let slen = self.s_stack().len;
        let t = self.s_stack()[slen.wrapping_sub(1)];
        self.s_stack()[slen.wrapping_sub(1)] = t.wrapping_sub(1);
    }

    fn plus(&mut self) {
        let slen = self.s_stack().len;
        let t = self.s_stack()[slen.wrapping_sub(1)];
        let n = self.s_stack()[slen.wrapping_sub(2)];
        self.s_stack()[slen.wrapping_sub(2)] = n.wrapping_add(t);
        self.s_stack().len = slen.wrapping_sub(1);
    }

    fn minus(&mut self) {
        let slen = self.s_stack().len;
        let t = self.s_stack()[slen.wrapping_sub(1)];
        let n = self.s_stack()[slen.wrapping_sub(2)];
        self.s_stack()[slen.wrapping_sub(2)] = n.wrapping_sub(t);
        self.s_stack().len = slen.wrapping_sub(1);
    }

    fn star(&mut self) {
        let slen = self.s_stack().len;
        let t = self.s_stack()[slen.wrapping_sub(1)];
        let n = self.s_stack()[slen.wrapping_sub(2)];
        self.s_stack()[slen.wrapping_sub(2)] = n.wrapping_mul(t);
        self.s_stack().len = slen.wrapping_sub(1);
    }

    fn slash(&mut self) {
        let slen = self.s_stack().len;
        let t = self.s_stack()[slen.wrapping_sub(1)];
        let n = self.s_stack()[slen.wrapping_sub(2)];
        if t == 0 {
            self.abort_with(DivisionByZero);
        } else {
            self.s_stack()[slen.wrapping_sub(2)] = n.wrapping_div(t);
            self.s_stack().len = slen.wrapping_sub(1);
        }
    }

    fn p_mod(&mut self) {
        let slen = self.s_stack().len;
        let t = self.s_stack()[slen.wrapping_sub(1)];
        let n = self.s_stack()[slen.wrapping_sub(2)];
        if t == 0 {
            self.abort_with(DivisionByZero);
        } else {
            self.s_stack()[slen.wrapping_sub(2)] = n.wrapping_rem(t);
            self.s_stack().len = slen.wrapping_sub(1);
        }
    }

    fn slash_mod(&mut self) {
        let slen = self.s_stack().len;
        let t = self.s_stack()[slen.wrapping_sub(1)];
        let n = self.s_stack()[slen.wrapping_sub(2)];
        if t == 0 {
            self.abort_with(DivisionByZero);
        } else {
            self.s_stack()[slen.wrapping_sub(2)] = n.wrapping_rem(t);
            self.s_stack()[slen.wrapping_sub(1)] = n.wrapping_div(t);
        }
    }

    fn abs(&mut self) {
        let t = self.s_stack().pop();
        self.push(t.wrapping_abs());
    }

    fn negate(&mut self) {
        let t = self.s_stack().pop();
        self.push(t.wrapping_neg());
    }

    fn zero_less(&mut self) {
        let t = self.s_stack().pop();
        self.push(if t < 0 { TRUE } else { FALSE });
    }

    fn zero_equals(&mut self) {
        let t = self.s_stack().pop();
        self.push(if t == 0 { TRUE } else { FALSE });
    }

    fn zero_greater(&mut self) {
        let t = self.s_stack().pop();
        self.push(if t > 0 { TRUE } else { FALSE });
    }

    fn zero_not_equals(&mut self) {
        let t = self.s_stack().pop();
        self.push(if t == 0 { FALSE } else { TRUE });
    }

    fn equals(&mut self) {
        let (n, t) = self.s_stack().pop2();
        self.push(if t == n { TRUE } else { FALSE });
    }

    fn less_than(&mut self) {
        let (n, t) = self.s_stack().pop2();
        self.push(if n < t { TRUE } else { FALSE });
    }

    fn greater_than(&mut self) {
        let (n, t) = self.s_stack().pop2();
        self.push(if n > t { TRUE } else { FALSE });
    }

    fn not_equals(&mut self) {
        let (n, t) = self.s_stack().pop2();
        self.push(if n == t { FALSE } else { TRUE });
    }

    fn between(&mut self) {
        let (x1, x2, x3) = self.s_stack().pop3();
        self.push(if x2 <= x1 && x1 <= x3 { TRUE } else { FALSE });
    }

    fn invert(&mut self) {
        let t = self.s_stack().pop();
        self.push(!t);
    }

    fn and(&mut self) {
        let (n, t) = self.s_stack().pop2();
        self.push(t & n);
    }

    fn or(&mut self) {
        let (n, t) = self.s_stack().pop2();
        self.push(t | n);
    }

    fn xor(&mut self) {
        let (n, t) = self.s_stack().pop2();
        self.push(t ^ n);
    }

    /// Run-time: ( x1 u -- x2 )
    ///
    /// Perform a logical left shift of `u` bit-places on `x1`, giving `x2`. Put
    /// zeroes into the least significant bits vacated by the shift. An
    /// ambiguous condition exists if `u` is greater than or equal to the number
    /// of bits in a cell.
    fn lshift(&mut self) {
        let (n, t) = self.s_stack().pop2();
        self.push(n.wrapping_shl(t as u32));
    }

    /// Run-time: ( x1 u -- x2 )
    ///
    /// Perform a logical right shift of `u` bit-places on `x1`, giving `x2`. Put
    /// zeroes into the most significant bits vacated by the shift. An
    /// ambiguous condition exists if `u` is greater than or equal to the number
    /// of bits in a cell.
    fn rshift(&mut self) {
        let (n, t) = self.s_stack().pop2();
        self.push(((n as usize).wrapping_shr(t as u32)) as isize);
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
        let rlen = self.r_stack().len.wrapping_sub(1);
        self.state().instruction_pointer = self.r_stack()[rlen] as usize;
        self.r_stack().len = rlen;
    }

    /// Run-time: ( a-addr -- x )
    ///
    /// `x` is the value stored at `a-addr`.
    fn fetch(&mut self) {
        let t = self.s_stack().pop();
        if (t as usize + mem::size_of::<i32>()) <= self.data_space().capacity() {
            let value = self.data_space().get_i32(t as usize) as isize;
            self.push(value);
        } else {
            self.abort_with(InvalidMemoryAddress);
        }
    }

    /// Run-time: ( x a-addr -- )
    ///
    /// Store `x` at `a-addr`.
    fn store(&mut self) {
        let (n, t) = self.s_stack().pop2();
        if (t as usize + mem::size_of::<i32>()) <= self.data_space().capacity() {
            self.data_space().put_i32(n as i32, t as usize);
        } else {
            self.abort_with(InvalidMemoryAddress);
        }
    }

    /// Run-time: ( c-addr -- char )
    ///
    /// Fetch the character stored at `c-addr`. When the cell size is greater than
    /// character size, the unused high-order bits are all zeroes.
    fn c_fetch(&mut self) {
        let t = self.s_stack().pop();
        if (t as usize + mem::size_of::<u8>()) <= self.data_space().capacity() {
            let value = self.data_space().get_u8(t as usize) as isize;
            self.push(value);
        } else {
            self.abort_with(InvalidMemoryAddress);
        }
    }

    /// Run-time: ( char c-addr -- )
    ///
    /// Store `char` at `c-addr`. When character size is smaller than cell size,
    /// only the number of low-order bits corresponding to character size are
    /// transferred.
    fn c_store(&mut self) {
        let (n, t) = self.s_stack().pop2();
        if (t as usize + mem::size_of::<u8>()) <= self.data_space().capacity() {
            self.data_space().put_u8(n as u8, t as usize);
        } else {
            self.abort_with(InvalidMemoryAddress);
        }
    }

    /// Run-time: ( "<spaces>name" -- xt )
    ///
    /// Skip leading space delimiters. Parse name delimited by a space. Find
    /// `name` and return `xt`, the execution token for name. An ambiguous
    /// condition exists if name is not found.
    fn tick(&mut self) {
        self.parse_word();
        let last_token = self.last_token().take().expect("last token");
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
        let t = self.s_stack().pop();
        self.execute_word(t as usize);
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
        let v = self.s_stack().pop();
        self.data_space().allot(v);
    }

    /// Run-time: ( x -- )
    ///
    /// Reserve one cell of data space and store `x` in the cell. If the
    /// data-space pointer is aligned when `,` begins execution, it will remain
    /// aligned when `,` finishes execution. An ambiguous condition exists if the
    /// data-space pointer is not aligned prior to execution of `,`.
    fn comma(&mut self) {
        let v = self.s_stack().pop();
        self.data_space().compile_i32(v as i32);
    }

    fn p_to_r(&mut self) {
        let slen = self.s_stack().len;
        let rlen = self.r_stack().len.wrapping_add(1);
        self.r_stack().len = rlen;
        self.r_stack()[rlen.wrapping_sub(1)] = self.s_stack()[slen.wrapping_sub(1)];
        self.s_stack().len = slen.wrapping_sub(1);
    }

    fn r_from(&mut self) {
        let slen = self.s_stack().len.wrapping_add(1);
        let rlen = self.r_stack().len;
        self.s_stack().len = slen;
        self.s_stack()[slen.wrapping_sub(1)] = self.r_stack()[rlen.wrapping_sub(1)];
        self.r_stack().len = rlen.wrapping_sub(1);
    }

    fn r_fetch(&mut self) {
        let slen = self.s_stack().len.wrapping_add(1);
        let rlen = self.r_stack().len;
        self.s_stack().len = slen;
        self.s_stack()[slen.wrapping_sub(1)] = self.r_stack()[rlen.wrapping_sub(1)];
    }

    fn two_to_r(&mut self) {
        let slen = self.s_stack().len;
        let rlen = self.r_stack().len.wrapping_add(2);
        self.r_stack().len = rlen;
        self.r_stack()[rlen.wrapping_sub(2)] = self.s_stack()[slen.wrapping_sub(2)];
        self.r_stack()[rlen.wrapping_sub(1)] = self.s_stack()[slen.wrapping_sub(1)];
        self.s_stack().len = slen.wrapping_sub(2);
    }

    fn two_r_from(&mut self) {
        let slen = self.s_stack().len.wrapping_add(2);
        let rlen = self.r_stack().len;
        self.s_stack().len = slen;
        self.s_stack()[slen.wrapping_sub(2)] = self.r_stack()[rlen.wrapping_sub(2)];
        self.s_stack()[slen.wrapping_sub(1)] = self.r_stack()[rlen.wrapping_sub(1)];
        self.r_stack().len = rlen.wrapping_sub(2);
    }

    fn two_r_fetch(&mut self) {
        let slen = self.s_stack().len.wrapping_add(2);
        let rlen = self.r_stack().len;
        self.s_stack().len = slen;
        self.s_stack()[slen.wrapping_sub(2)] = self.r_stack()[rlen.wrapping_sub(2)];
        self.s_stack()[slen.wrapping_sub(1)] = self.r_stack()[rlen.wrapping_sub(1)];
    }

    // ----------------
    // Error handlling
    // ----------------

    fn check_stacks(&mut self) {
        if self.s_stack().overflow() {
            self.abort_with(StackOverflow);
        } else if self.s_stack().underflow() {
            self.abort_with(StackUnderflow);
        } else if self.r_stack().overflow() {
            self.abort_with(ReturnStackOverflow);
        } else if self.r_stack().underflow() {
            self.abort_with(ReturnStackUnderflow);
        } else if self.c_stack().overflow() {
            self.abort_with(ControlStructureMismatch);
        } else if self.c_stack().underflow() {
            self.abort_with(ControlStructureMismatch);
        } else if self.f_stack().overflow() {
            self.abort_with(FloatingPointStackOverflow);
        } else if self.f_stack().underflow() {
            self.abort_with(FloatingPointStackUnderflow);
        }
    }

    fn handler_store(&mut self) {
        let t = self.s_stack().pop();
        self.set_handler(t as usize);
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
                        write!(buf, "{} ", e.description()).expect("write");
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
        self.s_stack().reset();
        self.f_stack().reset();
    }

    /// Reset VM, do not clear data stack and floating point stack.
    /// Called by VM's client upon Quit.
    fn reset(&mut self) {
        self.r_stack().len = 0;
        self.c_stack().len = 0;
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
                               UnexpectedEndOfFile, ControlStructureMismatch,
                               ReturnStackUnderflow, InvalidMemoryAddress};
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
        vm.check_stacks();
        assert_eq!(vm.last_error(), Some(StackUnderflow));
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(1).expect("pushed");
        vm.p_drop();
        vm.check_stacks();
        assert!(vm.s_stack().is_empty());
        assert!(vm.last_error().is_none());
    }

    #[bench]
    fn bench_drop(b: &mut Bencher) {
        let vm = &mut VM::new(16);
        vm.s_stack().push(1).expect("pushed");
        b.iter(|| {
                   vm.p_drop();
                   vm.s_stack().push(1).expect("pushed");
               });
    }

    #[test]
    fn test_nip() {
        let vm = &mut VM::new(16);
        vm.nip();
        vm.check_stacks();
        assert_eq!(vm.last_error(), Some(StackUnderflow));
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(1).expect("pushed");
        vm.nip();
        vm.check_stacks();
        assert_eq!(vm.last_error(), Some(StackUnderflow));
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(1).expect("pushed");
        vm.s_stack().push(2).expect("pushed");
        vm.nip();
        vm.check_stacks();
        assert!(vm.last_error().is_none());
        assert!(vm.s_stack().len() == 1);
        assert!(vm.s_stack().last() == Some(2));
    }

    #[bench]
    fn bench_nip(b: &mut Bencher) {
        let vm = &mut VM::new(16);
        vm.s_stack().push(1).expect("pushed");
        vm.s_stack().push(1).expect("pushed");
        b.iter(|| {
                   vm.nip();
                   vm.s_stack().push(1).expect("pushed");
               });
    }

    #[test]
    fn test_swap() {
        let vm = &mut VM::new(16);
        vm.swap();
        vm.check_stacks();
        // check_stacks() cannot detect this kind of underflow.
        // assert_eq!(vm.last_error(), Some(StackUnderflow));
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(1).expect("pushed");
        vm.swap();
        vm.check_stacks();
        assert_eq!(vm.last_error(), Some(StackUnderflow));
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(1).expect("pushed");
        vm.s_stack().push(2).expect("pushed");
        vm.swap();
        vm.check_stacks();
        assert!(vm.last_error().is_none());
        assert_eq!(vm.s_stack().len(), 2);
        assert_eq!(vm.s_stack().pop(), 1);
        assert_eq!(vm.s_stack().pop(), 2);
        vm.check_stacks();
        assert!(vm.last_error().is_none());
    }

    #[bench]
    fn bench_swap(b: &mut Bencher) {
        let vm = &mut VM::new(16);
        vm.s_stack().push(1).expect("pushed");
        vm.s_stack().push(2).expect("pushed");
        b.iter(|| vm.swap());
    }

    #[test]
    fn test_dup() {
        let vm = &mut VM::new(16);
        vm.dup();
        vm.check_stacks();
        // check_stacks can not detect this underflow();
        //        assert_eq!(vm.last_error(), Some(StackUnderflow));
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(1).expect("pushed");
        vm.dup();
        vm.check_stacks();
        assert!(vm.last_error().is_none());
        assert_eq!(vm.s_stack().len(), 2);
        assert_eq!(vm.s_stack().pop(), 1);
        assert_eq!(vm.s_stack().pop(), 1);
        vm.check_stacks();
        assert!(vm.last_error().is_none());
    }

    #[bench]
    fn bench_dup(b: &mut Bencher) {
        let vm = &mut VM::new(16);
        vm.s_stack().push(1).expect("pushed");
        b.iter(|| {
                   vm.dup();
                   vm.s_stack().pop();
               });
    }

    #[test]
    fn test_over() {
        let vm = &mut VM::new(16);
        vm.over();
        vm.check_stacks();
        // check_stacks() cannot detect stack underflow of over().
        // assert_eq!(vm.last_error(), Some(StackUnderflow));
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(1).expect("pushed");
        vm.check_stacks();
        vm.over();
        // check_stacks() cannot detect stack underflow of over().
        // assert_eq!(vm.last_error(), Some(StackUnderflow));
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(1).expect("pushed");
        vm.s_stack().push(2).expect("pushed");
        vm.over();
        vm.check_stacks();
        assert!(vm.last_error().is_none());
        assert_eq!(vm.s_stack().len(), 3);
        assert_eq!(vm.s_stack().pop(), 1);
        assert_eq!(vm.s_stack().pop(), 2);
        assert_eq!(vm.s_stack().pop(), 1);
        vm.check_stacks();
        assert!(vm.last_error().is_none());
    }

    #[bench]
    fn bench_over(b: &mut Bencher) {
        let vm = &mut VM::new(16);
        vm.s_stack().push(1).expect("pushed");
        vm.s_stack().push(2).expect("pushed");
        b.iter(|| {
                   vm.over();
                   vm.s_stack().pop();
               });
    }

    #[test]
    fn test_rot() {
        let vm = &mut VM::new(16);
        vm.rot();
        vm.check_stacks();
        // check_stacks() cannot detect this kind of stack underflow of over().
        // assert_eq!(vm.last_error(), Some(StackUnderflow));
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(1).expect("pushed");
        vm.rot();
        vm.check_stacks();
        assert_eq!(vm.last_error(), Some(StackUnderflow));
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(1).expect("pushed");
        vm.s_stack().push(2).unwrap();
        vm.rot();
        vm.check_stacks();
        assert_eq!(vm.last_error(), Some(StackUnderflow));
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(1).unwrap();
        vm.s_stack().push(2).unwrap();
        vm.s_stack().push(3).unwrap();
        vm.rot();
        vm.check_stacks();
        assert!(vm.last_error().is_none());
        assert_eq!(vm.s_stack().len(), 3);
        assert_eq!(vm.s_stack().pop(), 1);
        assert_eq!(vm.s_stack().pop(), 3);
        assert_eq!(vm.s_stack().pop(), 2);
        vm.check_stacks();
        assert!(vm.last_error().is_none());
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
        assert!(vm.s_stack().underflow());
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(1).unwrap();
        vm.two_drop();
        assert!(vm.s_stack().underflow());
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(1).unwrap();
        vm.s_stack().push(2).unwrap();
        vm.two_drop();
        assert!(!vm.s_stack().underflow());
        assert!(!vm.s_stack().overflow());
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
        assert!(!vm.s_stack().underflow());
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(1).unwrap();
        vm.two_dup();
        assert!(!vm.s_stack().underflow());
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(1).unwrap();
        vm.s_stack().push(2).unwrap();
        vm.two_dup();
        assert!(!vm.s_stack().underflow());
        assert!(!vm.s_stack().overflow());
        assert!(vm.last_error().is_none());
        assert_eq!(vm.s_stack().len(), 4);
        assert_eq!(vm.s_stack().pop(), 2);
        assert_eq!(vm.s_stack().pop(), 1);
        assert_eq!(vm.s_stack().pop(), 2);
        assert_eq!(vm.s_stack().pop(), 1);
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
        assert!(!vm.s_stack().underflow());
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(1).unwrap();
        vm.two_swap();
        assert!(!vm.s_stack().underflow());
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(1).unwrap();
        vm.s_stack().push(2).unwrap();
        vm.two_swap();
        assert!(vm.s_stack().underflow());
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(1).unwrap();
        vm.s_stack().push(2).unwrap();
        vm.s_stack().push(3).unwrap();
        vm.two_swap();
        assert!(vm.s_stack().underflow());
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(1).unwrap();
        vm.s_stack().push(2).unwrap();
        vm.s_stack().push(3).unwrap();
        vm.s_stack().push(4).unwrap();
        vm.two_swap();
        assert!(!vm.s_stack().underflow());
        assert!(vm.last_error().is_none());
        assert_eq!(vm.s_stack().len(), 4);
        assert_eq!(vm.s_stack().pop(), 2);
        assert_eq!(vm.s_stack().pop(), 1);
        assert_eq!(vm.s_stack().pop(), 4);
        assert_eq!(vm.s_stack().pop(), 3);
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
        assert!(!vm.s_stack().underflow());
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(1).unwrap();
        vm.two_over();
        assert!(!vm.s_stack().underflow());
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(1).unwrap();
        vm.s_stack().push(2).unwrap();
        vm.two_over();
        assert!(!vm.s_stack().underflow());
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(1).unwrap();
        vm.s_stack().push(2).unwrap();
        vm.s_stack().push(3).unwrap();
        vm.two_over();
        assert!(!vm.s_stack().underflow());
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(1).unwrap();
        vm.s_stack().push(2).unwrap();
        vm.s_stack().push(3).unwrap();
        vm.s_stack().push(4).unwrap();
        vm.two_over();
        assert!(!vm.s_stack().underflow());
        assert!(!vm.s_stack().overflow());
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
        vm.check_stacks();
        assert_eq!(vm.last_error(), Some(StackUnderflow));
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(1).unwrap();
        vm.one_plus();
        vm.check_stacks();
        assert!(vm.last_error().is_none());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), 2);
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
        vm.check_stacks();
        assert_eq!(vm.last_error(), Some(StackUnderflow));
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(2).unwrap();
        vm.one_minus();
        vm.check_stacks();
        assert!(vm.last_error().is_none());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), 1);
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
        vm.check_stacks();
        assert_eq!(vm.last_error(), Some(StackUnderflow));
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(5).unwrap();
        vm.minus();
        vm.check_stacks();
        assert_eq!(vm.last_error(), Some(StackUnderflow));
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(5).unwrap();
        vm.s_stack().push(7).unwrap();
        vm.minus();
        vm.check_stacks();
        assert_eq!(vm.last_error(), None);
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), -2);
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
        vm.check_stacks();
        assert_eq!(vm.last_error(), Some(StackUnderflow));
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(5).unwrap();
        vm.plus();
        vm.check_stacks();
        assert_eq!(vm.last_error(), Some(StackUnderflow));
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(5).unwrap();
        vm.s_stack().push(7).unwrap();
        vm.plus();
        vm.check_stacks();
        assert!(vm.last_error().is_none());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), 12);
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
        vm.check_stacks();
        assert_eq!(vm.last_error(), Some(StackUnderflow));
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(5).unwrap();
        vm.star();
        vm.check_stacks();
        assert_eq!(vm.last_error(), Some(StackUnderflow));
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(5).unwrap();
        vm.s_stack().push(7).unwrap();
        vm.star();
        vm.check_stacks();
        assert!(vm.last_error().is_none());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), 35);
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
        vm.check_stacks();
        assert_eq!(vm.last_error(), Some(StackUnderflow));
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(30).unwrap();
        vm.slash();
        vm.check_stacks();
        assert_eq!(vm.last_error(), Some(StackUnderflow));
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(30).unwrap();
        vm.s_stack().push(7).unwrap();
        vm.slash();
        vm.check_stacks();
        assert!(vm.last_error().is_none());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), 4);
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
        vm.check_stacks();
        assert_eq!(vm.last_error(), Some(StackUnderflow));
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(30).unwrap();
        vm.p_mod();
        vm.check_stacks();
        assert_eq!(vm.last_error(), Some(StackUnderflow));
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(30).unwrap();
        vm.s_stack().push(7).unwrap();
        vm.p_mod();
        vm.check_stacks();
        assert!(vm.last_error().is_none());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), 2);
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
        vm.check_stacks();
        assert_eq!(vm.last_error(), Some(StackUnderflow));
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(30).unwrap();
        vm.slash_mod();
        vm.check_stacks();
        assert_eq!(vm.last_error(), Some(StackUnderflow));
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(30).unwrap();
        vm.s_stack().push(7).unwrap();
        vm.slash_mod();
        vm.check_stacks();
        assert!(vm.last_error().is_none());
        assert_eq!(vm.s_stack().len(), 2);
        assert_eq!(vm.s_stack().pop(), 4);
        assert_eq!(vm.s_stack().pop(), 2);
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
        vm.s_stack().push(-30).unwrap();
        vm.abs();
        assert!(vm.last_error().is_none());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), 30);
    }

    #[test]
    fn test_negate() {
        let vm = &mut VM::new(16);
        vm.negate();
        vm.check_stacks();
        assert_eq!(vm.last_error(), Some(StackUnderflow));
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(30).unwrap();
        vm.negate();
        vm.check_stacks();
        assert!(vm.last_error().is_none());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), -30);
    }

    #[test]
    fn test_zero_less() {
        let vm = &mut VM::new(16);
        vm.zero_less();
        vm.check_stacks();
        assert_eq!(vm.last_error(), Some(StackUnderflow));
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(-1).unwrap();
        vm.zero_less();
        vm.check_stacks();
        assert!(vm.last_error().is_none());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), -1);
        vm.s_stack().push(0).unwrap();
        vm.zero_less();
        vm.check_stacks();
        assert!(vm.last_error().is_none());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), 0);
    }

    #[test]
    fn test_zero_equals() {
        let vm = &mut VM::new(16);
        vm.zero_equals();
        vm.check_stacks();
        assert_eq!(vm.last_error(), Some(StackUnderflow));
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(0).unwrap();
        vm.zero_equals();
        vm.check_stacks();
        assert!(vm.last_error().is_none());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), -1);
        vm.s_stack().push(-1).unwrap();
        vm.zero_equals();
        vm.check_stacks();
        assert!(vm.last_error().is_none());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), 0);
        vm.s_stack().push(1).unwrap();
        vm.zero_equals();
        vm.check_stacks();
        assert!(vm.last_error().is_none());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), 0);
    }

    #[test]
    fn test_zero_greater() {
        let vm = &mut VM::new(16);
        vm.zero_greater();
        vm.check_stacks();
        assert_eq!(vm.last_error(), Some(StackUnderflow));
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(1).unwrap();
        vm.zero_greater();
        vm.check_stacks();
        assert!(vm.last_error().is_none());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), -1);
        vm.s_stack().push(0).unwrap();
        vm.zero_greater();
        vm.check_stacks();
        assert!(vm.last_error().is_none());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), 0);
    }

    #[test]
    fn test_zero_not_equals() {
        let vm = &mut VM::new(16);
        vm.zero_not_equals();
        vm.check_stacks();
        assert_eq!(vm.last_error(), Some(StackUnderflow));
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(0).unwrap();
        vm.zero_not_equals();
        vm.check_stacks();
        assert!(vm.last_error().is_none());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), 0);
        vm.s_stack().push(-1).unwrap();
        vm.zero_not_equals();
        vm.check_stacks();
        assert!(vm.last_error().is_none());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), -1);
        vm.s_stack().push(1).unwrap();
        vm.zero_not_equals();
        vm.check_stacks();
        assert!(vm.last_error().is_none());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), -1);
    }

    #[test]
    fn test_less_than() {
        let vm = &mut VM::new(16);
        vm.less_than();
        vm.check_stacks();
        assert_eq!(vm.last_error(), Some(StackUnderflow));
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(-1).unwrap();
        vm.less_than();
        vm.check_stacks();
        assert_eq!(vm.last_error(), Some(StackUnderflow));
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(-1).unwrap();
        vm.s_stack().push(0).unwrap();
        vm.less_than();
        vm.check_stacks();
        assert!(vm.last_error().is_none());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), -1);
        vm.s_stack().push(0).unwrap();
        vm.s_stack().push(0).unwrap();
        vm.less_than();
        vm.check_stacks();
        assert!(vm.last_error().is_none());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), 0);
    }

    #[test]
    fn test_equals() {
        let vm = &mut VM::new(16);
        vm.equals();
        vm.check_stacks();
        assert_eq!(vm.last_error(), Some(StackUnderflow));
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(0).unwrap();
        vm.equals();
        vm.check_stacks();
        assert_eq!(vm.last_error(), Some(StackUnderflow));
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(0).unwrap();
        vm.s_stack().push(0).unwrap();
        vm.equals();
        vm.check_stacks();
        assert!(vm.last_error().is_none());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), -1);
        vm.s_stack().push(-1).unwrap();
        vm.s_stack().push(0).unwrap();
        vm.equals();
        vm.check_stacks();
        assert!(vm.last_error().is_none());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), 0);
        vm.s_stack().push(1).unwrap();
        vm.s_stack().push(0).unwrap();
        vm.equals();
        vm.check_stacks();
        assert!(vm.last_error().is_none());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), 0);
    }

    #[test]
    fn test_greater_than() {
        let vm = &mut VM::new(16);
        vm.greater_than();
        vm.check_stacks();
        assert_eq!(vm.last_error(), Some(StackUnderflow));
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(1).unwrap();
        vm.greater_than();
        vm.check_stacks();
        assert_eq!(vm.last_error(), Some(StackUnderflow));
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(1).unwrap();
        vm.s_stack().push(0).unwrap();
        vm.greater_than();
        vm.check_stacks();
        assert!(vm.last_error().is_none());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), -1);
        vm.s_stack().push(0).unwrap();
        vm.s_stack().push(0).unwrap();
        vm.greater_than();
        vm.check_stacks();
        assert!(vm.last_error().is_none());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), 0);
    }

    #[test]
    fn test_not_equals() {
        let vm = &mut VM::new(16);
        vm.not_equals();
        vm.check_stacks();
        assert_eq!(vm.last_error(), Some(StackUnderflow));
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(0).unwrap();
        vm.not_equals();
        vm.check_stacks();
        assert_eq!(vm.last_error(), Some(StackUnderflow));
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(0).unwrap();
        vm.s_stack().push(0).unwrap();
        vm.not_equals();
        vm.check_stacks();
        assert!(vm.last_error().is_none());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), 0);
        vm.s_stack().push(-1).unwrap();
        vm.s_stack().push(0).unwrap();
        vm.not_equals();
        vm.check_stacks();
        assert!(vm.last_error().is_none());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), -1);
        vm.s_stack().push(1).unwrap();
        vm.s_stack().push(0).unwrap();
        vm.not_equals();
        vm.check_stacks();
        assert!(vm.last_error().is_none());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), -1);
    }

    #[test]
    fn test_between() {
        let vm = &mut VM::new(16);
        vm.between();
        vm.check_stacks();
        assert_eq!(vm.last_error(), Some(StackUnderflow));
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(1).unwrap();
        vm.between();
        vm.check_stacks();
        assert_eq!(vm.last_error(), Some(StackUnderflow));
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(1).unwrap();
        vm.s_stack().push(1).unwrap();
        vm.between();
        vm.check_stacks();
        assert_eq!(vm.last_error(), Some(StackUnderflow));
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(1).unwrap();
        vm.s_stack().push(1).unwrap();
        vm.s_stack().push(2).unwrap();
        vm.between();
        vm.check_stacks();
        assert!(vm.last_error().is_none());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), -1);
        vm.s_stack().push(1).unwrap();
        vm.s_stack().push(0).unwrap();
        vm.s_stack().push(1).unwrap();
        vm.between();
        vm.check_stacks();
        assert!(vm.last_error().is_none());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), -1);
        vm.s_stack().push(0).unwrap();
        vm.s_stack().push(1).unwrap();
        vm.s_stack().push(2).unwrap();
        vm.between();
        vm.check_stacks();
        assert!(vm.last_error().is_none());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), 0);
        vm.s_stack().push(3).unwrap();
        vm.s_stack().push(1).unwrap();
        vm.s_stack().push(2).unwrap();
        vm.between();
        vm.check_stacks();
        assert!(vm.last_error().is_none());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), 0);
    }

    #[test]
    fn test_invert() {
        let vm = &mut VM::new(16);
        vm.invert();
        vm.check_stacks();
        assert_eq!(vm.last_error(), Some(StackUnderflow));
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(707).unwrap();
        vm.invert();
        vm.check_stacks();
        assert!(vm.last_error().is_none());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), -708);
    }

    #[test]
    fn test_and() {
        let vm = &mut VM::new(16);
        vm.s_stack().push(707).unwrap();
        vm.s_stack().push(007).unwrap();
        vm.and();
        vm.check_stacks();
        assert!(vm.last_error().is_none());
        assert!(!vm.s_stack().overflow());
        assert!(!vm.s_stack().underflow());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), 3);
    }

    #[test]
    fn test_or() {
        let vm = &mut VM::new(16);
        vm.or();
        vm.check_stacks();
        assert_eq!(vm.last_error(), Some(StackUnderflow));
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(707).unwrap();
        vm.or();
        vm.check_stacks();
        assert_eq!(vm.last_error(), Some(StackUnderflow));
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(707).unwrap();
        vm.s_stack().push(07).unwrap();
        vm.or();
        vm.check_stacks();
        assert!(vm.last_error().is_none());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), 711);
    }

    #[test]
    fn test_xor() {
        let vm = &mut VM::new(16);
        vm.xor();
        vm.check_stacks();
        assert_eq!(vm.last_error(), Some(StackUnderflow));
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(707).unwrap();
        vm.xor();
        vm.check_stacks();
        assert_eq!(vm.last_error(), Some(StackUnderflow));
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(707).unwrap();
        vm.s_stack().push(07).unwrap();
        vm.xor();
        vm.check_stacks();
        assert!(vm.last_error().is_none());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), 708);
    }

    #[test]
    fn test_lshift() {
        let vm = &mut VM::new(16);
        vm.lshift();
        vm.check_stacks();
        assert_eq!(vm.last_error(), Some(StackUnderflow));
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(1).unwrap();
        vm.lshift();
        vm.check_stacks();
        assert_eq!(vm.last_error(), Some(StackUnderflow));
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(1).unwrap();
        vm.s_stack().push(1).unwrap();
        vm.lshift();
        vm.check_stacks();
        assert!(vm.last_error().is_none());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), 2);
        vm.s_stack().push(1).unwrap();
        vm.s_stack().push(2).unwrap();
        vm.lshift();
        vm.check_stacks();
        assert!(vm.last_error().is_none());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), 4);
    }

    #[test]
    fn test_rshift() {
        let vm = &mut VM::new(16);
        vm.rshift();
        vm.check_stacks();
        assert_eq!(vm.last_error(), Some(StackUnderflow));
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(8).unwrap();
        vm.rshift();
        vm.check_stacks();
        assert_eq!(vm.last_error(), Some(StackUnderflow));
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(8).unwrap();
        vm.s_stack().push(1).unwrap();
        vm.rshift();
        vm.check_stacks();
        assert!(vm.last_error().is_none());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), 4);
        vm.s_stack().push(-1).unwrap();
        vm.s_stack().push(1).unwrap();
        vm.rshift();
        vm.check_stacks();
        assert!(vm.last_error().is_none());
        assert_eq!(vm.s_stack().len(), 1);
        assert!(vm.s_stack().pop() > 0);
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
        assert_eq!(vm.s_stack().pop(), -3);
        assert_eq!(vm.s_stack().pop(), 2);
        assert_eq!(vm.s_stack().pop(), 0);
        assert_eq!(vm.s_stack().pop(), -1);
        assert_eq!(vm.s_stack().pop(), 0);
    }

    #[bench]
    fn bench_compile_words_at_beginning_of_wordlist(b: &mut Bencher) {
        let vm = &mut VM::new(16);
        b.iter(|| {
            vm.set_source("marker empty : main noop noop noop noop noop noop noop noop ; empty");
            vm.evaluate();
            vm.s_stack().reset();
        });
    }

    #[bench]
    fn bench_compile_words_at_end_of_wordlist(b: &mut Bencher) {
        let vm = &mut VM::new(16);
        b.iter(|| {
                   vm.set_source("marker empty : main bye bye bye bye bye bye bye bye ; empty");
                   vm.evaluate();
                   vm.s_stack().reset();
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
        assert_eq!(vm.s_stack().pop(), 5);
    }

    #[test]
    fn test_constant() {
        let vm = &mut VM::new(16);
        // constant x
        vm.set_source("constant");
        vm.evaluate();
        // Note: cannot detect underflow.
        // assert_eq!(vm.last_error(), Some(StackUnderflow));
        vm.reset();
        vm.clear_stacks();
        // 5 constant x x x
        vm.set_source("5 constant x x x");
        vm.evaluate();
        assert!(vm.last_error().is_none());
        assert_eq!(vm.s_stack().len(), 2);
        assert_eq!(vm.s_stack().pop(), 5);
        assert_eq!(vm.s_stack().pop(), 5);
    }

    #[test]
    fn test_variable_and_store_fetch() {
        let vm = &mut VM::new(16);
        // @
        vm.set_source("@");
        vm.evaluate();
        assert_eq!(vm.last_error(), Some(InvalidMemoryAddress));
        vm.reset();
        vm.clear_stacks();
        // !
        vm.set_source("!");
        vm.evaluate();
        assert_eq!(vm.last_error(), Some(InvalidMemoryAddress));
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
        assert_eq!(vm.s_stack().pop(), 3);
        assert_eq!(vm.s_stack().pop(), 0);
    }

    #[test]
    fn test_char_plus_and_chars() {
        let vm = &mut VM::new(16);
        vm.char_plus();
        vm.check_stacks();
        assert_eq!(vm.last_error(), Some(StackUnderflow));
        vm.reset();
        vm.chars();
        vm.check_stacks();
        // Note: Cannot detecht underflow because size of char is 1.
        // assert_eq!(vm.last_error(), Some(StackUnderflow));
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
        vm.check_stacks();
        assert_eq!(vm.last_error(), Some(StackUnderflow));
        vm.reset();
        vm.cells();
        vm.check_stacks();
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
        vm.check_stacks();
        assert_eq!(vm.last_error(), Some(UndefinedWord));
        vm.reset();
        vm.clear_stacks();
        // ' drop execute
        vm.set_source("' drop");
        vm.evaluate();
        vm.execute();
        vm.check_stacks();
        assert_eq!(vm.last_error(), Some(StackUnderflow));
        vm.reset();
        vm.clear_stacks();
        // 1 2  ' swap execute
        vm.set_source("1 2  ' swap execute");
        vm.evaluate();
        assert!(vm.last_error().is_none());
        assert_eq!(vm.s_stack().len(), 2);
        assert_eq!(vm.s_stack().pop(), 1);
        assert_eq!(vm.s_stack().pop(), 2);
    }

    #[test]
    fn test_here_allot() {
        let vm = &mut VM::new(16);
        // allot
        vm.allot();
        vm.check_stacks();
        assert_eq!(vm.last_error(), Some(StackUnderflow));
        vm.reset();
        // here 2 cells allot here -
        vm.set_source("here 2 cells allot here -");
        vm.evaluate();
        assert!(vm.last_error().is_none());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), -((mem::size_of::<i32>() * 2) as isize));
    }

    #[test]
    fn test_here_comma_compile_interpret() {
        let vm = &mut VM::new(16);
        vm.comma();
        vm.check_stacks();
        assert_eq!(vm.last_error(), Some(StackUnderflow));
        vm.reset();
        // here 1 , 2 , ] lit exit [ here
        let here = vm.data_space().len();
        vm.set_source("here 1 , 2 , ] lit exit [ here");
        vm.evaluate();
        assert!(vm.last_error().is_none());
        assert_eq!(vm.s_stack().len(), 2);
        let (n, t) = vm.s_stack().pop2();
        assert!(!vm.s_stack().underflow());
        assert_eq!(t - n, 4 * mem::size_of::<u32>() as isize);
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
        assert_eq!(vm.s_stack().pop(), 8);
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
        assert_eq!(vm.s_stack().pop(), -3);
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
        assert_eq!(vm.s_stack().pop(), 0);
        // : t2 1 if true else false then ; t2
        vm.set_source(": t2 1 if true else false then ; t2");
        vm.evaluate();
        assert!(vm.last_error().is_none());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), -1);
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
        assert_eq!(vm.s_stack().pop(), 3);
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
        assert_eq!(vm.s_stack().pop(), 3);
    }

    #[test]
    fn test_backslash() {
        let vm = &mut VM::new(16);
        vm.set_source("1 2 3 \\ 5 6 7");
        vm.evaluate();
        assert!(vm.last_error().is_none());
        assert_eq!(vm.s_stack().len(), 3);
        assert_eq!(vm.s_stack().pop(), 3);
        assert_eq!(vm.s_stack().pop(), 2);
        assert_eq!(vm.s_stack().pop(), 1);
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
        assert_eq!(vm.s_stack().pop(), -1);
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
        assert_eq!(vm.s_stack().pop(), 6);
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
        assert_eq!(vm.s_stack().pop(), 3);
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
        assert_eq!(vm.s_stack().pop(), 4);
        // : t4 1 6 0 do 1+ 2 +loop ;  t4
        vm.set_source(": t4 1 6 0 do 1+ 2 +loop ;  t4");
        vm.evaluate();
        assert!(vm.last_error().is_none());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), 4);
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
        assert_eq!(vm.s_stack().pop2(), (88, 9));
    }

    #[test]
    fn test_do_i_loop() {
        let vm = &mut VM::new(16);
        // : main 3 0 do i loop ;  main
        vm.set_source(": main 3 0 do i loop ;  main");
        vm.evaluate();
        assert_eq!(vm.last_error(), None);
        assert_eq!(vm.s_stack().len(), 3);
        assert_eq!(vm.s_stack().pop3(), (0, 1, 2));
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
