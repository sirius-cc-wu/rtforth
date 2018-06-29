extern crate libc;

use codespace::CodeSpace;
use dataspace::DataSpace;
use exception::Exception::{self, Abort, ControlStructureMismatch, DivisionByZero,
                           FloatingPointStackOverflow, FloatingPointStackUnderflow,
                           InterpretingACompileOnlyWord, InvalidMemoryAddress,
                           ReturnStackOverflow, ReturnStackUnderflow, StackOverflow,
                           StackUnderflow, UndefinedWord, UnexpectedEndOfFile,
                           UnsupportedOperation};
use parser;
use std::fmt::Write;
use std::fmt::{self, Display};
use std::mem;
use std::ops::{Index, IndexMut};
use std::result;
use std::str;
use {FALSE, TRUE};

pub type Result = result::Result<(), Exception>;

// Word
pub struct Word<Target> {
    symbol: Symbol,
    is_immediate: bool,
    is_compile_only: bool,
    hidden: bool,
    dfa: usize,
    cfa: usize,
    action: primitive!{ fn (&mut Target) },
    pub(crate) compilation_semantics: fn(&mut Target, usize),
}

impl<Target> Word<Target> {
    pub fn new(
        symbol: Symbol,
        action: primitive!{fn(&mut Target)},
        compilation_semantics: fn(&mut Target, usize),
        dfa: usize,
        cfa: usize
) -> Word<Target>{
        Word {
            symbol: symbol,
            is_immediate: false,
            is_compile_only: false,
            hidden: false,
            dfa: dfa,
            cfa: cfa,
            action: action,
            compilation_semantics: compilation_semantics,
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

    pub fn cfa(&self) -> usize {
        self.cfa
    }

pub fn action(&self) -> primitive!{fn(&mut Target)}{
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

    pub fn push(&mut self, v: T) {
        let len = self.len.wrapping_add(1);
        self.len = len;
        self.inner[len.wrapping_sub(1) as usize] = v;
    }

    pub fn pop(&mut self) -> T {
        let result = self.inner[self.len.wrapping_sub(1) as usize];
        self.len = self.len.wrapping_sub(1);
        result
    }

    pub fn push2(&mut self, v1: T, v2: T) {
        let len = self.len.wrapping_add(2);
        self.len = len;
        self.inner[self.len.wrapping_sub(2) as usize] = v1;
        self.inner[self.len.wrapping_sub(1) as usize] = v2;
    }

    pub fn push3(&mut self, v1: T, v2: T, v3: T) {
        let len = self.len.wrapping_add(3);
        self.len = len;
        self.inner[self.len.wrapping_sub(3) as usize] = v1;
        self.inner[self.len.wrapping_sub(2) as usize] = v2;
        self.inner[self.len.wrapping_sub(1) as usize] = v3;
    }

    pub fn pop2(&mut self) -> (T, T) {
        let result = (
            self.inner[self.len.wrapping_sub(2) as usize],
            self.inner[self.len.wrapping_sub(1) as usize],
        );
        self.len = self.len.wrapping_sub(2);
        result
    }

    pub fn pop3(&mut self) -> (T, T, T) {
        let result = (
            self.inner[self.len.wrapping_sub(3) as usize],
            self.inner[self.len.wrapping_sub(2) as usize],
            self.inner[self.len.wrapping_sub(1) as usize],
        );
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
    #[inline(always)]
    fn index(&self, index: u8) -> &isize {
        &self.inner[index as usize]
    }
}

impl IndexMut<u8> for Stack<isize> {
    #[inline(always)]
    fn index_mut(&mut self, index: u8) -> &mut isize {
        &mut self.inner[index as usize]
    }
}

impl Index<u8> for Stack<f64> {
    type Output = f64;
    #[inline(always)]
    fn index(&self, index: u8) -> &f64 {
        &self.inner[index as usize]
    }
}

impl IndexMut<u8> for Stack<f64> {
    #[inline(always)]
    fn index_mut(&mut self, index: u8) -> &mut f64 {
        &mut self.inner[index as usize]
    }
}

impl Index<u8> for Stack<Control> {
    type Output = Control;
    #[inline(always)]
    fn index(&self, index: u8) -> &Control {
        &self.inner[index as usize]
    }
}

impl fmt::Debug for Stack<isize> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
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
}

impl fmt::Debug for Stack<f64> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.len == 0 {
        } else {
            for i in 0..self.len {
                let v = self[i];
                match write!(f, "{:.7} ", v) {
                    Ok(_) => {}
                    Err(e) => {
                        return Err(e);
                    }
                }
            }
        }
        Ok(())
    }
}

pub struct ForwardReferences {
    pub idx_lit: usize,
    pub idx_flit: usize,
    pub idx_exit: usize,
    pub idx_zero_branch: usize,
    pub idx_branch: usize,
    pub idx_do: usize,
    pub idx_qdo: usize,
    pub idx_loop: usize,
    pub idx_plus_loop: usize,
    pub idx_s_quote: usize,
    pub idx_type: usize,
    pub idx_over: usize,
    pub idx_equal: usize,
    pub idx_drop: usize,
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
            idx_qdo: 0,
            idx_loop: 0,
            idx_plus_loop: 0,
            idx_s_quote: 0,
            idx_type: 0,
            idx_over: 0,
            idx_equal: 0,
            idx_drop: 0,
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

#[derive(Clone, Copy, PartialEq)]
pub enum Control {
    Default,
    Canary,
    If(usize),
    Else(usize),
    Begin(usize),
    While(usize),
    Do(usize, usize),
    Case,
    Of(usize),
    Endof(usize),
}

impl Default for Control {
    fn default() -> Self {
        Control::Default
    }
}

impl Display for Control {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match *self {
            Control::Default => "Default",
            Control::Canary => "Canary",
            Control::If(_) => "If",
            Control::Else(_) => "Else",
            Control::Begin(_) => "Begin",
            Control::While(_) => "While",
            Control::Do(_, _) => "Do",
            Control::Case => "Case",
            Control::Of(_) => "Of",
            Control::Endof(_) => "Endof",
        };
        write!(f, "{}", s)
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
    fn code_space(&mut self) -> &mut CodeSpace;
    fn code_space_const(&self) -> &CodeSpace;
    /// Numeric output buffer
    fn hold_buffer(&mut self) -> &mut String;
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
    fn regs(&mut self) -> &mut [usize; 2];
    fn s_stack(&mut self) -> &mut Stack<isize>;
    fn r_stack(&mut self) -> &mut Stack<isize>;
    fn c_stack(&mut self) -> &mut Stack<Control>;
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
        self.add_compile_only("lit", Core::lit); // Ngaro, jx, eForth
        self.add_compile_only("flit", Core::flit);
        self.add_compile_only("_s\"", Core::p_s_quote);
        self.add_compile_only("branch", Core::branch); // j1, eForth
        self.add_compile_only("0branch", Core::zero_branch); // j1, eForth
        self.add_compile_only("_do", Core::_do); // jx
        self.add_compile_only("_qdo", Core::_qdo); // jx
        self.add_compile_only("_loop", Core::_loop); // jx
        self.add_compile_only("_+loop", Core::_plus_loop); // jx
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
        self.add_immediate_and_compile_only("case", Core::imm_case);
        self.add_immediate_and_compile_only("of", Core::imm_of);
        self.add_immediate_and_compile_only("endof", Core::imm_endof);
        self.add_immediate_and_compile_only("endcase", Core::imm_endcase);
        self.add_immediate_and_compile_only("begin", Core::imm_begin);
        self.add_immediate_and_compile_only("while", Core::imm_while);
        self.add_immediate_and_compile_only("repeat", Core::imm_repeat);
        self.add_immediate_and_compile_only("again", Core::imm_again);
        self.add_immediate_and_compile_only("recurse", Core::imm_recurse);
        self.add_immediate_and_compile_only("do", Core::imm_do);
        self.add_immediate_and_compile_only("?do", Core::imm_qdo);
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
        self.add_primitive("within", Core::within);
        self.add_primitive("rot", Core::rot);
        self.add_primitive("-rot", Core::minus_rot);
        self.add_primitive("2dup", Core::two_dup);
        self.add_primitive("2drop", Core::two_drop);
        self.add_primitive("2swap", Core::two_swap);
        self.add_primitive("2over", Core::two_over);
        self.add_primitive("/", Core::slash);
        self.add_primitive("mod", Core::p_mod);
        self.add_primitive("abs", Core::abs);
        self.add_primitive("negate", Core::negate);
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
        self.references().idx_qdo = self.find("_qdo").expect("_qdo undefined");
        self.references().idx_loop = self.find("_loop").expect("_loop undefined");
        self.references().idx_plus_loop = self.find("_+loop").expect("_+loop undefined");
        self.references().idx_over = self.find("over").expect("over undefined");
        self.references().idx_equal = self.find("=").expect("= undefined");
        self.references().idx_drop = self.find("drop").expect("drop undefined");

        self.patch_compilation_semanticses();
    }

    /// Add a primitive word to word list.
fn add_primitive(&mut self, name: &str, action: primitive!{fn(&mut Self)}){
        let symbol = self.new_symbol(name);
        self.data_space().align();
        self.code_space().align();
        let word = Word::new(
            symbol,
            action,
            Core::compile_word,
            self.data_space().len(),
            self.code_space().here(),
        );
        let len = self.wordlist().len();
        self.set_last_definition(len);
        self.wordlist_mut().push(word);
    }

    /// Add an immediate word to word list.
fn add_immediate(&mut self, name: &str, action: primitive!{fn(&mut Self)}){
        self.add_primitive(name, action);
        let def = self.last_definition();
        self.wordlist_mut()[def].set_immediate(true);
    }

    /// Add a compile-only word to word list.
fn add_compile_only(&mut self, name: &str, action: primitive!{fn(&mut Self)}){
        self.add_primitive(name, action);
        let def = self.last_definition();
        self.wordlist_mut()[def].set_compile_only(true);
    }

    /// Add an immediate and compile-only word to word list.
fn add_immediate_and_compile_only(&mut self, name: &str, action: primitive!{fn(&mut Self)}){
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
        Symbol {
            id: self.symbols().len() - 1,
        }
    }

    // -------------------------------
    // Token threaded code
    // -------------------------------

    /// Evaluate a compiled program following self.state().instruction_pointer.
    /// Any exception causes termination of inner loop.
    #[inline(never)]
    #[cfg(not(feature = "subroutine-threaded"))]
    fn run(&mut self) {
        let mut ip = self.state().instruction_pointer;
        while 0 < ip && ip < self.data_space().len() {
            let w = self.data_space().get_i32(ip) as usize;
            self.state().instruction_pointer += mem::size_of::<i32>();
            self.execute_word(w);
            ip = self.state().instruction_pointer;
        }
    }

    #[cfg(not(feature = "subroutine-threaded"))]
    fn compile_word(&mut self, word_index: usize) {
        self.data_space().compile_u32(word_index as u32);
    }

    #[cfg(not(feature = "subroutine-threaded"))]
    fn compile_nest(&mut self, word_index: usize) {
        self.compile_word(word_index);
    }

    #[cfg(not(feature = "subroutine-threaded"))]
    fn compile_nest_code(&mut self, _: usize) {
        // Do nothing.
    }

    #[cfg(not(feature = "subroutine-threaded"))]
    fn compile_var(&mut self, word_index: usize) {
        self.compile_word(word_index);
    }

    #[cfg(not(feature = "subroutine-threaded"))]
    fn compile_const(&mut self, word_index: usize) {
        self.compile_word(word_index);
    }

    #[cfg(not(feature = "subroutine-threaded"))]
    fn compile_unmark(&mut self, word_index: usize) {
        self.compile_word(word_index);
    }

    #[cfg(not(feature = "subroutine-threaded"))]
    fn compile_fconst(&mut self, word_index: usize) {
        self.compile_word(word_index);
    }

    #[cfg(not(feature = "subroutine-threaded"))]
    primitive!{fn lit(&mut self) {
        let ip = self.state().instruction_pointer;
        let v = self.data_space().get_i32(ip) as isize;
        let slen = self.s_stack().len.wrapping_add(1);
        self.s_stack().len = slen;
        self.s_stack()[slen.wrapping_sub(1)] = v;
        self.state().instruction_pointer = self.state().instruction_pointer + mem::size_of::<i32>();
    }}

    /// Compile integer `i`.
    #[cfg(not(feature = "subroutine-threaded"))]
    fn compile_integer(&mut self, i: isize) {
        let idx = self.references().idx_lit;
        self.compile_word(idx);
        self.data_space().compile_i32(i as i32);
    }

    #[cfg(not(feature = "subroutine-threaded"))]
    primitive!{fn flit(&mut self) {
        let ip = DataSpace::aligned_f64(self.state().instruction_pointer as usize);
        let v = self.data_space().get_f64(ip);
        let flen = self.f_stack().len.wrapping_add(1);
        self.f_stack().len = flen;
        self.f_stack()[flen.wrapping_sub(1)] = v;
        self.state().instruction_pointer = ip + mem::size_of::<f64>();
    }}

    /// Compile float 'f'.
    #[cfg(not(feature = "subroutine-threaded"))]
    fn compile_float(&mut self, f: f64) {
        let idx_flit = self.references().idx_flit;
        self.compile_word(idx_flit);
        self.data_space().align_f64();
        self.data_space().compile_f64(f);
    }

    /// Runtime of S"
    #[cfg(not(feature = "subroutine-threaded"))]
    primitive!{fn p_s_quote(&mut self) {
        let ip = self.state().instruction_pointer;
        let cnt = self.data_space().get_i32(ip);
        let addr = self.state().instruction_pointer + mem::size_of::<i32>();
        let slen = self.s_stack().len.wrapping_add(2);
        self.s_stack().len = slen;
        self.s_stack()[slen.wrapping_sub(1)] = cnt as isize;
        self.s_stack()[slen.wrapping_sub(2)] = addr as isize;
        self.state().instruction_pointer =
            self.state().instruction_pointer + mem::size_of::<i32>() + cnt as usize;
    }}

    #[cfg(not(feature = "subroutine-threaded"))]
    fn patch_compilation_semanticses(&mut self) {
        let idx_leave = self.find("leave").expect("leave");
        self.wordlist_mut()[idx_leave].compilation_semantics = Self::compile_leave;
    }

    #[cfg(not(feature = "subroutine-threaded"))]
    primitive!{fn branch(&mut self) {
        let ip = self.state().instruction_pointer;
        self.state().instruction_pointer = self.data_space().get_i32(ip) as usize;
    }}

    #[cfg(not(feature = "subroutine-threaded"))]
    primitive!{fn zero_branch(&mut self) {
        let v = self.s_stack().pop();
        if v == 0 {
            self.branch();
        } else {
            self.state().instruction_pointer += mem::size_of::<i32>();
        }
    }}

    /// ( n1|u1 n2|u2 -- ) ( R: -- loop-sys )
    ///
    /// Set up loop control parameters with index `n2`|`u2` and limit `n1`|`u1`. An
    /// ambiguous condition exists if `n1`|`u1` and `n2`|`u2` are not both the same
    /// type.  Anything already on the return stack becomes unavailable until
    /// the loop-control parameters are discarded.
    ///
    ///         +--------------------------+
    ///         |                          |
    ///         |                          v
    /// +-----+-+-+-----------+-------+---+--
    /// | _do | x | loop body | _loop | x |
    /// +-----+---+-----------+-------+-+-+--
    ///         ^
    ///         |
    ///         ip
    ///
    #[cfg(not(feature = "subroutine-threaded"))]
    primitive!{fn _do(&mut self) {
        let ip = self.state().instruction_pointer as isize;
        self.r_stack().push(ip);
        self.state().instruction_pointer += mem::size_of::<i32>();
        self.two_to_r();
    }}

    /// ( n1|u1 n2|u2 -- ) ( R: -- loop-sys )
    ///
    /// If n1|u1 is equal to n2|u2, continue execution at the location given by
    /// the consumer of do-sys. Otherwise set up loop control parameters with
    /// index n2|u2 and limit n1|u1 and continue executing immediately
    /// following ?DO. Anything already on the return stack becomes unavailable
    /// until the loop control parameters are discarded. An ambiguous condition
    /// exists if n1|u1 and n2|u2 are not both of the same type.
    ///
    ///          +--------------------------+
    ///          |                          |
    ///          |                          v
    /// +------+-+-+-----------+-------+---+--
    /// | _qdo | x | loop body | _loop | x |
    /// +------+---+-----------+-------+-+-+--
    ///          ^
    ///          |
    ///          ip
    ///
    #[cfg(not(feature = "subroutine-threaded"))]
    primitive!{fn _qdo(&mut self) {
        let (n1, n2) = self.s_stack().pop2();
        if n1 == n2 {
            self.branch();
        } else {
            let ip = self.state().instruction_pointer as isize;
            self.r_stack().push(ip);
            self.state().instruction_pointer += mem::size_of::<i32>();
            self.r_stack().push2(n1, n2);
        }
    }}

    /// Run-time: ( -- ) ( R:  loop-sys1 --  | loop-sys2 )
    ///
    /// An ambiguous condition exists if the loop control parameters are
    /// unavailable. Add one to the loop index. If the loop index is then equal
    /// to the loop limit, discard the loop parameters and continue execution
    /// immediately following the loop. Otherwise continue execution at the
    /// beginning of the loop.
    #[cfg(not(feature = "subroutine-threaded"))]
    primitive!{fn _loop(&mut self) {
        let (rn, rt) = self.r_stack().pop2();
        if rt + 1 < rn {
            self.r_stack().push2(rn, rt + 1);
            self.branch();
        } else {
            let _ = self.r_stack().pop();
            self.state().instruction_pointer += mem::size_of::<i32>();
        }
    }}

    /// Run-time: ( n -- ) ( R: loop-sys1 -- | loop-sys2 )
    ///
    /// An ambiguous condition exists if the loop control parameters are
    /// unavailable. Add `n` to the loop index. If the loop index did not cross
    /// the boundary between the loop limit minus one and the loop limit,
    /// continue execution at the beginning of the loop. Otherwise, discard the
    /// current loop control parameters and continue execution immediately
    /// following the loop.
    #[cfg(not(feature = "subroutine-threaded"))]
    primitive!{fn _plus_loop(&mut self) {
        let (rn, rt) = self.r_stack().pop2();
        let t = self.s_stack().pop();
        if rt + t < rn {
            self.r_stack().push2(rn, rt + t);
            self.branch();
        } else {
            let _ = self.r_stack().pop();
            self.state().instruction_pointer += mem::size_of::<i32>();
        }
    }}

    /// Run-time: ( -- ) ( R: loop-sys -- )
    ///
    /// Discard the loop-control parameters for the current nesting level. An
    /// `UNLOOP` is required for each nesting level before the definition may be
    /// `EXIT`ed. An ambiguous condition exists if the loop-control parameters
    /// are unavailable.
    #[cfg(not(feature = "subroutine-threaded"))]
    primitive!{fn unloop(&mut self) {
        let _ = self.r_stack().pop3();
    }}


    #[cfg(not(feature = "subroutine-threaded"))]
    primitive!{fn leave(&mut self) {
        let (third, _, _) = self.r_stack().pop3();
        if self.r_stack().underflow() {
            self.abort_with(ReturnStackUnderflow);
            return;
        }
        self.state().instruction_pointer = self.data_space().get_i32(third as usize) as usize;
    }}

    #[cfg(not(feature = "subroutine-threaded"))]
    fn compile_leave(&mut self, word_idx: usize) {
        match self.leave_part() {
            Some(leave_part) => {
                self.compile_word(word_idx);
            }
            _ => {
                self.abort_with(ControlStructureMismatch);
                return;
            }
        };
    }

    #[cfg(not(feature = "subroutine-threaded"))]
    primitive!{fn p_j(&mut self) {
        let pos = self.r_stack().len() - 4;
        match self.r_stack().get(pos) {
            Some(j) => self.s_stack().push(j),
            None => self.abort_with(ReturnStackUnderflow),
        }
    }}

    fn leave_part(&mut self) -> Option<usize> {
        let position = self.c_stack().as_slice().iter().rposition(|&c| match c {
            Control::Do(_, _) => true,
            _ => false,
        });
        match position {
            Some(p) => match self.c_stack()[p as u8] {
                Control::Do(_, leave_part) => Some(leave_part),
                _ => None,
            },
            _ => None,
        }
    }

    /// IF A THEN
    ///
    ///         +------+
    ///         |      |
    ///         |      v
    /// +-----+---+---+--
    /// | _if | x | A |
    /// +-----+---+---+--
    ///         ^
    ///         |
    ///         ip
    ///
    #[cfg(not(feature = "subroutine-threaded"))]
    primitive!{fn imm_if(&mut self) {
        let idx = self.references().idx_zero_branch;
        self.compile_word(idx);
        self.data_space().compile_i32(0);
        let here = self.data_space().len();
        self.c_stack().push(Control::If(here));
    }}

    /// IF A ELSE B THEN
    ///
    ///             +--------------------+
    ///             |                    |
    ///             |                    v
    /// +---------+---+---+--------+---+---+--
    /// | ?branch | x | A | branch | x | B |
    /// +---------+---+---+--------+---+---+--
    ///             ^                |       ^
    ///             |                |       |
    ///             ip               +-------+
    ///
    #[cfg(not(feature = "subroutine-threaded"))]
    primitive!{fn imm_else(&mut self) {
        let if_part = match self.c_stack().pop() {
            Control::If(if_part) => if_part,
            _ => {
                self.abort_with(ControlStructureMismatch);
                return;
            }
        };
        if self.c_stack().underflow() {
            self.abort_with(ControlStructureMismatch);
        } else {
            let idx = self.references().idx_branch;
            self.compile_word(idx);
            self.data_space().compile_i32(0);
            let here = self.data_space().len();
            self.c_stack().push(Control::Else(here));
            self.data_space()
                .put_i32(here as i32, if_part - mem::size_of::<i32>());
        }
    }}

    #[cfg(not(feature = "subroutine-threaded"))]
    primitive!{fn imm_then(&mut self) {
        let branch_part = match self.c_stack().pop() {
            Control::If(branch_part) => branch_part,
            Control::Else(branch_part) => branch_part,
            _ => {
                self.abort_with(ControlStructureMismatch);
                return;
            }
        };
        if self.c_stack().underflow() {
            self.abort_with(ControlStructureMismatch);
        } else {
            let here = self.data_space().len();
            self.data_space()
                .put_i32(here as i32, branch_part - mem::size_of::<i32>());
        }
    }}

    /// n1 CASE
    ///   n2 OF A ENDOF
    ///   n3 OF B ENDOF
    ///   C
    /// ENDCASE
    /// D
    ///
    /// +-----+----+------+---+---------+---+------+---+--------+---+
    /// | lit | n2 | over | = | 0branch | x | drop | A | branch | x |
    /// +-----+----+------+---+---------+---+------+---+--------+---+
    ///                                   |                       |
    ///   +-------------------------------+                       +--------------+
    ///   |                                                       |              |
    ///   v                                                       |              v
    /// +-----+----+------+---+---------+---+------+---+--------+---+---+------+---+
    /// | lit | n3 | over | = | 0branch | x | drop | B | branch | x | C | drop | D |
    /// +-----+----+------+---+---------+---+------+---+--------+---+---+------+---+
    ///                                   |                           ^
    ///                                   |                           |
    ///                                   +---------------------------+
    ///
    #[cfg(not(feature = "subroutine-threaded"))]
    primitive!{fn imm_case(&mut self) {
        self.c_stack().push(Control::Case);
    }}

    #[cfg(not(feature = "subroutine-threaded"))]
    primitive!{fn imm_of(&mut self) {
        match self.c_stack().pop() {
            Control::Case => {
                self.c_stack().push(Control::Case);
            },
            Control::Endof(n) => {
                self.c_stack().push(Control::Endof(n));
            },
            _ => {
                self.abort_with(ControlStructureMismatch);
                return;
            }
        };
        if self.c_stack().underflow() {
            self.abort_with(ControlStructureMismatch);
        } else {
            let idx = self.references().idx_over;
            self.compile_word(idx);
            let idx = self.references().idx_equal;
            self.compile_word(idx);
            let idx = self.references().idx_zero_branch;
            self.compile_word(idx);
            self.data_space().compile_i32(0);
            let here = self.data_space().len();
            self.c_stack().push(Control::Of(here));
            let idx = self.references().idx_drop;
            self.compile_word(idx);
        }
    }}

    #[cfg(not(feature = "subroutine-threaded"))]
    primitive!{fn imm_endof(&mut self) {
        let of_part = match self.c_stack().pop() {
            Control::Of(of_part) => {
                of_part
            },
            _ => {
                self.abort_with(ControlStructureMismatch);
                return;
            }
        };
        if self.c_stack().underflow() {
            self.abort_with(ControlStructureMismatch);
        } else {
            let idx = self.references().idx_branch;
            self.compile_word(idx);
            self.data_space().compile_i32(0);
            let here = self.data_space().len();
            self.c_stack().push(Control::Endof(here));
            self.data_space()
                .put_i32(here as i32, of_part - mem::size_of::<i32>());
        }
    }}

    #[cfg(not(feature = "subroutine-threaded"))]
    primitive!{fn imm_endcase(&mut self) {
        let idx = self.references().idx_drop;
        self.compile_word(idx);
        loop {
            let endof_part = match self.c_stack().pop() {
                Control::Case => { break; }
                Control::Endof(endof_part) => {
                    endof_part
                }
                _ => {
                    self.abort_with(ControlStructureMismatch);
                    return;
                }
            };
            if self.c_stack().underflow() {
                self.abort_with(ControlStructureMismatch);
            } else {
                let here = self.data_space().len();
                self.data_space()
                    .put_i32(here as i32, endof_part - mem::size_of::<i32>());
            }
        }
        if self.c_stack().underflow() {
            self.abort_with(ControlStructureMismatch);
        }
    }}

    #[cfg(not(feature = "subroutine-threaded"))]
    primitive!{fn imm_begin(&mut self) {
        let here = self.data_space().len();
        self.c_stack().push(Control::Begin(here));
    }}

    #[cfg(not(feature = "subroutine-threaded"))]
    primitive!{fn imm_while(&mut self) {
        let idx = self.references().idx_zero_branch;
        self.compile_word(idx);
        self.data_space().compile_i32(0);
        let here = self.data_space().len();
        self.c_stack().push(Control::While(here));
    }}

    #[cfg(not(feature = "subroutine-threaded"))]
    primitive!{fn imm_repeat(&mut self) {
        let (begin_part, while_part) = match self.c_stack().pop2() {
            (Control::Begin(begin_part), Control::While(while_part)) => {
                (begin_part, while_part)
            },
            _ => {
                self.abort_with(ControlStructureMismatch);
                return;
            }
        };
        if self.c_stack().underflow() {
            self.abort_with(ControlStructureMismatch);
        } else {
            let idx = self.references().idx_branch;
            self.compile_word(idx);
            self.data_space().compile_i32(begin_part as i32);
            let here = self.data_space().len();
            self.data_space()
                .put_i32(here as i32, while_part - mem::size_of::<i32>());
        }
    }}

    #[cfg(not(feature = "subroutine-threaded"))]
    primitive!{fn imm_again(&mut self) {
        let begin_part = match self.c_stack().pop() {
            Control::Begin(begin_part) => begin_part,
            _ => {
                self.abort_with(ControlStructureMismatch);
                return;
            }
        };
        if self.c_stack().underflow() {
            self.abort_with(ControlStructureMismatch);
        } else {
            let idx = self.references().idx_branch;
            self.compile_word(idx);
            self.data_space().compile_i32(begin_part as i32);
        }
    }}

    /// Execution: ( -- a-ddr )
    ///
    /// Append the run-time semantics of `_do` to the current definition.
    /// The semantics are incomplete until resolved by `LOOP` or `+LOOP`.
    ///
    /// +-----+---+--
    /// | _do | 0 |
    /// +-----+---+--
    ///            ^
    ///            |
    ///            ++-----+
    ///             |     |
    /// Control::Do(here, here)
    ///
    #[cfg(not(feature = "subroutine-threaded"))]
    primitive!{fn imm_do(&mut self) {
        let idx = self.references().idx_do;
        self.compile_word(idx);
        self.data_space().compile_i32(0);
        let here = self.data_space().len();
        self.c_stack().push(Control::Do(here,here));
    }}

    primitive!{fn imm_recurse(&mut self) {
        let last = self.wordlist().len() - 1;
        self.compile_nest(last);
    }}

    /// Execution: ( -- a-ddr )
    ///
    /// Append the run-time semantics of `_qdo` to the current definition.
    /// The semantics are incomplete until resolved by `LOOP` or `+LOOP`.
    ///
    /// +------+---+--
    /// | _qdo | 0 |
    /// +------+---+--
    ///             ^
    ///             |
    ///             ++-----+
    ///              |     |
    /// Control::Do(here, here)
    ///
    #[cfg(not(feature = "subroutine-threaded"))]
    primitive!{fn imm_qdo(&mut self) {
        let idx = self.references().idx_qdo;
        self.compile_word(idx);
        self.data_space().compile_i32(0);
        let here = self.data_space().len();
        self.c_stack().push(Control::Do(here,here));
    }}

    /// Run-time: ( a-addr -- )
    ///
    /// Append the run-time semantics of `_LOOP` to the current definition.
    /// Resolve the destination of all unresolved occurrences of `LEAVE` between
    /// the location given by do-sys and the next location for a transfer of
    /// control, to execute the words following the `LOOP`.
    ///
    /// For DO ... LOOP,
    ///
    ///         +--------------------------+
    ///         |                          |
    ///         |                          v
    /// +-----+-+-+-----------+-------+---+--
    /// | _do | x | loop body | _loop | x |
    /// +-----+---+-----------+-------+-+-+--
    ///            ^                    |
    ///            |                    |
    ///            ++-------------------+
    ///             |
    /// Control::Do(do_part, _)
    ///
    /// For ?DO ... LOOP,
    ///
    ///          +--------------------------+
    ///          |                          |
    ///          |                          v
    /// +------+-+-+-----------+-------+---+--
    /// | _qdo | x | loop body | _loop | x |
    /// +------+---+-----------+-------+-+-+--
    ///             ^                    |
    ///             |                    |
    ///             +--------------------+
    ///             |
    /// Control::Do(do_part, _)
    ///
    #[cfg(not(feature = "subroutine-threaded"))]
    primitive!{fn imm_loop(&mut self) {
        let do_part = match self.c_stack().pop() {
            Control::Do(do_part,_) => do_part,
            _ => {
                self.abort_with(ControlStructureMismatch);
                return;
            }
        };
        if self.c_stack().underflow() {
            self.abort_with(ControlStructureMismatch);
        } else {
            let idx = self.references().idx_loop;
            self.compile_word(idx);
            self.data_space().compile_i32(do_part as i32);
            let here = self.data_space().len();
            self.data_space()
                .put_i32(here as i32, (do_part - mem::size_of::<i32>()) as usize);
        }
    }}

    /// Run-time: ( a-addr -- )
    ///
    /// Append the run-time semantics of `_+LOOP` to the current definition.
    /// Resolve the destination of all unresolved occurrences of `LEAVE` between
    /// the location given by do-sys and the next location for a transfer of
    /// control, to execute the words following `+LOOP`.
    #[cfg(not(feature = "subroutine-threaded"))]
    primitive!{fn imm_plus_loop(&mut self) {
        let do_part = match self.c_stack().pop() {
            Control::Do(do_part,_) => do_part,
            _ => {
                self.abort_with(ControlStructureMismatch);
                return;
            }
        };
        if self.c_stack().underflow() {
            self.abort_with(ControlStructureMismatch);
        } else {
            let idx = self.references().idx_plus_loop;
            self.compile_word(idx);
            self.data_space().compile_i32(do_part as i32);
            let here = self.data_space().len();
            self.data_space()
                .put_i32(here as i32, (do_part - mem::size_of::<i32>()) as usize);
        }
    }}

    // -------------------------------
    // Subroutine threaded code
    // -------------------------------

    /// Evaluate a compiled program following self.state().instruction_pointer.
    /// Any exception causes termination of inner loop.
    #[inline(never)]
    #[cfg(all(feature = "subroutine-threaded", target_arch = "x86"))]
    fn run(&mut self) {
        // Do nothing.
    }

    #[cfg(all(feature = "subroutine-threaded", target_arch = "x86"))]
    fn compile_word(&mut self, word_index: usize) {
        // 89 f1            mov    %esi,%ecx
        // e8 xx xx xx xx   call   self.wordlist()[word_index].action
        self.code_space().compile_u8(0x89);
        self.code_space().compile_u8(0xf1);
        self.code_space().compile_u8(0xe8);
        let w = self.wordlist()[word_index].action as usize;
        self.code_space().compile_relative(w);
    }

    #[cfg(all(feature = "subroutine-threaded", target_arch = "x86"))]
    fn compile_nest(&mut self, word_index: usize) {
        self.compile_word(word_index);
    }

    #[cfg(all(feature = "subroutine-threaded", target_arch = "x86"))]
    fn compile_nest_code(&mut self, word_index: usize) {
        self.wordlist_mut()[word_index].action =
            unsafe { mem::transmute(self.code_space().here()) };
        // ; make a copy of vm in %esi because %ecx may be destropyed by subroutine call.
        // 56               push   %esi
        // 89 ce            mov    %ecx,%esi
        // 83 ec 08         sub    $8,%esp
        self.code_space().compile_u8(0x56);
        self.code_space().compile_u8(0x89);
        self.code_space().compile_u8(0xce);
        self.code_space().compile_u8(0x83);
        self.code_space().compile_u8(0xec);
        self.code_space().compile_u8(0x08);
    }

    #[cfg(all(feature = "subroutine-threaded", target_arch = "x86"))]
    fn compile_exit(&mut self, _: usize) {
        // 83 c4 08         add    $8,%esp
        // 5e               pop    %esi
        // c3               ret
        self.code_space().compile_u8(0x83);
        self.code_space().compile_u8(0xc4);
        self.code_space().compile_u8(0x08);
        self.code_space().compile_u8(0x5e);
        self.code_space().compile_u8(0xc3);
    }

    #[cfg(all(feature = "subroutine-threaded", target_arch = "x86"))]
    fn compile_var(&mut self, word_index: usize) {
        let dfa = self.wordlist()[word_index].dfa();
        self.compile_integer(dfa as isize);
    }

    #[cfg(all(feature = "subroutine-threaded", target_arch = "x86"))]
    fn compile_const(&mut self, word_index: usize) {
        let dfa = self.wordlist()[word_index].dfa();
        let value = self.data_space().get_i32(dfa) as isize;
        self.compile_integer(value);
    }

    #[cfg(all(feature = "subroutine-threaded", target_arch = "x86"))]
    fn compile_unmark(&mut self, _: usize) {
        // Do nothing.
    }

    #[cfg(all(feature = "subroutine-threaded", target_arch = "x86"))]
    fn compile_fconst(&mut self, word_index: usize) {
        // self.compile_word(word_index);
    }

    #[cfg(all(feature = "subroutine-threaded", target_arch = "x86"))]
    primitive!{fn lit(&mut self) {
        // Do nothing.
    }}

    #[cfg(all(feature = "subroutine-threaded", target_arch = "x86"))]
    primitive!{fn lit_integer(&mut self, i: isize) {
        let slen = self.s_stack().len.wrapping_add(1);
        self.s_stack().len = slen;
        self.s_stack()[slen.wrapping_sub(1)] = i;
    }}

    /// Compile integer `i`.
    #[cfg(all(feature = "subroutine-threaded", target_arch = "x86"))]
    fn compile_integer(&mut self, i: isize) {
        // ba nn nn nn nn   mov    $0xnnnn,%edx
        // 89 f1            mov    %esi,%ecx
        // e8 xx xx xx xx   call   lit_integer
        self.code_space().compile_u8(0xba);
        self.code_space().compile_i32(i as i32);
        self.code_space().compile_u8(0x89);
        self.code_space().compile_u8(0xf1);
        self.code_space().compile_u8(0xe8);
        self.code_space()
            .compile_relative(Self::lit_integer as usize);
    }

    #[cfg(all(feature = "subroutine-threaded", target_arch = "x86"))]
    primitive!{fn flit(&mut self) {
        // Do nothing.
    }}

    #[cfg(all(feature = "subroutine-threaded", target_arch = "x86"))]
    primitive!{fn lit_float(&mut self, f: f64) {
        let flen = self.f_stack().len.wrapping_add(1);
        self.f_stack().len = flen;
        self.f_stack()[flen.wrapping_sub(1)] = f;
    }}

    #[cfg(all(feature = "subroutine-threaded", target_arch = "x86"))]
    fn compile_float(&mut self, f: f64) {
        // ba nn nn nn nn   mov    <addr of f>, %edx
        // 83 ec 08         sub    $0x08,%esp
        // f2 0f 10 02      movsd  (%edx), %xmm0
        // 89 f1            mov    %esi, %ecx
        // f2 0f 11 04 24   movsd  %xmm0,(%esp)
        // e8 xx xx xx xx   call   lit_float
        self.data_space().align_f64();
        let data_addr = self.data_space().here();
        self.data_space().compile_f64(f);
        self.code_space().compile_u8(0xba);
        self.code_space().compile_u32(data_addr as u32);
        self.code_space().compile_u8(0x83);
        self.code_space().compile_u8(0xec);
        self.code_space().compile_u8(0x08);
        self.code_space().compile_u8(0xf2);
        self.code_space().compile_u8(0x0f);
        self.code_space().compile_u8(0x10);
        self.code_space().compile_u8(0x02);
        self.code_space().compile_u8(0x89);
        self.code_space().compile_u8(0xf1);
        self.code_space().compile_u8(0xf2);
        self.code_space().compile_u8(0x0f);
        self.code_space().compile_u8(0x11);
        self.code_space().compile_u8(0x04);
        self.code_space().compile_u8(0x24);
        self.code_space().compile_u8(0xe8);
        self.code_space().compile_relative(Self::lit_float as usize);
    }

    /// Runtime of S"
    #[cfg(all(feature = "subroutine-threaded", target_arch = "x86"))]
    primitive!{fn p_s_quote(&mut self) {
        // Do nothing.
    }}

    #[cfg(all(feature = "subroutine-threaded", target_arch = "x86"))]
    primitive!{fn lit_counted_string(&mut self, idx: usize) {
        let cnt = self.data_space().get_i32(idx);
        let addr = idx + mem::size_of::<i32>();
        // FIXME
        // 加下一行會造成 cargo test test_s_quote_and_type --features="subroutine-threaded"
        // 時 invalid memory reference 。但如果列印的方式改為 {} 就 ok。
        // 懷疑是 compile_s_quote 設計有問題。或是 println 改變了 calling convension。
        // 但奇怪的是，無法用 objdump -d 找到 compile_s_quote 以及 lit_counted_string，以致於無法
        // 理解。
        // println!("lit_counted_string: addr: {:x}!", addr);
        let slen = self.s_stack().len.wrapping_add(2);
        self.s_stack().len = slen;
        self.s_stack()[slen.wrapping_sub(1)] = cnt as isize;
        self.s_stack()[slen.wrapping_sub(2)] = addr as isize;
    }}

    #[cfg(all(feature = "subroutine-threaded", target_arch = "x86"))]
    fn compile_s_quote(&mut self, _: usize) {
        // ba nn nn nn nn   mov    <index of counted string>, %edx
        // 89 f1            mov    %esi, %ecx
        // e8 xx xx xx xx   call   lit_counted_string
        let data_idx = self.data_space().len();
        self.code_space().compile_u8(0xba);
        self.code_space().compile_u32(data_idx as u32);
        self.code_space().compile_u8(0x89);
        self.code_space().compile_u8(0xf1);
        self.code_space().compile_u8(0xe8);
        self.code_space()
            .compile_relative(Self::lit_counted_string as usize);
    }

    #[cfg(all(feature = "subroutine-threaded", target_arch = "x86"))]
    fn patch_compilation_semanticses(&mut self) {
        let idx_exit = self.find("exit").expect("exit");
        self.wordlist_mut()[idx_exit].compilation_semantics = Self::compile_exit;
        let idx_s_quote = self.find("_s\"").expect("_s\"");
        self.wordlist_mut()[idx_s_quote].compilation_semantics = Self::compile_s_quote;
        let idx_leave = self.find("leave").expect("leave");
        self.wordlist_mut()[idx_leave].compilation_semantics = Self::compile_leave;
        let idx_reset = self.find("reset").expect("reset");
        self.wordlist_mut()[idx_reset].compilation_semantics = Self::compile_reset;
    }

    #[cfg(all(feature = "subroutine-threaded", target_arch = "x86"))]
    primitive!{fn branch(&mut self) {
        // Do nothing.
    }}

    #[cfg(all(feature = "subroutine-threaded", target_arch = "x86"))]
    fn compile_branch(&mut self, destination: usize) -> usize {
        // e9 xx xx xx xx      jmp xxxx
        let here = self.code_space().here();
        self.code_space().compile_u8(0xe9);
        self.code_space().compile_relative(destination);
        here
    }

    #[cfg(all(feature = "subroutine-threaded", target_arch = "x86"))]
    primitive!{fn zero_branch(&mut self) {
        // Do nothing.
    }}

    #[cfg(all(feature = "subroutine-threaded", target_arch = "x86"))]
    fn compile_zero_branch(&mut self, destination: usize) -> usize {
        // 89 f1                mov    %esi,%ecx
        // e8 xx xx xx xx       call   pop_stack ; pop value into %eax.
        // 85 c0                test   %eax,%eax
        // 0f 84 yy yy yy yy    je     yyyy
        self.code_space().compile_u8(0x89);
        self.code_space().compile_u8(0xf1);
        self.code_space().compile_u8(0xe8);
        self.code_space()
            .compile_relative(Self::pop_s_stack as usize);
        self.code_space().compile_u8(0x85);
        self.code_space().compile_u8(0xc0);
        let here = self.code_space().here();
        self.code_space().compile_u8(0x0f);
        self.code_space().compile_u8(0x84);
        self.code_space().compile_relative(destination);
        here
    }

    #[cfg(all(feature = "subroutine-threaded", target_arch = "x86"))]
    primitive!{fn _do(&mut self) {
        // Do nothing.
    }}

    #[cfg(all(feature = "subroutine-threaded", target_arch = "x86"))]
    primitive!{fn _stc_do(&mut self) {
        self.two_to_r();
    }}

    #[cfg(all(feature = "subroutine-threaded", target_arch = "x86"))]
    primitive!{fn _qdo(&mut self) {
        // Do nothing.
    }}

    #[cfg(all(feature = "subroutine-threaded", target_arch = "x86"))]
    primitive!{fn _stc_qdo(&mut self) -> isize {
        let (n1, n2) = self.s_stack().pop2();
        if n1 == n2 {
            -1
        } else {
            self.r_stack().push2(n1, n2);
            0
        }
    }}

    #[cfg(all(feature = "subroutine-threaded", target_arch = "x86"))]
    primitive!{fn _loop(&mut self) {
        // Do nothing.
    }}

    #[cfg(all(feature = "subroutine-threaded", target_arch = "x86"))]
    primitive!{fn _stc_loop(&mut self) -> isize {
        let (rn, rt) = self.r_stack().pop2();
        if rt + 1 < rn {
            self.r_stack().push2(rn, rt + 1);
            0
        } else {
            -1
        }
    }}

    #[cfg(all(feature = "subroutine-threaded", target_arch = "x86"))]
    primitive!{fn _plus_loop(&mut self) {
        // Do nothing.
    }}

    #[cfg(all(feature = "subroutine-threaded", target_arch = "x86"))]
    primitive!{fn _stc_plus_loop(&mut self) -> isize {
        let (rn, rt) = self.r_stack().pop2();
        let t = self.s_stack().pop();
        if rt + t < rn {
            self.r_stack().push2(rn, rt + t);
            0
        } else {
            -1
        }
    }}

    #[cfg(all(feature = "subroutine-threaded", target_arch = "x86"))]
    primitive!{fn unloop(&mut self) {
        let _ = self.r_stack().pop2();
    }}

    #[cfg(all(feature = "subroutine-threaded", target_arch = "x86"))]
    primitive!{fn leave(&mut self) {
        // Do nothing.
    }}

    /// Code space
    /// +-----------+------+--
    /// | loop body | LOOP |
    /// +-----------+------+--
    ///                     ^
    ///                     |
    ///               +-----+
    ///               |
    /// Data space    |
    ///             +---+--
    ///             | x |
    ///             +---+--
    ///           leave_part
    ///
    #[cfg(all(feature = "subroutine-threaded", target_arch = "x86"))]
    fn compile_leave(&mut self, _: usize) {
        let leave_part = match self.leave_part() {
            Some(leave_part) => leave_part,
            _ => {
                self.abort_with(ControlStructureMismatch);
                return;
            }
        };
        // 89 f1                mov    %esi,%ecx
        // e8 xx xx xx xx       call   unloop
        // b8 yy yy yy yy       mov    leave_part,%eax
        // ff 20                jmp    *(%eax)
        self.code_space().compile_u8(0x89);
        self.code_space().compile_u8(0xf1);
        self.code_space().compile_u8(0xe8);
        self.code_space().compile_relative(Self::unloop as usize);
        self.code_space().compile_u8(0xb8);
        self.code_space().compile_u32(leave_part as u32);
        self.code_space().compile_u8(0xff);
        self.code_space().compile_u8(0x20);
    }

    primitive!{fn p_i(&mut self) {
        match self.r_stack().last() {
            Some(i) => self.s_stack().push(i),
            None => self.abort_with(ReturnStackUnderflow),
        }
    }}

    #[cfg(all(feature = "subroutine-threaded", target_arch = "x86"))]
    primitive!{fn p_j(&mut self) {
        let pos = self.r_stack().len() - 3;
        match self.r_stack().get(pos) {
            Some(j) => self.s_stack().push(j),
            None => self.abort_with(ReturnStackUnderflow),
        }
    }}

    #[cfg(all(feature = "subroutine-threaded", target_arch = "x86"))]
    primitive!{fn imm_if(&mut self) {
        let here = self.compile_zero_branch(0);
        self.c_stack().push(Control::If(here));
    }}

    #[cfg(all(feature = "subroutine-threaded", target_arch = "x86"))]
    primitive!{fn imm_else(&mut self) {
        let if_part = match self.c_stack().pop() {
            Control::If(if_part) => if_part,
            _ => {
                self.abort_with(ControlStructureMismatch);
                return;
            }
        };
        if self.c_stack().underflow() {
            self.abort_with(ControlStructureMismatch);
        } else {
            let else_part = self.compile_branch(0);
            self.c_stack().push(Control::Else(else_part));
            // if_part: 0f 84 yy yy yy yy    je yyyy
            let here = self.code_space().here();
            unsafe{
                self.code_space()
                    .put_i32((here - (if_part + 2 + mem::size_of::<i32>())) as i32, (if_part + 2));
            }
        }
    }}

    #[cfg(all(feature = "subroutine-threaded", target_arch = "x86"))]
    primitive!{fn imm_then(&mut self) {
        let branch_part = match self.c_stack().pop() {
            Control::If(branch_part) => branch_part,
            Control::Else(branch_part) => branch_part,
            _ => {
                self.abort_with(ControlStructureMismatch);
                return;
            }
        };
        if self.c_stack().underflow() {
            self.abort_with(ControlStructureMismatch);
        } else {
            // branch_part:
            //      0f 84 yy yy yy yy   je yyyy
            // or
            //      e9 xx xx xx xx      jmp xxxx
            let here = self.code_space().here();
            unsafe{
                let c = self.code_space().get_u8(branch_part);
                if c == 0x0f {
                    self.code_space()
                        .put_i32((here - (branch_part + 2 +
                         mem::size_of::<i32>())) as i32, (branch_part + 2));
                } else {
                    self.code_space()
                        .put_i32((here - (branch_part +
                         1 + mem::size_of::<i32>())) as i32, (branch_part + 1));
                }
            }
        }
    }}

    #[cfg(all(feature = "subroutine-threaded", target_arch = "x86"))]
    primitive!{fn imm_begin(&mut self) {
        let here = self.code_space().here();
        self.c_stack().push(Control::Begin(here));
    }}

    #[cfg(all(feature = "subroutine-threaded", target_arch = "x86"))]
    primitive!{fn imm_while(&mut self) {
        let while_part = self.compile_zero_branch(0);
        self.c_stack().push(Control::While(while_part));
    }}

    #[cfg(all(feature = "subroutine-threaded", target_arch = "x86"))]
    primitive!{fn imm_repeat(&mut self) {
        let (begin_part, while_part) = match self.c_stack().pop2() {
            (Control::Begin(begin_part), Control::While(while_part)) => (begin_part, while_part),
            _ => {
                self.abort_with(ControlStructureMismatch);
                return;
            }
        };
        if self.c_stack().underflow() {
            self.abort_with(ControlStructureMismatch);
        } else {
            let _ = self.compile_branch(begin_part);
            // while_part: 0f 84 yy yy yy yy    je yyyy
            let here = self.code_space().here();
            unsafe{
                self.code_space()
                    .put_i32((here - (while_part +
                     2 + mem::size_of::<i32>())) as i32, (while_part + 2));
            }
        }
    }}

    #[cfg(all(feature = "subroutine-threaded", target_arch = "x86"))]
    primitive!{fn imm_again(&mut self) {
        let begin_part = match self.c_stack().pop() {
            Control::Begin(begin_part) => begin_part,
            _ => {
                self.abort_with(ControlStructureMismatch);
                return;
            }
        };
        if self.c_stack().underflow() {
            self.abort_with(ControlStructureMismatch);
        } else {
            let _ = self.compile_branch(begin_part);
        }
    }}

    /// Code space
    /// +-----------------+--------------+--
    /// | move %esi, %ecx | call _stc_do |
    /// +-----------------+--------------+--
    ///                                   ^
    ///                                   |
    ///             +---------------------+
    ///             |
    /// Control::Do(here, leave_part)
    ///                   |
    ///               +---+
    ///               |
    /// Data space    v
    ///             +---+--
    ///             | 0 |
    ///             +---+--
    ///
    #[cfg(all(feature = "subroutine-threaded", target_arch = "x86"))]
    primitive!{fn imm_do(&mut self) {
        // 89 f1                mov    %esi,%ecx
        // e8 xx xx xx xx       call   _stc_do
        self.code_space().compile_u8(0x89);
        self.code_space().compile_u8(0xf1);
        self.code_space().compile_u8(0xe8);
        self.code_space().compile_relative(Self::_stc_do as usize);
        let here = self.code_space().here();
        let leave_part = self.data_space().here();
        self.data_space().compile_u32(0);
        self.c_stack().push(Control::Do(here, leave_part));
    }}

    /// Code space
    /// +-------+-----------+------+--
    /// | (?DO) | loop body | LOOP |
    /// +-------+-----------+------+--
    ///          ^                  ^
    ///          |                  |
    ///          +--+               +----+
    ///             |                    |
    /// Control::Do(here, leave_part)    |
    ///                   |              |
    ///               +---+              |
    ///               |                  |
    /// Data space    v                  |
    ///             +---+--              |
    ///             | 0 |                |
    ///             +---+--              |
    ///               |                  |
    ///               +------------------+
    ///
    #[cfg(all(feature = "subroutine-threaded", target_arch = "x86"))]
    primitive!{fn imm_qdo(&mut self) {
        //   89 f1                mov    %esi,%ecx
        //   e8 xx xx xx xx       call   _stc_qdo
        //   85 c0                test   %eax,%eax
        //   0f 84 yy yy yy yy    je     do_part
        //   b8 yy yy yy yy       mov    leave_part,%eax
        //   ff 20                jmp    *(%eax)
        // do_part:
        self.code_space().compile_u8(0x89);
        self.code_space().compile_u8(0xf1);
        self.code_space().compile_u8(0xe8);
        self.code_space().compile_relative(Self::_stc_qdo as usize);
        self.code_space().compile_u8(0x85);
        self.code_space().compile_u8(0xc0);
        self.code_space().compile_u8(0x0f);
        self.code_space().compile_u8(0x84);
        self.code_space().compile_u32(7);
        let leave_part = self.data_space().here();
        self.data_space().compile_u32(0);
        self.code_space().compile_u8(0xb8);
        self.code_space().compile_u32(leave_part as u32);
        self.code_space().compile_u8(0xff);
        self.code_space().compile_u8(0x20);
        let here = self.code_space().here();
        self.c_stack().push(Control::Do(here, leave_part));
    }}

    /// Code space
    /// +-----------+-----------+--
    /// | loop body | loop code |
    /// +-----------+-----------+--
    ///  ^                       ^
    ///  |                       |
    ///  |                       +----------+
    ///  +----------+                       |
    ///             |                       |
    /// Control::Do(do_part, leave_part)    |
    ///                      |              |
    ///               +------+              |
    ///               |                     |
    /// Data space    v                     |
    ///             +---+--                 |
    ///             | x |                   |
    ///             +---+--                 |
    ///               |                     |
    ///               +---------------------+
    ///
    #[cfg(all(feature = "subroutine-threaded", target_arch = "x86"))]
    primitive!{fn imm_loop(&mut self) {
        let (do_part, leave_part) = match self.c_stack().pop() {
            Control::Do(do_part, leave_part) => (do_part, leave_part),
            _ => {
                self.abort_with(ControlStructureMismatch);
                return;
            }
        };
        if self.c_stack().underflow() {
            self.abort_with(ControlStructureMismatch);
        } else {
            //      89 f1                mov    %esi,%ecx
            //      e8 xx xx xx xx       call    _stc_loop
            //      85 c0                test   %eax,%eax
            //      0f 84 yy yy yy yy    je     do_part
            // leave_part:
            self.code_space().compile_u8(0x89);
            self.code_space().compile_u8(0xf1);
            self.code_space().compile_u8(0xe8);
            self.code_space().compile_relative(Self::_stc_loop as usize);
            self.code_space().compile_u8(0x85);
            self.code_space().compile_u8(0xc0);
            self.code_space().compile_u8(0x0f);
            self.code_space().compile_u8(0x84);
            self.code_space().compile_relative(do_part);
            // TODO: 目前 data_space() 的 put_u32 和 code_space() 的作法不同。
            // 未來 data_space() 要改為和 code_space() 作法一樣。雖然 leave_part 在
            // data space，這兒先使用 code_space() 的作法。
            let here = self.code_space().here();
            unsafe{ self.code_space().put_u32(here as u32, leave_part); }
        }
    }}

    #[cfg(all(feature = "subroutine-threaded", target_arch = "x86"))]
    primitive!{fn imm_plus_loop(&mut self) {
        let (do_part, leave_part) = match self.c_stack().pop() {
            Control::Do(do_part, leave_part) => (do_part, leave_part),
            _ => {
                self.abort_with(ControlStructureMismatch);
                return;
            }
        };
        if self.c_stack().underflow() {
            self.abort_with(ControlStructureMismatch);
        } else {
            //      89 f1                mov    %esi,%ecx
            //      e8 xx xx xx xx       call    _stc_plus_loop
            //      85 c0                test   %eax,%eax
            //      0f 84 yy yy yy yy    je     do_part
            // leave_part:
            self.code_space().compile_u8(0x89);
            self.code_space().compile_u8(0xf1);
            self.code_space().compile_u8(0xe8);
            self.code_space().compile_relative(Self::_stc_plus_loop as usize);
            self.code_space().compile_u8(0x85);
            self.code_space().compile_u8(0xc0);
            self.code_space().compile_u8(0x0f);
            self.code_space().compile_u8(0x84);
            self.code_space().compile_relative(do_part);
            // TODO: 目前 data_space() 的 put_u32 和 code_space() 的作法不同。
            // 未來 data_space() 要改為和 code_space() 作法一樣。雖然 leave_part 在
            // data space，這兒先使用 code_space() 的作法。
            let here = self.code_space().here();
            unsafe{ self.code_space().put_u32(here as u32, leave_part); }
        }
    }}

    // -----------
    // Evaluation
    // -----------

    primitive!{fn left_bracket(&mut self) {
        self.state().is_compiling = false;
    }}

    primitive!{fn right_bracket(&mut self) {
        self.state().is_compiling = true;
    }}

    /// Copy content of `s` to `input_buffer` and set `source_index` to 0.
    fn set_source(&mut self, s: &str) {
        let mut buffer = self.input_buffer().take().expect("input buffer");
        buffer.clear();
        buffer.push_str(s);
        self.state().source_index = 0;
        self.set_input_buffer(buffer);
    }

    /// Push content of `s` to `input_buffer`.
    fn push_source(&mut self, s: &str) {
        let mut buffer = self.input_buffer().take().expect("input buffer");
        buffer.push_str(s);
        self.set_input_buffer(buffer);
    }

    /// Run-time: ( "ccc" -- )
    ///
    /// Parse word delimited by white space, skipping leading white spaces.
    primitive!{fn parse_word(&mut self) {
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
    }}

    /// Run-time: ( "&lt;spaces&gt;name" -- char)
    ///
    /// Skip leading space delimiters. Parse name delimited by a space.
    /// Put the value of its first character onto the stack.
    primitive!{fn char(&mut self) {
        self.parse_word();
        let last_token = self.last_token().take().expect("token");
        match last_token.chars().nth(0) {
            Some(c) => {
                self.set_last_token(last_token);
                self.s_stack().push(c as isize);
            }
            None => {
                self.set_last_token(last_token);
                self.abort_with(UnexpectedEndOfFile);
            }
        }
    }}

    /// Compilation: ( "&lt;spaces&gt;name" -- )
    ///
    /// Skip leading space delimiters. Parse name delimited by a space.
    /// Append the run-time semantics given below to the current definition.
    ///
    /// Run-time: ( -- char )
    ///
    /// Place `char`, the value of the first character of name, on the stack.
    primitive!{fn bracket_char(&mut self) {
        self.char();
        if self.last_error().is_some() {
            return;
        }
        let ch = self.s_stack().pop();
        self.compile_integer(ch);
    }}

    /// Run-time: ( char "ccc&lt;char&gt;" -- )
    ///
    /// Parse ccc delimited by the delimiter char.
    primitive!{fn parse(&mut self) {
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
    }}

    primitive!{fn imm_paren(&mut self) {
        self.s_stack().push(')' as isize);
        self.parse();
    }}

    primitive!{fn imm_backslash(&mut self) {
        self.state().source_index = match *self.input_buffer() {
            Some(ref buf) => buf.len(),
            None => 0,
        };
    }}

    primitive!{fn compile_token(&mut self) {
        let last_token = self.last_token().take().expect("token");
        match self.find(&last_token) {
            Some(found_index) => {
                self.set_last_token(last_token);
                let compilation_semantics = self.wordlist()[found_index].compilation_semantics;
                if !self.wordlist()[found_index].is_immediate() {
                    compilation_semantics(self, found_index);
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
                if done {
                    self.set_last_token(last_token);
                } else {
                    match self.output_buffer().as_mut() {
                        Some(buf) => {
                            write!(buf, "{} ", &last_token).expect("write");
                        }
                        None => {}
                    }
                    self.set_last_token(last_token);
                    self.abort_with(UndefinedWord);
                }
            }
        }
    }}

    primitive!{fn interpret_token(&mut self) {
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
                if done {
                    self.set_last_token(last_token);
                } else {
                    match self.output_buffer().as_mut() {
                        Some(buf) => {
                            write!(buf, "{} ", &last_token).expect("write");
                        }
                        None => {}
                    }
                    self.set_last_token(last_token);
                    self.abort_with(UndefinedWord);
                }
            }
        }
    }}

    primitive!{fn p_compiling(&mut self) {
        let value = if self.state().is_compiling {
            TRUE
        } else {
            FALSE
        };
        self.s_stack().push(value);
    }}

    primitive!{fn p_token_empty(&mut self) {
        let value = match self.last_token().as_ref() {
            Some(ref t) => if t.is_empty() { TRUE } else { FALSE },
            None => TRUE,
        };
        self.s_stack().push(value);
    }}

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

    primitive!{fn base(&mut self) {
        let base_addr = self.data_space().system_variables().base_addr();
        self.s_stack().push(base_addr as isize);
    }}

    fn evaluate_integer(&mut self, token: &str) {
        let base_addr = self.data_space().system_variables().base_addr();
        let default_base = self.data_space().get_isize(base_addr);
        match parser::quoted_char(&token.as_bytes()) {
            parser::IResult::Done(_bytes, c) => {
                if self.state().is_compiling {
                    self.compile_integer(c);
                } else {
                    self.s_stack().push(c);
                }
                return;
            }
            parser::IResult::Err(_) => {
                // Do nothing.
            }
        }
        match parser::base(&token.as_bytes(), default_base) {
            parser::IResult::Done(bytes, base) => match parser::sign(&bytes) {
                parser::IResult::Done(bytes, sign) => match parser::uint_in_base(&bytes, base) {
                    parser::IResult::Done(bytes, value) => {
                        if bytes.len() != 0 {
                            self.set_error(Some(UnsupportedOperation));
                        } else {
                            if self.state().is_compiling {
                                self.compile_integer(sign * value);
                            } else {
                                self.s_stack().push(sign * value);
                            }
                        }
                    }
                    parser::IResult::Err(e) => self.set_error(Some(e)),
                },
                parser::IResult::Err(e) => {
                    self.set_error(Some(e));
                }
            }
            parser::IResult::Err(e) => {
                self.set_error(Some(e));
            }
        }
    }

    /// Evaluate float.
    fn evaluate_float(&mut self, token: &str) {
        let significand_sign;
        let integer_part;
        let mut fraction_part = 0.0;
        let mut exponent_sign: isize = 0;
        let mut exponent_part: isize = 0;
        let mut failed = false;
        let mut bytes = token.as_bytes();

        match parser::sign(bytes) {
            parser::IResult::Done(input, value) => {
                significand_sign = value;
                bytes = input;
            }
            parser::IResult::Err(e) => {
                self.set_error(Some(e));
                return;
            }
        }

        let len_before = bytes.len();
        match parser::uint(bytes) {
            parser::IResult::Done(input, value) => {
                integer_part = value;
                bytes = input;
            }
            parser::IResult::Err(e) => {
                self.set_error(Some(e));
                return;
            }
        }
        if bytes.len() != len_before {
            match parser::fraction(bytes) {
                parser::IResult::Done(input, value) => {
                    fraction_part = value;
                    bytes = input;
                }
                parser::IResult::Err(e) => {
                    self.set_error(Some(e));
                    return;
                }
            }

            let len_before = bytes.len();
            match parser::ascii(bytes, b'E') {
                parser::IResult::Done(input, value) => {
                    if value {
                        match parser::sign(input) {
                            parser::IResult::Done(input, value) => {
                                exponent_sign = value;
                                bytes = input;
                            }
                            parser::IResult::Err(e) => {
                                self.set_error(Some(e));
                                return;
                            }
                        }
                        match parser::uint(bytes) {
                            parser::IResult::Done(input, value) => {
                                exponent_part = value;
                                bytes = input;
                            }
                            parser::IResult::Err(e) => {
                                self.set_error(Some(e));
                                return;
                            }
                        }
                    } else {
                        match parser::ascii(bytes, b'e') {
                            parser::IResult::Done(input, value) => {
                                if value {
                                    match parser::sign(input) {
                                        parser::IResult::Done(input, value) => {
                                            exponent_sign = value;
                                            bytes = input;
                                        }
                                        parser::IResult::Err(e) => {
                                            self.set_error(Some(e));
                                            return;
                                        }
                                    }
                                    match parser::uint(bytes) {
                                        parser::IResult::Done(input, value) => {
                                            exponent_part = value;
                                            bytes = input;
                                        }
                                        parser::IResult::Err(e) => {
                                            self.set_error(Some(e));
                                            return;
                                        }
                                    }
                                }
                            }
                            parser::IResult::Err(e) => {
                                self.set_error(Some(e));
                                return;
                            }
                        }
                    }
                }
                parser::IResult::Err(e) => {
                    self.set_error(Some(e));
                    return;
                }
            }

            if bytes.len() == len_before {
                failed = true;
            }
        } else {
            failed = true;
        }

        if bytes.len() != 0 {
            failed = true;
        }

        if failed {
            self.set_error(Some(UnsupportedOperation))
        } else {
            if self.references().idx_flit == 0 {
                self.set_error(Some(UnsupportedOperation));
            } else {
                let value = (significand_sign as f64) * (integer_part as f64 + fraction_part)
                    * ((10.0f64).powi((exponent_sign.wrapping_mul(exponent_part)) as i32) as f64);
                if self.state().is_compiling {
                    self.compile_float(value);
                } else {
                    self.f_stack().push(value);
                }
            }
        }
    }

    // -----------------------
    // High level definitions
    // -----------------------

    primitive!{fn nest(&mut self) {
        let rlen = self.r_stack().len.wrapping_add(1);
        self.r_stack().len = rlen;
        self.r_stack()[rlen.wrapping_sub(1)] = self.state().instruction_pointer as isize;
        let wp = self.state().word_pointer;
        self.state().instruction_pointer = self.wordlist()[wp].dfa();
    }}

    primitive!{fn p_var(&mut self) {
        let wp = self.state().word_pointer;
        let dfa = self.wordlist()[wp].dfa() as isize;
        self.s_stack().push(dfa);
    }}

    primitive!{fn p_const(&mut self) {
        let wp = self.state().word_pointer;
        let dfa = self.wordlist()[wp].dfa();
        let value = self.data_space().get_i32(dfa) as isize;
        self.s_stack().push(value);
    }}

fn define(&mut self, action: primitive!{fn(&mut Self)},
compilation_semantics: fn(&mut Self, usize)){
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
            self.data_space().align();
            self.code_space().align();
            let word = Word::new(
                symbol,
                action,
                compilation_semantics,
                self.data_space().len(),
                self.code_space().here(),
            );
            let len = self.wordlist().len();
            self.set_last_definition(len);
            self.wordlist_mut().push(word);
            self.set_last_token(last_token);
        }
    }

    primitive!{fn colon(&mut self) {
        self.define(Core::nest, Core::compile_nest);
        if self.last_error().is_none() {
            let def = self.last_definition();
            self.compile_nest_code(def);
            self.wordlist_mut()[def].set_hidden(true);
            self.right_bracket();
        }
    }}

    primitive!{fn semicolon(&mut self) {
        if self.last_definition() != 0 {
            if self.c_stack().len != 0 {
                self.abort_with(ControlStructureMismatch);
            } else {
                let idx = self.references().idx_exit;
                let compile = self.wordlist()[idx].compilation_semantics;
                compile(self, idx);
                let def = self.last_definition();
                self.wordlist_mut()[def].set_hidden(false);
            }
        }
        self.left_bracket();
    }}

    primitive!{fn create(&mut self) {
        self.define(Core::p_var, Core::compile_var);
    }}

    primitive!{fn variable(&mut self) {
        self.define(Core::p_var, Core::compile_var);
        if self.last_error().is_none() {
            self.data_space().compile_i32(0);
        }
    }}

    primitive!{fn constant(&mut self) {
        let v = self.s_stack().pop();
        self.define(Core::p_const, Core::compile_const);
        if self.last_error().is_none() {
            self.data_space().compile_i32(v as i32);
        }
    }}

    primitive!{fn unmark(&mut self) {
        let wp = self.state().word_pointer;
        let dfa;
        let cfa;
        let symbol;
        {
            let w = &self.wordlist()[wp];
            dfa = w.dfa();
            cfa = w.cfa();
            symbol = w.symbol();
        }
        self.data_space().truncate(dfa);
        self.code_space().truncate(cfa);
        self.wordlist_mut().truncate(wp);
        self.symbols_mut().truncate(symbol.id);
    }}

    primitive!{fn marker(&mut self) {
        self.define(Core::unmark, Core::compile_unmark);
    }}

    // -----------
    // Primitives
    // -----------

    /// Run-time: ( -- )
    ///
    /// No operation
    primitive!{fn noop(&mut self) {
        // Do nothing
    }}

    /// Run-time: ( -- true )
    ///
    /// Return a true flag, a single-cell value with all bits set.
    primitive!{fn p_true(&mut self) {
        self.s_stack().push(TRUE);
    }}

    /// Run-time: ( -- false )
    ///
    /// Return a false flag.
    primitive!{fn p_false(&mut self) {
        self.s_stack().push(FALSE);
    }}

    /// Run-time: (c-addr1 -- c-addr2 )
    ///
    ///Add the size in address units of a character to `c-addr1`, giving `c-addr2`.
    primitive!{fn char_plus(&mut self) {
        let v = self.s_stack().pop();
        self.s_stack().push(v + mem::size_of::<u8>() as isize);
    }}

    /// Run-time: (n1 -- n2 )
    ///
    /// `n2` is the size in address units of `n1` characters.
    primitive!{fn chars(&mut self) {
        let v = self.s_stack().pop();
        self.s_stack().push(v * mem::size_of::<u8>() as isize);
    }}

    /// Run-time: (a-addr1 -- a-addr2 )
    ///
    /// Add the size in address units of a cell to `a-addr1`, giving `a-addr2`.
    primitive!{fn cell_plus(&mut self) {
        let v = self.s_stack().pop();
        self.s_stack().push(v + mem::size_of::<i32>() as isize);
    }}

    /// Run-time: (n1 -- n2 )
    ///
    /// `n2` is the size in address units of `n1` cells.
    primitive!{fn cells(&mut self) {
        let v = self.s_stack().pop();
        self.s_stack().push(v * mem::size_of::<i32>() as isize);
    }}

    primitive!{fn swap(&mut self) {
        let slen = self.s_stack().len;
        let t = self.s_stack()[slen.wrapping_sub(1)];
        let n = self.s_stack()[slen.wrapping_sub(2)];
        self.s_stack()[slen.wrapping_sub(1)] = n;
        self.s_stack()[slen.wrapping_sub(2)] = t;
    }}

    primitive!{fn dup(&mut self) {
        let slen = self.s_stack().len.wrapping_add(1);
        self.s_stack().len = slen;
        self.s_stack()[slen.wrapping_sub(1)] = self.s_stack()[slen.wrapping_sub(2)];
    }}

    primitive!{fn p_drop(&mut self) {
        let slen = self.s_stack().len.wrapping_sub(1);
        self.s_stack().len = slen;
    }}

    primitive!{fn pop_s_stack(&mut self) -> isize {
        let slen = self.s_stack().len.wrapping_sub(1);
        let t = self.s_stack()[slen];
        self.s_stack().len = slen;
        t
    }}

    primitive!{fn nip(&mut self) {
        let slen = self.s_stack().len.wrapping_sub(1);
        let t = self.s_stack()[slen];
        self.s_stack().len = slen;
        self.s_stack()[slen.wrapping_sub(1)] = t;
    }}

    primitive!{fn over(&mut self) {
        let slen = self.s_stack().len.wrapping_add(1);
        self.s_stack().len = slen;
        self.s_stack()[slen.wrapping_sub(1)] = self.s_stack()[slen.wrapping_sub(3)];
    }}

    primitive!{fn rot(&mut self) {
        let slen = self.s_stack().len;
        let t = self.s_stack()[slen.wrapping_sub(1)];
        let n = self.s_stack()[slen.wrapping_sub(2)];
        self.s_stack()[slen.wrapping_sub(1)] = self.s_stack()[slen.wrapping_sub(3)];
        self.s_stack()[slen.wrapping_sub(2)] = t;
        self.s_stack()[slen.wrapping_sub(3)] = n;
    }}

    primitive!{fn minus_rot(&mut self) {
        let slen = self.s_stack().len;
        let t = self.s_stack()[slen.wrapping_sub(1)];
        let n = self.s_stack()[slen.wrapping_sub(2)];
        self.s_stack()[slen.wrapping_sub(2)] = self.s_stack()[slen.wrapping_sub(3)];
        self.s_stack()[slen.wrapping_sub(3)] = t;
        self.s_stack()[slen.wrapping_sub(1)] = n;
    }}

    primitive!{fn two_drop(&mut self) {
        let slen = self.s_stack().len.wrapping_sub(2);
        self.s_stack().len = slen;
    }}

    primitive!{fn two_dup(&mut self) {
        let slen = self.s_stack().len.wrapping_add(2);
        self.s_stack().len = slen;
        self.s_stack()[slen.wrapping_sub(1)] = self.s_stack()[slen.wrapping_sub(3)];
        self.s_stack()[slen.wrapping_sub(2)] = self.s_stack()[slen.wrapping_sub(4)];
    }}

    primitive!{fn two_swap(&mut self) {
        let slen = self.s_stack().len;
        let t = self.s_stack()[slen.wrapping_sub(1)];
        let n = self.s_stack()[slen.wrapping_sub(2)];
        self.s_stack()[slen.wrapping_sub(1)] = self.s_stack()[slen.wrapping_sub(3)];
        self.s_stack()[slen.wrapping_sub(2)] = self.s_stack()[slen.wrapping_sub(4)];
        self.s_stack()[slen.wrapping_sub(3)] = t;
        self.s_stack()[slen.wrapping_sub(4)] = n;
    }}

    primitive!{fn two_over(&mut self) {
        let slen = self.s_stack().len.wrapping_add(2);
        self.s_stack().len = slen;
        self.s_stack()[slen.wrapping_sub(1)] = self.s_stack()[slen.wrapping_sub(5)];
        self.s_stack()[slen.wrapping_sub(2)] = self.s_stack()[slen.wrapping_sub(6)];
    }}

    primitive!{fn depth(&mut self) {
        let len = self.s_stack().len;
        self.s_stack().push(len as isize);
    }}

    primitive!{fn one_plus(&mut self) {
        let slen = self.s_stack().len;
        let t = self.s_stack()[slen.wrapping_sub(1)];
        self.s_stack()[slen.wrapping_sub(1)] = t.wrapping_add(1);
    }}

    primitive!{fn one_minus(&mut self) {
        let slen = self.s_stack().len;
        let t = self.s_stack()[slen.wrapping_sub(1)];
        self.s_stack()[slen.wrapping_sub(1)] = t.wrapping_sub(1);
    }}

    primitive!{fn plus(&mut self) {
        let slen = self.s_stack().len;
        let t = self.s_stack()[slen.wrapping_sub(1)];
        let n = self.s_stack()[slen.wrapping_sub(2)];
        self.s_stack()[slen.wrapping_sub(2)] = n.wrapping_add(t);
        self.s_stack().len = slen.wrapping_sub(1);
    }}

    primitive!{fn minus(&mut self) {
        let slen = self.s_stack().len;
        let t = self.s_stack()[slen.wrapping_sub(1)];
        let n = self.s_stack()[slen.wrapping_sub(2)];
        self.s_stack()[slen.wrapping_sub(2)] = n.wrapping_sub(t);
        self.s_stack().len = slen.wrapping_sub(1);
    }}

    primitive!{fn star(&mut self) {
        let slen = self.s_stack().len;
        let t = self.s_stack()[slen.wrapping_sub(1)];
        let n = self.s_stack()[slen.wrapping_sub(2)];
        self.s_stack()[slen.wrapping_sub(2)] = n.wrapping_mul(t);
        self.s_stack().len = slen.wrapping_sub(1);
    }}

    primitive!{fn slash(&mut self) {
        let slen = self.s_stack().len;
        let t = self.s_stack()[slen.wrapping_sub(1)];
        let n = self.s_stack()[slen.wrapping_sub(2)];
        if t == 0 {
            self.abort_with(DivisionByZero);
        } else {
            self.s_stack()[slen.wrapping_sub(2)] = n.wrapping_div(t);
            self.s_stack().len = slen.wrapping_sub(1);
        }
    }}

    primitive!{fn p_mod(&mut self) {
        let slen = self.s_stack().len;
        let t = self.s_stack()[slen.wrapping_sub(1)];
        let n = self.s_stack()[slen.wrapping_sub(2)];
        if t == 0 {
            self.abort_with(DivisionByZero);
        } else {
            self.s_stack()[slen.wrapping_sub(2)] = n.wrapping_rem(t);
            self.s_stack().len = slen.wrapping_sub(1);
        }
    }}

    primitive!{fn slash_mod(&mut self) {
        let slen = self.s_stack().len;
        let t = self.s_stack()[slen.wrapping_sub(1)];
        let n = self.s_stack()[slen.wrapping_sub(2)];
        if t == 0 {
            self.abort_with(DivisionByZero);
        } else {
            self.s_stack()[slen.wrapping_sub(2)] = n.wrapping_rem(t);
            self.s_stack()[slen.wrapping_sub(1)] = n.wrapping_div(t);
        }
    }}

    primitive!{fn abs(&mut self) {
        let t = self.s_stack().pop();
        self.s_stack().push(t.wrapping_abs());
    }}

    primitive!{fn negate(&mut self) {
        let t = self.s_stack().pop();
        self.s_stack().push(t.wrapping_neg());
    }}

    primitive!{fn zero_less(&mut self) {
        let t = self.s_stack().pop();
        self.s_stack().push(if t < 0 { TRUE } else { FALSE });
    }}

    primitive!{fn zero_equals(&mut self) {
        let t = self.s_stack().pop();
        self.s_stack().push(if t == 0 { TRUE } else { FALSE });
    }}

    primitive!{fn zero_greater(&mut self) {
        let t = self.s_stack().pop();
        self.s_stack().push(if t > 0 { TRUE } else { FALSE });
    }}

    primitive!{fn zero_not_equals(&mut self) {
        let t = self.s_stack().pop();
        self.s_stack().push(if t == 0 { FALSE } else { TRUE });
    }}

    primitive!{fn equals(&mut self) {
        let (n, t) = self.s_stack().pop2();
        self.s_stack().push(if t == n { TRUE } else { FALSE });
    }}

    primitive!{fn less_than(&mut self) {
        let (n, t) = self.s_stack().pop2();
        self.s_stack().push(if n < t { TRUE } else { FALSE });
    }}

    primitive!{fn greater_than(&mut self) {
        let (n, t) = self.s_stack().pop2();
        self.s_stack().push(if n > t { TRUE } else { FALSE });
    }}

    primitive!{fn not_equals(&mut self) {
        let (n, t) = self.s_stack().pop2();
        self.s_stack().push(if n == t { FALSE } else { TRUE });
    }}

    /// `within` ( n1 n2 n3 -- flag )  true if n2 <= n1 and n1 <= n3.
    ///
    /// Note: implmenetation incompatible with Forth 2012 standards
    /// when n2 > n3.
    primitive!{fn within(&mut self) {
        let (x1, x2, x3) = self.s_stack().pop3();
        self.s_stack()
            .push(if x2 <= x1 && x1 < x3 { TRUE } else { FALSE });
    }}

    primitive!{fn invert(&mut self) {
        let t = self.s_stack().pop();
        self.s_stack().push(!t);
    }}

    primitive!{fn and(&mut self) {
        let (n, t) = self.s_stack().pop2();
        self.s_stack().push(t & n);
    }}

    primitive!{fn or(&mut self) {
        let (n, t) = self.s_stack().pop2();
        self.s_stack().push(t | n);
    }}

    primitive!{fn xor(&mut self) {
        let (n, t) = self.s_stack().pop2();
        self.s_stack().push(t ^ n);
    }}

    /// Run-time: ( x1 u -- x2 )
    ///
    /// Perform a logical left shift of `u` bit-places on `x1`, giving `x2`. Put
    /// zeroes into the least significant bits vacated by the shift. An
    /// ambiguous condition exists if `u` is greater than or equal to the number
    /// of bits in a cell.
    primitive!{fn lshift(&mut self) {
        let (n, t) = self.s_stack().pop2();
        self.s_stack().push(n.wrapping_shl(t as u32));
    }}

    /// Run-time: ( x1 u -- x2 )
    ///
    /// Perform a logical right shift of `u` bit-places on `x1`, giving `x2`. Put
    /// zeroes into the most significant bits vacated by the shift. An
    /// ambiguous condition exists if `u` is greater than or equal to the number
    /// of bits in a cell.
    primitive!{fn rshift(&mut self) {
        let (n, t) = self.s_stack().pop2();
        self.s_stack()
            .push(((n as usize).wrapping_shr(t as u32)) as isize);
    }}

    /// Interpretation: Interpretation semantics for this word are undefined.
    ///
    /// Execution: ( -- ) ( R: nest-sys -- )
    /// Return control to the calling definition specified by `nest-sys`.
    /// Before executing `EXIT` within a do-loop, a program shall discard the
    /// loop-control parameters by executing `UNLOOP`.
    ///
    primitive!{fn exit(&mut self) {
        let rlen = self.r_stack().len.wrapping_sub(1);
        self.state().instruction_pointer = self.r_stack()[rlen] as usize;
        self.r_stack().len = rlen;
    }}

    /// Run-time: ( a-addr -- x )
    ///
    /// `x` is the value stored at `a-addr`.
    primitive!{fn fetch(&mut self) {
        let t = self.s_stack().pop();
        if (t as usize + mem::size_of::<i32>()) <= self.data_space().capacity() {
            let value = self.data_space().get_i32(t as usize) as isize;
            self.s_stack().push(value);
        } else {
            self.abort_with(InvalidMemoryAddress);
        }
    }}

    /// Run-time: ( x a-addr -- )
    ///
    /// Store `x` at `a-addr`.
    primitive!{fn store(&mut self) {
        let (n, t) = self.s_stack().pop2();
        if (t as usize + mem::size_of::<i32>()) <= self.data_space().capacity() {
            self.data_space().put_i32(n as i32, t as usize);
        } else {
            self.abort_with(InvalidMemoryAddress);
        }
    }}

    /// Run-time: ( c-addr -- char )
    ///
    /// Fetch the character stored at `c-addr`. When the cell size is greater than
    /// character size, the unused high-order bits are all zeroes.
    primitive!{fn c_fetch(&mut self) {
        let t = self.s_stack().pop();
        if (t as usize + mem::size_of::<u8>()) <= self.data_space().capacity() {
            let value = self.data_space().get_u8(t as usize) as isize;
            self.s_stack().push(value);
        } else {
            self.abort_with(InvalidMemoryAddress);
        }
    }}

    /// Run-time: ( char c-addr -- )
    ///
    /// Store `char` at `c-addr`. When character size is smaller than cell size,
    /// only the number of low-order bits corresponding to character size are
    /// transferred.
    primitive!{fn c_store(&mut self) {
        let (n, t) = self.s_stack().pop2();
        if (t as usize + mem::size_of::<u8>()) <= self.data_space().capacity() {
            self.data_space().put_u8(n as u8, t as usize);
        } else {
            self.abort_with(InvalidMemoryAddress);
        }
    }}

    /// Run-time: ( "<spaces>name" -- xt )
    ///
    /// Skip leading space delimiters. Parse name delimited by a space. Find
    /// `name` and return `xt`, the execution token for name. An ambiguous
    /// condition exists if name is not found.
    primitive!{fn tick(&mut self) {
        self.parse_word();
        let last_token = self.last_token().take().expect("last token");
        if last_token.is_empty() {
            self.set_last_token(last_token);
            self.abort_with(UnexpectedEndOfFile);
        } else {
            match self.find(&last_token) {
                Some(found_index) => {
                    self.s_stack().push(found_index as isize);
                    self.set_last_token(last_token);
                }
                None => {
                    self.set_last_token(last_token);
                    self.abort_with(UndefinedWord);
                }
            }
        }
    }}

    /// Run-time: ( i*x xt -- j*x )
    ///
    /// Remove `xt` from the stack and perform the semantics identified by it.
    /// Other stack effects are due to the word `EXECUTE`d.
    primitive!{fn execute(&mut self) {
        let t = self.s_stack().pop();
        self.execute_word(t as usize);
    }}

    /// Run-time: ( -- addr )
    ///
    /// `addr` is the data-space pointer.
    primitive!{fn here(&mut self) {
        let len = self.data_space().len() as isize;
        self.s_stack().push(len);
    }}

    /// Run-time: ( n -- )
    ///
    /// If `n` is greater than zero, reserve n address units of data space. If `n`
    /// is less than zero, release `|n|` address units of data space. If `n` is
    /// zero, leave the data-space pointer unchanged.
    primitive!{fn allot(&mut self) {
        let v = self.s_stack().pop();
        self.data_space().allot(v);
    }}

    /// Run-time: ( x -- )
    ///
    /// Reserve one cell of data space and store `x` in the cell. If the
    /// data-space pointer is aligned when `,` begins execution, it will remain
    /// aligned when `,` finishes execution. An ambiguous condition exists if the
    /// data-space pointer is not aligned prior to execution of `,`.
    primitive!{fn comma(&mut self) {
        let v = self.s_stack().pop();
        self.data_space().compile_i32(v as i32);
    }}

    primitive!{fn p_to_r(&mut self) {
        let slen = self.s_stack().len;
        let rlen = self.r_stack().len.wrapping_add(1);
        self.r_stack().len = rlen;
        self.r_stack()[rlen.wrapping_sub(1)] = self.s_stack()[slen.wrapping_sub(1)];
        self.s_stack().len = slen.wrapping_sub(1);
    }}

    primitive!{fn r_from(&mut self) {
        let slen = self.s_stack().len.wrapping_add(1);
        let rlen = self.r_stack().len;
        self.s_stack().len = slen;
        self.s_stack()[slen.wrapping_sub(1)] = self.r_stack()[rlen.wrapping_sub(1)];
        self.r_stack().len = rlen.wrapping_sub(1);
    }}

    primitive!{fn r_fetch(&mut self) {
        let slen = self.s_stack().len.wrapping_add(1);
        let rlen = self.r_stack().len;
        self.s_stack().len = slen;
        self.s_stack()[slen.wrapping_sub(1)] = self.r_stack()[rlen.wrapping_sub(1)];
    }}

    primitive!{fn two_to_r(&mut self) {
        let slen = self.s_stack().len;
        let rlen = self.r_stack().len.wrapping_add(2);
        self.r_stack().len = rlen;
        self.r_stack()[rlen.wrapping_sub(2)] = self.s_stack()[slen.wrapping_sub(2)];
        self.r_stack()[rlen.wrapping_sub(1)] = self.s_stack()[slen.wrapping_sub(1)];
        self.s_stack().len = slen.wrapping_sub(2);
    }}

    primitive!{fn two_r_from(&mut self) {
        let slen = self.s_stack().len.wrapping_add(2);
        let rlen = self.r_stack().len;
        self.s_stack().len = slen;
        self.s_stack()[slen.wrapping_sub(2)] = self.r_stack()[rlen.wrapping_sub(2)];
        self.s_stack()[slen.wrapping_sub(1)] = self.r_stack()[rlen.wrapping_sub(1)];
        self.r_stack().len = rlen.wrapping_sub(2);
    }}

    primitive!{fn two_r_fetch(&mut self) {
        let slen = self.s_stack().len.wrapping_add(2);
        let rlen = self.r_stack().len;
        self.s_stack().len = slen;
        self.s_stack()[slen.wrapping_sub(2)] = self.r_stack()[rlen.wrapping_sub(2)];
        self.s_stack()[slen.wrapping_sub(1)] = self.r_stack()[rlen.wrapping_sub(1)];
    }}

    // ----------------
    // Error handlling
    // ----------------

    primitive!{fn check_stacks(&mut self) {
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
    }}

    primitive!{fn handler_store(&mut self) {
        let t = self.s_stack().pop();
        self.set_handler(t as usize);
    }}

    primitive!{fn p_error_q(&mut self) {
        let value = if self.last_error().is_some() {
            TRUE
        } else {
            FALSE
        };
        self.s_stack().push(value);
    }}

    primitive!{fn p_handle_error(&mut self) {
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
    }}

    /// Clear data and floating point stacks.
    /// Called by VM's client upon Abort.
    primitive!{fn clear_stacks(&mut self) {
        self.s_stack().reset();
        self.f_stack().reset();
    }}

    /// Reset VM, do not clear data stack and floating point stack.
    /// Called by VM's client upon Quit.
    primitive!{fn reset(&mut self) {
        self.r_stack().len = 0;
        self.c_stack().len = 0;
        if let Some(ref mut buf) = *self.input_buffer() {
            buf.clear()
        }
        self.state().source_index = 0;
        self.left_bracket();
        self.set_error(None);
    }}

    primitive!{fn _regs(&mut self) -> &mut [usize; 2] {
        self.regs()
    }}

    #[cfg(all(feature = "subroutine-threaded", target_arch = "x86"))]
    fn compile_reset(&mut self, _: usize) {
        //      ; 判斷之前是否執行過 set_regs。
        //      89 f1           mov    %esi,%ecx
        //      e8 xx xx xx xx  call   _regs()
        //      8b 10           mov    (%eax),%edx
        //      85 d2           test   %edx,%edx
        //      74 04           je     set_regs
        //      ; 若則執行過，重設 %esp 。
        //      89 d4           mov    %edx,%esp
        //      eb 02           jmp    call_reset
        // set_regs:
        //     ; 記住 quit 的 %esp 。
        //      89 20           mov %esp, (%eax)
        // call_reset:
        //      89 f1           mov %esi,%ecx
        //      e8 xx xx xx xx  call reset
        //      89 f1           mov %esi,%ecx
        //      e8 xx xx xx xx  call clear_stacks
        self.code_space().compile_u8(0x89);
        self.code_space().compile_u8(0xf1);
        self.code_space().compile_u8(0xe8);
        self.code_space().compile_relative(Self::_regs as usize);
        self.code_space().compile_u8(0x8b);
        self.code_space().compile_u8(0x10);
        self.code_space().compile_u8(0x85);
        self.code_space().compile_u8(0xd2);
        self.code_space().compile_u8(0x74);
        self.code_space().compile_u8(0x04);
        self.code_space().compile_u8(0x89);
        self.code_space().compile_u8(0xd4);
        self.code_space().compile_u8(0xeb);
        self.code_space().compile_u8(0x02);
        self.code_space().compile_u8(0x89);
        self.code_space().compile_u8(0x20);
        self.code_space().compile_u8(0x89);
        self.code_space().compile_u8(0xf1);
        self.code_space().compile_u8(0xe8);
        self.code_space().compile_relative(Self::reset as usize);
        self.code_space().compile_u8(0x89);
        self.code_space().compile_u8(0xf1);
        self.code_space().compile_u8(0xe8);
        self.code_space()
            .compile_relative(Self::clear_stacks as usize);
    }

    /// Abort the inner loop with an exception, reset VM and clears stacks.
    fn abort_with(&mut self, e: Exception) {
        self.clear_stacks();
        self.set_error(Some(e));
        let h = self.handler();
        self.execute_word(h);
    }

    /// Abort the inner loop with an exception, reset VM and clears stacks.
    primitive!{fn abort(&mut self) {
        self.abort_with(Abort);
    }}

}

#[cfg(test)]
mod tests {
    extern crate test;
    use self::test::Bencher;
    use super::Core;
    use exception::Exception::{Abort, ControlStructureMismatch, InterpretingACompileOnlyWord,
                               InvalidMemoryAddress, ReturnStackUnderflow, StackUnderflow,
                               UndefinedWord, UnexpectedEndOfFile};
    use loader::HasLoader;
    use std::mem;
    use vm::VM;

    #[bench]
    fn bench_noop(b: &mut Bencher) {
        let vm = &mut VM::new(16, 16);
        b.iter(|| vm.noop());
    }

    #[test]
    fn test_find() {
        let vm = &mut VM::new(16, 16);
        assert!(vm.find("").is_none());
        assert!(vm.find("word-not-exist").is_none());
        vm.find("noop").expect("noop not found");
    }

    #[bench]
    fn bench_find_word_not_exist(b: &mut Bencher) {
        let vm = &mut VM::new(16, 16);
        b.iter(|| vm.find("unknown"));
    }

    #[bench]
    fn bench_find_word_at_beginning_of_wordlist(b: &mut Bencher) {
        let vm = &mut VM::new(16, 16);
        b.iter(|| vm.find("noop"));
    }

    #[bench]
    fn bench_inner_interpreter_without_nest(b: &mut Bencher) {
        let vm = &mut VM::new(16, 16);
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
        let vm = &mut VM::new(16, 16);
        vm.p_drop();
        vm.check_stacks();
        assert_eq!(vm.last_error(), Some(StackUnderflow));
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(1);
        vm.p_drop();
        vm.check_stacks();
        assert!(vm.s_stack().is_empty());
        assert!(vm.last_error().is_none());
    }

    #[bench]
    fn bench_drop(b: &mut Bencher) {
        let vm = &mut VM::new(16, 16);
        vm.s_stack().push(1);
        b.iter(|| {
            vm.p_drop();
            vm.s_stack().push(1);
        });
    }

    #[test]
    fn test_nip() {
        let vm = &mut VM::new(16, 16);
        vm.nip();
        vm.check_stacks();
        assert_eq!(vm.last_error(), Some(StackUnderflow));
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(1);
        vm.nip();
        vm.check_stacks();
        assert_eq!(vm.last_error(), Some(StackUnderflow));
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(1);
        vm.s_stack().push(2);
        vm.nip();
        vm.check_stacks();
        assert!(vm.last_error().is_none());
        assert!(vm.s_stack().len() == 1);
        assert!(vm.s_stack().last() == Some(2));
    }

    #[bench]
    fn bench_nip(b: &mut Bencher) {
        let vm = &mut VM::new(16, 16);
        vm.s_stack().push(1);
        vm.s_stack().push(1);
        b.iter(|| {
            vm.nip();
            vm.s_stack().push(1);
        });
    }

    #[test]
    fn test_swap() {
        let vm = &mut VM::new(16, 16);
        vm.swap();
        vm.check_stacks();
        // check_stacks() cannot detect this kind of underflow.
        // assert_eq!(vm.last_error(), Some(StackUnderflow));
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(1);
        vm.swap();
        vm.check_stacks();
        assert_eq!(vm.last_error(), Some(StackUnderflow));
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(1);
        vm.s_stack().push(2);
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
        let vm = &mut VM::new(16, 16);
        vm.s_stack().push(1);
        vm.s_stack().push(2);
        b.iter(|| vm.swap());
    }

    #[test]
    fn test_dup() {
        let vm = &mut VM::new(16, 16);
        vm.dup();
        vm.check_stacks();
        // check_stacks can not detect this underflow();
        //        assert_eq!(vm.last_error(), Some(StackUnderflow));
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(1);
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
        let vm = &mut VM::new(16, 16);
        vm.s_stack().push(1);
        b.iter(|| {
            vm.dup();
            vm.s_stack().pop();
        });
    }

    #[test]
    fn test_over() {
        let vm = &mut VM::new(16, 16);
        vm.over();
        vm.check_stacks();
        // check_stacks() cannot detect stack underflow of over().
        // assert_eq!(vm.last_error(), Some(StackUnderflow));
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(1);
        vm.check_stacks();
        vm.over();
        // check_stacks() cannot detect stack underflow of over().
        // assert_eq!(vm.last_error(), Some(StackUnderflow));
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(1);
        vm.s_stack().push(2);
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
        let vm = &mut VM::new(16, 16);
        vm.s_stack().push(1);
        vm.s_stack().push(2);
        b.iter(|| {
            vm.over();
            vm.s_stack().pop();
        });
    }

    #[test]
    fn test_rot() {
        let vm = &mut VM::new(16, 16);
        vm.rot();
        vm.check_stacks();
        // check_stacks() cannot detect this kind of stack underflow of over().
        // assert_eq!(vm.last_error(), Some(StackUnderflow));
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(1);
        vm.rot();
        vm.check_stacks();
        assert_eq!(vm.last_error(), Some(StackUnderflow));
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(1);
        vm.s_stack().push(2);
        vm.rot();
        vm.check_stacks();
        assert_eq!(vm.last_error(), Some(StackUnderflow));
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(1);
        vm.s_stack().push(2);
        vm.s_stack().push(3);
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
        let vm = &mut VM::new(16, 16);
        vm.s_stack().push(1);
        vm.s_stack().push(2);
        vm.s_stack().push(3);
        b.iter(|| vm.rot());
    }

    #[test]
    fn test_2drop() {
        let vm = &mut VM::new(16, 16);
        vm.two_drop();
        assert!(vm.s_stack().underflow());
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(1);
        vm.two_drop();
        assert!(vm.s_stack().underflow());
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(1);
        vm.s_stack().push(2);
        vm.two_drop();
        assert!(!vm.s_stack().underflow());
        assert!(!vm.s_stack().overflow());
        assert!(vm.last_error().is_none());
        assert!(vm.s_stack().is_empty());
    }

    #[bench]
    fn bench_2drop(b: &mut Bencher) {
        let vm = &mut VM::new(16, 16);
        b.iter(|| {
            vm.s_stack().push(1);
            vm.s_stack().push(2);
            vm.two_drop();
        });
    }

    #[test]
    fn test_2dup() {
        let vm = &mut VM::new(16, 16);
        vm.two_dup();
        assert!(!vm.s_stack().underflow());
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(1);
        vm.two_dup();
        assert!(!vm.s_stack().underflow());
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(1);
        vm.s_stack().push(2);
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
        let vm = &mut VM::new(16, 16);
        vm.s_stack().push(1);
        vm.s_stack().push(2);
        b.iter(|| {
            vm.two_dup();
            vm.two_drop();
        });
    }

    #[test]
    fn test_2swap() {
        let vm = &mut VM::new(16, 16);
        vm.two_swap();
        assert!(!vm.s_stack().underflow());
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(1);
        vm.two_swap();
        assert!(!vm.s_stack().underflow());
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(1);
        vm.s_stack().push(2);
        vm.two_swap();
        assert!(vm.s_stack().underflow());
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(1);
        vm.s_stack().push(2);
        vm.s_stack().push(3);
        vm.two_swap();
        assert!(vm.s_stack().underflow());
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(1);
        vm.s_stack().push(2);
        vm.s_stack().push(3);
        vm.s_stack().push(4);
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
        let vm = &mut VM::new(16, 16);
        vm.s_stack().push(1);
        vm.s_stack().push(2);
        vm.s_stack().push(3);
        vm.s_stack().push(4);
        b.iter(|| vm.two_swap());
    }

    #[test]
    fn test_2over() {
        let vm = &mut VM::new(16, 16);
        vm.two_over();
        assert!(!vm.s_stack().underflow());
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(1);
        vm.two_over();
        assert!(!vm.s_stack().underflow());
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(1);
        vm.s_stack().push(2);
        vm.two_over();
        assert!(!vm.s_stack().underflow());
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(1);
        vm.s_stack().push(2);
        vm.s_stack().push(3);
        vm.two_over();
        assert!(!vm.s_stack().underflow());
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(1);
        vm.s_stack().push(2);
        vm.s_stack().push(3);
        vm.s_stack().push(4);
        vm.two_over();
        assert!(!vm.s_stack().underflow());
        assert!(!vm.s_stack().overflow());
        assert!(vm.last_error().is_none());
        assert_eq!(vm.s_stack().len(), 6);
        assert_eq!(vm.s_stack().as_slice(), [1, 2, 3, 4, 1, 2]);
    }

    #[bench]
    fn bench_2over(b: &mut Bencher) {
        let vm = &mut VM::new(16, 16);
        vm.s_stack().push(1);
        vm.s_stack().push(2);
        vm.s_stack().push(3);
        vm.s_stack().push(4);
        b.iter(|| {
            vm.two_over();
            vm.two_drop();
        });
    }

    #[test]
    fn test_depth() {
        let vm = &mut VM::new(16, 16);
        vm.depth();
        vm.depth();
        vm.depth();
        assert_eq!(vm.s_stack().as_slice(), [0, 1, 2]);
    }

    #[test]
    fn test_one_plus() {
        let vm = &mut VM::new(16, 16);
        vm.one_plus();
        vm.check_stacks();
        assert_eq!(vm.last_error(), Some(StackUnderflow));
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(1);
        vm.one_plus();
        vm.check_stacks();
        assert!(vm.last_error().is_none());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), 2);
    }

    #[bench]
    fn bench_one_plus(b: &mut Bencher) {
        let vm = &mut VM::new(16, 16);
        vm.s_stack().push(0);
        b.iter(|| {
            vm.one_plus();
        });
    }

    #[test]
    fn test_one_minus() {
        let vm = &mut VM::new(16, 16);
        vm.one_minus();
        vm.check_stacks();
        assert_eq!(vm.last_error(), Some(StackUnderflow));
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(2);
        vm.one_minus();
        vm.check_stacks();
        assert!(vm.last_error().is_none());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), 1);
    }

    #[bench]
    fn bench_one_minus(b: &mut Bencher) {
        let vm = &mut VM::new(16, 16);
        vm.s_stack().push(0);
        b.iter(|| {
            vm.one_minus();
        });
    }

    #[test]
    fn test_minus() {
        let vm = &mut VM::new(16, 16);
        vm.minus();
        vm.check_stacks();
        assert_eq!(vm.last_error(), Some(StackUnderflow));
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(5);
        vm.minus();
        vm.check_stacks();
        assert_eq!(vm.last_error(), Some(StackUnderflow));
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(5);
        vm.s_stack().push(7);
        vm.minus();
        vm.check_stacks();
        assert_eq!(vm.last_error(), None);
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), -2);
    }

    #[bench]
    fn bench_minus(b: &mut Bencher) {
        let vm = &mut VM::new(16, 16);
        vm.s_stack().push(0);
        b.iter(|| {
            vm.dup();
            vm.minus();
        });
    }

    #[test]
    fn test_plus() {
        let vm = &mut VM::new(16, 16);
        vm.plus();
        vm.check_stacks();
        assert_eq!(vm.last_error(), Some(StackUnderflow));
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(5);
        vm.plus();
        vm.check_stacks();
        assert_eq!(vm.last_error(), Some(StackUnderflow));
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(5);
        vm.s_stack().push(7);
        vm.plus();
        vm.check_stacks();
        assert!(vm.last_error().is_none());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), 12);
    }

    #[bench]
    fn bench_plus(b: &mut Bencher) {
        let vm = &mut VM::new(16, 16);
        vm.s_stack().push(1);
        b.iter(|| {
            vm.dup();
            vm.plus();
        });
    }

    #[test]
    fn test_star() {
        let vm = &mut VM::new(16, 16);
        vm.star();
        vm.check_stacks();
        assert_eq!(vm.last_error(), Some(StackUnderflow));
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(5);
        vm.star();
        vm.check_stacks();
        assert_eq!(vm.last_error(), Some(StackUnderflow));
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(5);
        vm.s_stack().push(7);
        vm.star();
        vm.check_stacks();
        assert!(vm.last_error().is_none());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), 35);
    }

    #[bench]
    fn bench_star(b: &mut Bencher) {
        let vm = &mut VM::new(16, 16);
        vm.s_stack().push(1);
        b.iter(|| {
            vm.dup();
            vm.star();
        });
    }

    #[test]
    fn test_slash() {
        let vm = &mut VM::new(16, 16);
        vm.slash();
        vm.check_stacks();
        assert_eq!(vm.last_error(), Some(StackUnderflow));
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(30);
        vm.slash();
        vm.check_stacks();
        assert_eq!(vm.last_error(), Some(StackUnderflow));
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(30);
        vm.s_stack().push(7);
        vm.slash();
        vm.check_stacks();
        assert!(vm.last_error().is_none());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), 4);
    }

    #[bench]
    fn bench_slash(b: &mut Bencher) {
        let vm = &mut VM::new(16, 16);
        vm.s_stack().push(1);
        b.iter(|| {
            vm.dup();
            vm.slash();
        });
    }

    #[test]
    fn test_mod() {
        let vm = &mut VM::new(16, 16);
        vm.p_mod();
        vm.check_stacks();
        assert_eq!(vm.last_error(), Some(StackUnderflow));
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(30);
        vm.p_mod();
        vm.check_stacks();
        assert_eq!(vm.last_error(), Some(StackUnderflow));
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(30);
        vm.s_stack().push(7);
        vm.p_mod();
        vm.check_stacks();
        assert!(vm.last_error().is_none());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), 2);
    }

    #[bench]
    fn bench_mod(b: &mut Bencher) {
        let vm = &mut VM::new(16, 16);
        vm.s_stack().push(1);
        vm.s_stack().push(2);
        b.iter(|| {
            vm.p_mod();
            vm.s_stack().push(2);
        });
    }

    #[test]
    fn test_slash_mod() {
        let vm = &mut VM::new(16, 16);
        vm.slash_mod();
        vm.check_stacks();
        assert_eq!(vm.last_error(), Some(StackUnderflow));
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(30);
        vm.slash_mod();
        vm.check_stacks();
        assert_eq!(vm.last_error(), Some(StackUnderflow));
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(30);
        vm.s_stack().push(7);
        vm.slash_mod();
        vm.check_stacks();
        assert!(vm.last_error().is_none());
        assert_eq!(vm.s_stack().len(), 2);
        assert_eq!(vm.s_stack().pop(), 4);
        assert_eq!(vm.s_stack().pop(), 2);
    }

    #[bench]
    fn bench_slash_mod(b: &mut Bencher) {
        let vm = &mut VM::new(16, 16);
        vm.s_stack().push2(1, 2);
        b.iter(|| {
            vm.slash_mod();
            vm.p_drop();
            vm.s_stack().push(2);
        });
    }

    #[test]
    fn test_abs() {
        let vm = &mut VM::new(16, 16);
        vm.s_stack().push(-30);
        vm.abs();
        assert!(vm.last_error().is_none());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), 30);
    }

    #[test]
    fn test_negate() {
        let vm = &mut VM::new(16, 16);
        vm.negate();
        vm.check_stacks();
        assert_eq!(vm.last_error(), Some(StackUnderflow));
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(30);
        vm.negate();
        vm.check_stacks();
        assert!(vm.last_error().is_none());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), -30);
    }

    #[test]
    fn test_zero_less() {
        let vm = &mut VM::new(16, 16);
        vm.zero_less();
        vm.check_stacks();
        assert_eq!(vm.last_error(), Some(StackUnderflow));
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(-1);
        vm.zero_less();
        vm.check_stacks();
        assert!(vm.last_error().is_none());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), -1);
        vm.s_stack().push(0);
        vm.zero_less();
        vm.check_stacks();
        assert!(vm.last_error().is_none());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), 0);
    }

    #[test]
    fn test_zero_equals() {
        let vm = &mut VM::new(16, 16);
        vm.zero_equals();
        vm.check_stacks();
        assert_eq!(vm.last_error(), Some(StackUnderflow));
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(0);
        vm.zero_equals();
        vm.check_stacks();
        assert!(vm.last_error().is_none());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), -1);
        vm.s_stack().push(-1);
        vm.zero_equals();
        vm.check_stacks();
        assert!(vm.last_error().is_none());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), 0);
        vm.s_stack().push(1);
        vm.zero_equals();
        vm.check_stacks();
        assert!(vm.last_error().is_none());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), 0);
    }

    #[test]
    fn test_zero_greater() {
        let vm = &mut VM::new(16, 16);
        vm.zero_greater();
        vm.check_stacks();
        assert_eq!(vm.last_error(), Some(StackUnderflow));
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(1);
        vm.zero_greater();
        vm.check_stacks();
        assert!(vm.last_error().is_none());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), -1);
        vm.s_stack().push(0);
        vm.zero_greater();
        vm.check_stacks();
        assert!(vm.last_error().is_none());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), 0);
    }

    #[test]
    fn test_zero_not_equals() {
        let vm = &mut VM::new(16, 16);
        vm.zero_not_equals();
        vm.check_stacks();
        assert_eq!(vm.last_error(), Some(StackUnderflow));
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(0);
        vm.zero_not_equals();
        vm.check_stacks();
        assert!(vm.last_error().is_none());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), 0);
        vm.s_stack().push(-1);
        vm.zero_not_equals();
        vm.check_stacks();
        assert!(vm.last_error().is_none());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), -1);
        vm.s_stack().push(1);
        vm.zero_not_equals();
        vm.check_stacks();
        assert!(vm.last_error().is_none());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), -1);
    }

    #[test]
    fn test_less_than() {
        let vm = &mut VM::new(16, 16);
        vm.less_than();
        vm.check_stacks();
        assert_eq!(vm.last_error(), Some(StackUnderflow));
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(-1);
        vm.less_than();
        vm.check_stacks();
        assert_eq!(vm.last_error(), Some(StackUnderflow));
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(-1);
        vm.s_stack().push(0);
        vm.less_than();
        vm.check_stacks();
        assert!(vm.last_error().is_none());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), -1);
        vm.s_stack().push(0);
        vm.s_stack().push(0);
        vm.less_than();
        vm.check_stacks();
        assert!(vm.last_error().is_none());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), 0);
    }

    #[test]
    fn test_equals() {
        let vm = &mut VM::new(16, 16);
        vm.equals();
        vm.check_stacks();
        assert_eq!(vm.last_error(), Some(StackUnderflow));
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(0);
        vm.equals();
        vm.check_stacks();
        assert_eq!(vm.last_error(), Some(StackUnderflow));
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(0);
        vm.s_stack().push(0);
        vm.equals();
        vm.check_stacks();
        assert!(vm.last_error().is_none());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), -1);
        vm.s_stack().push(-1);
        vm.s_stack().push(0);
        vm.equals();
        vm.check_stacks();
        assert!(vm.last_error().is_none());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), 0);
        vm.s_stack().push(1);
        vm.s_stack().push(0);
        vm.equals();
        vm.check_stacks();
        assert!(vm.last_error().is_none());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), 0);
    }

    #[test]
    fn test_greater_than() {
        let vm = &mut VM::new(16, 16);
        vm.greater_than();
        vm.check_stacks();
        assert_eq!(vm.last_error(), Some(StackUnderflow));
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(1);
        vm.greater_than();
        vm.check_stacks();
        assert_eq!(vm.last_error(), Some(StackUnderflow));
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(1);
        vm.s_stack().push(0);
        vm.greater_than();
        vm.check_stacks();
        assert!(vm.last_error().is_none());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), -1);
        vm.s_stack().push(0);
        vm.s_stack().push(0);
        vm.greater_than();
        vm.check_stacks();
        assert!(vm.last_error().is_none());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), 0);
    }

    #[test]
    fn test_not_equals() {
        let vm = &mut VM::new(16, 16);
        vm.not_equals();
        vm.check_stacks();
        assert_eq!(vm.last_error(), Some(StackUnderflow));
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(0);
        vm.not_equals();
        vm.check_stacks();
        assert_eq!(vm.last_error(), Some(StackUnderflow));
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(0);
        vm.s_stack().push(0);
        vm.not_equals();
        vm.check_stacks();
        assert!(vm.last_error().is_none());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), 0);
        vm.s_stack().push(-1);
        vm.s_stack().push(0);
        vm.not_equals();
        vm.check_stacks();
        assert!(vm.last_error().is_none());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), -1);
        vm.s_stack().push(1);
        vm.s_stack().push(0);
        vm.not_equals();
        vm.check_stacks();
        assert!(vm.last_error().is_none());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), -1);
    }

    #[test]
    fn test_within() {
        let vm = &mut VM::new(16, 16);
        vm.within();
        vm.check_stacks();
        assert_eq!(vm.last_error(), Some(StackUnderflow));
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(1);
        vm.within();
        vm.check_stacks();
        assert_eq!(vm.last_error(), Some(StackUnderflow));
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(1);
        vm.s_stack().push(1);
        vm.within();
        vm.check_stacks();
        assert_eq!(vm.last_error(), Some(StackUnderflow));
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(1);
        vm.s_stack().push(1);
        vm.s_stack().push(2);
        vm.within();
        vm.check_stacks();
        assert!(vm.last_error().is_none());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), -1);
        vm.s_stack().push(1);
        vm.s_stack().push(0);
        vm.s_stack().push(1);
        vm.within();
        vm.check_stacks();
        assert!(vm.last_error().is_none());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), 0);
        vm.s_stack().push(0);
        vm.s_stack().push(1);
        vm.s_stack().push(2);
        vm.within();
        vm.check_stacks();
        assert!(vm.last_error().is_none());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), 0);
        vm.s_stack().push(3);
        vm.s_stack().push(1);
        vm.s_stack().push(2);
        vm.within();
        vm.check_stacks();
        assert!(vm.last_error().is_none());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), 0);
    }

    #[test]
    fn test_invert() {
        let vm = &mut VM::new(16, 16);
        vm.invert();
        vm.check_stacks();
        assert_eq!(vm.last_error(), Some(StackUnderflow));
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(707);
        vm.invert();
        vm.check_stacks();
        assert!(vm.last_error().is_none());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), -708);
    }

    #[test]
    fn test_and() {
        let vm = &mut VM::new(16, 16);
        vm.s_stack().push(707);
        vm.s_stack().push(007);
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
        let vm = &mut VM::new(16, 16);
        vm.or();
        vm.check_stacks();
        assert_eq!(vm.last_error(), Some(StackUnderflow));
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(707);
        vm.or();
        vm.check_stacks();
        assert_eq!(vm.last_error(), Some(StackUnderflow));
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(707);
        vm.s_stack().push(07);
        vm.or();
        vm.check_stacks();
        assert!(vm.last_error().is_none());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), 711);
    }

    #[test]
    fn test_xor() {
        let vm = &mut VM::new(16, 16);
        vm.xor();
        vm.check_stacks();
        assert_eq!(vm.last_error(), Some(StackUnderflow));
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(707);
        vm.xor();
        vm.check_stacks();
        assert_eq!(vm.last_error(), Some(StackUnderflow));
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(707);
        vm.s_stack().push(07);
        vm.xor();
        vm.check_stacks();
        assert!(vm.last_error().is_none());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), 708);
    }

    #[test]
    fn test_lshift() {
        let vm = &mut VM::new(16, 16);
        vm.lshift();
        vm.check_stacks();
        assert_eq!(vm.last_error(), Some(StackUnderflow));
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(1);
        vm.lshift();
        vm.check_stacks();
        assert_eq!(vm.last_error(), Some(StackUnderflow));
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(1);
        vm.s_stack().push(1);
        vm.lshift();
        vm.check_stacks();
        assert!(vm.last_error().is_none());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), 2);
        vm.s_stack().push(1);
        vm.s_stack().push(2);
        vm.lshift();
        vm.check_stacks();
        assert!(vm.last_error().is_none());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), 4);
    }

    #[test]
    fn test_rshift() {
        let vm = &mut VM::new(16, 16);
        vm.rshift();
        vm.check_stacks();
        assert_eq!(vm.last_error(), Some(StackUnderflow));
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(8);
        vm.rshift();
        vm.check_stacks();
        assert_eq!(vm.last_error(), Some(StackUnderflow));
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(8);
        vm.s_stack().push(1);
        vm.rshift();
        vm.check_stacks();
        assert!(vm.last_error().is_none());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), 4);
        vm.s_stack().push(-1);
        vm.s_stack().push(1);
        vm.rshift();
        vm.check_stacks();
        assert!(vm.last_error().is_none());
        assert_eq!(vm.s_stack().len(), 1);
        assert!(vm.s_stack().pop() > 0);
    }

    #[test]
    fn test_parse_word() {
        let vm = &mut VM::new(16, 16);
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
        let vm = &mut VM::new(16, 16);
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

    /*
    #[bench]
    fn bench_compile_words_at_beginning_of_wordlist(b: &mut Bencher) {
        let vm = &mut VM::new(16, 16);
        b.iter(|| {
            vm.set_source("marker empty : main noop noop noop noop noop noop noop noop ; empty");
            vm.evaluate();
            vm.s_stack().reset();
        });
    }

    #[bench]
    fn bench_compile_words_at_end_of_wordlist(b: &mut Bencher) {
        let vm = &mut VM::new(16, 16);
        b.iter(|| {
                   vm.set_source("marker empty : main bye bye bye bye bye bye bye bye ; empty");
                   vm.evaluate();
                   vm.s_stack().reset();
               });
    }

*/

    #[test]
    fn test_push_source() {
        let mut vm = VM::new(16, 16);
        vm.set_source(": x");
        vm.push_source(" ");
        vm.push_source("1");
        vm.push_source(" ");
        vm.push_source(";");
        assert_eq!(vm.input_buffer(), &Some(": x 1 ;".to_owned()));
    }

    #[test]
    fn test_colon_and_semi_colon() {
        let vm = &mut VM::new(16, 16);
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
        let vm = &mut VM::new(16, 16);
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
    fn test_constant_in_colon() {
        let vm = &mut VM::new(16, 16);
        // 77 constant x
        // : 2x  x 2 * ;  2x
        vm.set_source("77 constant x  : 2x x 2 * ;  2x");
        vm.evaluate();
        vm.run();
        assert_eq!(vm.s_stack().pop(), 154);
        assert_eq!(vm.s_stack().len, 0);
    }

    #[test]
    fn test_variable_and_store_fetch() {
        let vm = &mut VM::new(16, 16);
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
    fn test_variable_and_fetch_in_colon() {
        let vm = &mut VM::new(16, 16);
        // variable x
        // 7 x !
        // : x@ x @ ; x@
        vm.set_source("variable x  7 x !  : x@ x @ ;  x@");
        vm.evaluate();
        vm.run();
        assert_eq!(vm.s_stack().pop(), 7);
        assert_eq!(vm.s_stack().len, 0);
    }

    #[test]
    fn test_create_in_colon() {
        let vm = &mut VM::new(16, 16);
        // create x 7 ,
        // : x@ x @ ; x@
        vm.set_source("create x 7 ,  : x@ x @ ;  x@");
        vm.evaluate();
        vm.run();
        assert_eq!(vm.s_stack().pop(), 7);
        assert_eq!(vm.s_stack().len, 0);
    }

    #[test]
    fn test_char_plus_and_chars() {
        let vm = &mut VM::new(16, 16);
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
        let vm = &mut VM::new(16, 16);
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
        let vm = &mut VM::new(16, 16);
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
        let vm = &mut VM::new(16, 16);
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
        let vm = &mut VM::new(16, 16);
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
    fn test_to_r_r_fetch_r_from() {
        let vm = &mut VM::new(16, 16);
        vm.set_source(": t 3 >r 2 r@ + r> + ; t");
        vm.evaluate();
        assert!(vm.last_error().is_none());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), 8);
    }

    #[bench]
    fn bench_to_r_r_fetch_r_from(b: &mut Bencher) {
        let vm = &mut VM::new(16, 16);
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
        let vm = &mut VM::new(16, 16);
        vm.set_source(": t 1 2 2>r 2r@ + 2r> - * ; t");
        vm.evaluate();
        assert!(vm.last_error().is_none());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), -3);
    }

    #[bench]
    fn bench_two_to_r_two_r_fetch_two_r_from(b: &mut Bencher) {
        let vm = &mut VM::new(16, 16);
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
    fn test_if_then() {
        let vm = &mut VM::new(16, 16);
        // : t5 if ; t5
        vm.set_source(": t5 if ;");
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
        // : t1 false dup if drop true then ; t1
        vm.set_source(": t1 0 dup if drop -1 then ; t1");
        vm.evaluate();
        assert!(vm.last_error().is_none());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), 0);
        vm.reset();
        vm.clear_stacks();
        // : t2 true dup if drop false then ; t1
        vm.set_source(": t1 -1 dup if drop -1 then ; t1");
        vm.evaluate();
        assert!(vm.last_error().is_none());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), -1);
    }

    #[test]
    fn test_if_else_then() {
        let vm = &mut VM::new(16, 16);
        // : t3 else then ; t3
        vm.set_source(": t3 else then ;");
        vm.evaluate();
        assert_eq!(vm.last_error(), Some(ControlStructureMismatch));
        vm.reset();
        vm.clear_stacks();
        // : t1 0 if true else false then ; t1
        // let action = vm.code_space().here();
        vm.set_source(": t1 0 if true else false then ; t1");
        vm.evaluate();
        // dump(vm, action);
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
        let vm = &mut VM::new(16, 16);
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
        let vm = &mut VM::new(16, 16);
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
        let vm = &mut VM::new(16, 16);
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
        let vm = &mut VM::new(16, 16);
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
        let vm = &mut VM::new(16, 16);
        vm.set_source("1 2 3 abort 5 6 7");
        vm.evaluate();
        assert_eq!(vm.last_error(), Some(Abort));
        assert_eq!(vm.s_stack().len(), 0);
    }

    #[test]
    fn test_do_loop() {
        let vm = &mut VM::new(16, 16);
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
        let vm = &mut VM::new(16, 16);
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
        let vm = &mut VM::new(16, 16);
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
        let vm = &mut VM::new(16, 16);
        // : t1 leave ;
        vm.set_source(": t1 leave ;  t1");
        vm.evaluate();
        assert_eq!(vm.last_error(), Some(ControlStructureMismatch));
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
    fn test_do_leave_plus_loop() {
        let vm = &mut VM::new(16, 16);
        // : main 1 5 0 do 1+ dup 3 = if drop 88 leave then 2 +loop 9 ;  main
        vm.set_source(": main 1 5 0 do 1+ dup 3 = if drop 88 leave then 2 +loop 9 ;  main");
        vm.evaluate();
        assert_eq!(vm.last_error(), None);
        assert_eq!(vm.s_stack().len(), 2);
        assert_eq!(vm.s_stack().pop2(), (88, 9));
    }

    #[test]
    fn test_do_i_loop() {
        let vm = &mut VM::new(16, 16);
        // : main 3 0 do i loop ;  main
        vm.set_source(": main 3 0 do i loop ;  main");
        vm.evaluate();
        assert_eq!(vm.last_error(), None);
        assert_eq!(vm.s_stack().len(), 3);
        assert_eq!(vm.s_stack().pop3(), (0, 1, 2));
    }

    #[test]
    fn test_do_i_j_loop() {
        let vm = &mut VM::new(16, 16);
        vm.set_source(": main 6 4 do 3 1 do i j * loop loop ;  main");
        vm.evaluate();
        assert_eq!(vm.last_error(), None);
        assert_eq!(vm.s_stack().len(), 4);
        assert_eq!(vm.s_stack().as_slice(), [4, 8, 5, 10]);
    }

    #[bench]
    fn bench_fib(b: &mut Bencher) {
        let vm = &mut VM::new(16, 16);
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
        let vm = &mut VM::new(16, 16);
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
        let vm = &mut VM::new(16, 16);
        vm.load("./lib.fs");
        if vm.last_error().is_some() {
            eprintln!("Error {:?} at {:?}", vm.last_error().unwrap(), vm.last_token());
        }
        assert_eq!(vm.last_error(), None);
        vm.set_source("CREATE FLAGS 8190 ALLOT   VARIABLE EFLAG");
        vm.evaluate();
        assert_eq!(vm.last_error(), None);
        vm.set_source(
            "
            : PRIMES  ( -- n )  FLAGS 8190 1 FILL  0 3  EFLAG @ FLAGS
                DO   I C@
                    IF  DUP I + DUP EFLAG @ <
                        IF    EFLAG @ SWAP
                            DO  0 I C! DUP  +LOOP
                        ELSE  DROP  THEN  SWAP 1+ SWAP
                    THEN  2 +
                LOOP  DROP ;
        ",
        );
        vm.evaluate();
        assert_eq!(vm.last_error(), None);
        vm.set_source(
            "
            : BENCHMARK  0 1 0 DO  PRIMES NIP  LOOP ;
        ",
        );
        vm.evaluate();
        assert_eq!(vm.last_error(), None);
        vm.set_source(
            "
            : MAIN
                FLAGS 8190 + EFLAG !
                BENCHMARK DROP
            ;
        ",
        );
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

    #[test]
    #[cfg(not(feature = "subroutine-threaded"))]
    fn test_here_comma_compile_interpret() {
        let vm = &mut VM::new(16, 16);
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
        assert_eq!(vm.data_space().get_i32(here + 0), 1);
        assert_eq!(vm.data_space().get_i32(here + 4), 2);
        assert_eq!(
            vm.data_space().get_i32(here + 8),
            vm.references().idx_lit as i32
        );
        assert_eq!(
            vm.data_space().get_i32(here + 12),
            vm.references().idx_exit as i32
        );
    }

    #[test]
    #[cfg(all(feature = "subroutine-threaded", target_arch = "x86"))]
    fn test_subroutine_threaded_colon_and_semi_colon() {
        let vm = &mut VM::new(16, 16);
        // : nop ;
        // 56               push   %esi
        // 89 ce            mov    %ecx,%esi
        // 83 ec 08         sub    $8,%esp
        //
        // 83 c4 08         add    $8,%esp
        // 5e               pop    %esi
        // c3               ret
        let action = vm.code_space().here();
        vm.set_source(": nop ; nop");
        vm.evaluate();
        let w = vm.last_definition();
        assert_eq!(vm.wordlist()[w].action as usize, action);
        unsafe {
            assert_eq!(vm.code_space().get_u8(action + 0), 0x56);
            assert_eq!(vm.code_space().get_u8(action + 1), 0x89);
            assert_eq!(vm.code_space().get_u8(action + 2), 0xce);
            assert_eq!(vm.code_space().get_u8(action + 3), 0x83);
            assert_eq!(vm.code_space().get_u8(action + 4), 0xec);
            assert_eq!(vm.code_space().get_u8(action + 5), 0x08);

            assert_eq!(vm.code_space().get_u8(action + 6), 0x83);
            assert_eq!(vm.code_space().get_u8(action + 7), 0xc4);
            assert_eq!(vm.code_space().get_u8(action + 8), 0x08);
            assert_eq!(vm.code_space().get_u8(action + 9), 0x5e);
            assert_eq!(vm.code_space().get_u8(action + 10), 0xc3);
        }
    }

    // TODO: added a dump forth word.
    fn dump(vm: &mut VM, addr: usize) {
        if vm.code_space().has(addr) {
            for i in 0..8 {
                if vm.code_space().has(addr + 7 + i * 8) {
                    unsafe {
                        println!(
                            "{:2x} {:2x} {:2x} {:2x} {:2x} {:2x} {:2x} {:2x}",
                            vm.code_space().get_u8(addr + 0 + i * 8),
                            vm.code_space().get_u8(addr + 1 + i * 8),
                            vm.code_space().get_u8(addr + 2 + i * 8),
                            vm.code_space().get_u8(addr + 3 + i * 8),
                            vm.code_space().get_u8(addr + 4 + i * 8),
                            vm.code_space().get_u8(addr + 5 + i * 8),
                            vm.code_space().get_u8(addr + 6 + i * 8),
                            vm.code_space().get_u8(addr + 7 + i * 8),
                        );
                    }
                } else {
                    panic!("Error: invaild dump address.");
                }
            }
        } else {
            panic!("Error: invaild dump address.");
        }
    }

    #[test]
    #[cfg(all(feature = "subroutine-threaded", target_arch = "x86"))]
    fn test_subroutine_threaded_lit_and_plus() {
        let vm = &mut VM::new(16, 16);
        // : 2+3 2 3 + ;
        // let action = vm.code_space().here();
        vm.set_source(": 2+3 2 3 + ;");
        vm.evaluate();
        // dump(vm, action);
        // 2+3
        vm.set_source("2+3");
        vm.evaluate();
        assert!(vm.last_error().is_none());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), 5);
    }

    #[test]
    #[cfg(all(feature = "subroutine-threaded", target_arch = "x86"))]
    fn test_subroutine_threaded_if_then() {
        let vm = &mut VM::new(16, 16);
        // : t5 if ; t5
        vm.set_source(": t5 if ;");
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
        // let action = vm.code_space().here();
        // : tx dup drop ;
        vm.set_source(": tx dup drop ;");
        vm.evaluate();
        // dump(vm, action);
        // println!("**** t1 ****");
        // let action = vm.code_space().here();
        // : t1 0 dup if drop -1 then ; t1
        vm.set_source(": t1 0 dup if drop -1 then ; t1");
        vm.evaluate();
        // dump(vm, action);
        assert!(vm.last_error().is_none());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), 0);
        vm.reset();
        vm.clear_stacks();
        // : t2 -1 dup if drop 0 then ; t1
        vm.set_source(": t2 -1 dup if drop -1 then ; t2");
        vm.evaluate();
        assert!(vm.last_error().is_none());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), -1);
    }

}
