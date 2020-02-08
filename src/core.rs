extern crate libc;
use exception::{
    self, ABORT, CONTROL_STRUCTURE_MISMATCH, DIVISION_BY_ZERO, FLOATING_POINT_STACK_OVERFLOW,
    FLOATING_POINT_STACK_UNDERFLOW, INTERPRETING_A_COMPILE_ONLY_WORD, INVALID_MEMORY_ADDRESS,
    INVALID_NUMERIC_ARGUMENT, RETURN_STACK_OVERFLOW, RETURN_STACK_UNDERFLOW, STACK_OVERFLOW,
    STACK_UNDERFLOW, UNDEFINED_WORD, UNEXPECTED_END_OF_FILE, FLOATING_POINT_UNIDENTIFIED_FAULT,
    INTEGER_UNIDENTIFIED_FAULT, INVALID_EXECUTION_TOKEN,
};
use hibitset::{BitSet, BitSetLike};
use loader::Source;
use memory::{CodeSpace, DataSpace, Memory};
use parser;
use std::fmt::Write;
use std::fmt::{self, Display};
use std::fs::File;
use std::mem;
use std::ops::{Index, IndexMut};
use std::str;
use {FALSE, NUM_TASKS, TRUE};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum WordType {
    // Words implemented with a rust function or with subroutine-threaded code
    Native,
    // Words implemented with token-threaded code
    Nest,
    // Words defined with CREATE, VARIABLE or  FVARIABLE
    Var,
    // Words defined with CONSTANT
    Const,
    // Words defined with FCONSTANT
    Fconst,
    // Words defined with CREATE DOES>
    Does,
    // Words defined with Marker
    Marker,
}

// Word
pub struct Word<Target> {
    word_type: WordType,
    is_immediate: bool,
    is_compile_only: bool,
    hidden: bool,
    link: usize,
    hash: u32,
    nfa: usize,
    dfa: usize,
    cfa: usize,
    pub action: primitive! { fn (&mut Target) },
    pub compilation_semantics: primitive! { fn(&mut Target) },
    // Minimum execution time in [ns]
    pub(crate) min_execution_time: usize,
    // Maximum execution time in [ns]
    pub(crate) max_execution_time: usize,
}

impl<Target> Word<Target> {
    pub fn new(
        word_type: WordType,
        action: primitive! {fn(&mut Target)},
        compilation_semantics: primitive! { fn(&mut Target) },
        nfa: usize,
        dfa: usize,
        cfa: usize,
    ) -> Word<Target> {
        Word {
            word_type,
            is_immediate: false,
            is_compile_only: false,
            hidden: false,
            link: 0,
            hash: 0,
            nfa: nfa,
            dfa: dfa,
            cfa: cfa,
            action: action,
            compilation_semantics: compilation_semantics,
            min_execution_time: 0,
            max_execution_time: 0,
        }
    }

    pub fn word_type(&self) -> WordType {
        self.word_type
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

    pub fn nfa(&self) -> usize {
        self.nfa
    }

    pub fn dfa(&self) -> usize {
        self.dfa
    }

    pub fn cfa(&self) -> usize {
        self.cfa
    }

    pub fn action(&self) -> primitive! {fn(&mut Target)} {
        self.action
    }

    pub fn set_action(&mut self, action: primitive! {fn(&mut Target)}) {
        self.action = action;
    }
}

const BUCKET_SIZE: usize = 64;

/// Wordlist
pub struct Wordlist<Target> {
    words: Vec<Word<Target>>,
    buckets: [usize; BUCKET_SIZE],
    temp_buckets: [usize; BUCKET_SIZE],
    last: usize,
}

impl<Target> Wordlist<Target> {
    /// Create a wordlist with capacity of `cap`.
    pub fn with_capacity(cap: usize) -> Wordlist<Target> {
        Wordlist {
            words: Vec::with_capacity(cap),
            buckets: [0; BUCKET_SIZE],
            temp_buckets: [0; BUCKET_SIZE],
            last: 0,
        }
    }

    /// Word count
    pub fn len(&self) -> usize {
        self.words.len()
    }

    /// Last word
    pub fn last(&self) -> usize {
        self.last
    }

    // Hash function
    //
    // Alogrithm djb2 at http://www.cse.yorku.ca/~oz/hash.html .
    fn hash(name: &str) -> u32 {
        let mut hash: u32 = 5381;
        for c in name.bytes() {
            hash = hash
                .wrapping_shl(5)
                .wrapping_add(hash)
                .wrapping_add(c.to_ascii_lowercase() as u32); /* hash * 33 + c */
        }
        hash
    }

    /// Push word `w` into list.
    fn push(&mut self, name: &str, mut w: Word<Target>) {
        w.hash = Self::hash(name);
        let b = w.hash as usize % BUCKET_SIZE;
        w.link = self.buckets[b];
        self.last = self.words.len();
        self.buckets[b] = self.last;
        self.words.push(w);
    }

    /// Remove the `i`th word and all words behind it.
    fn truncate(&mut self, i: usize) {
        self.words.truncate(i);
        self.last = self.words.len() - 1;
    }
}

impl<Target> Index<usize> for Wordlist<Target> {
    type Output = Word<Target>;
    #[inline(always)]
    fn index(&self, index: usize) -> &Word<Target> {
        &self.words[index]
    }
}

impl<Target> IndexMut<usize> for Wordlist<Target> {
    #[inline(always)]
    fn index_mut(&mut self, index: usize) -> &mut Word<Target> {
        &mut self.words[index]
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
        for i in 0..256 {
            self.inner[i] = self.canary;
        }
    }

    pub fn underflow(&self) -> bool {
        (self.inner[255] != self.canary) || (self.len > 128)
    }

    pub fn overflow(&self) -> bool {
        (self.inner[64] != self.canary) || (self.len > 64 && self.len <= 128)
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

    pub fn last(&self) -> T {
        self.inner[self.len.wrapping_sub(1) as usize]
    }

    pub fn get(&self, pos: u8) -> T {
        self.inner[pos as usize]
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

#[allow(non_snake_case)]
pub struct ForwardReferences {
    pub idx_lit: usize,
    pub idx_flit: usize,
    pub idx_exit: usize,
    pub idx_zero_branch: usize,
    pub idx_branch: usize,
    pub idx_if: usize,
    pub idx_else: usize,
    pub idx_then: usize,
    pub idx_case: usize,
    pub idx_of: usize,
    pub idx_endof: usize,
    pub idx_endcase: usize,
    pub idx_begin: usize,
    pub idx_while: usize,
    pub idx_repeat: usize,
    pub idx_until: usize,
    pub idx_again: usize,
    pub idx_do: usize,
    pub idx__do: usize,
    pub idx_qdo: usize,
    pub idx__qdo: usize,
    pub idx_loop: usize,
    pub idx__loop: usize,
    pub idx_plus_loop: usize,
    pub idx__plus_loop: usize,
    pub idx_unloop: usize,
    pub idx_leave: usize,
    pub idx_j: usize,
    pub idx_s_quote: usize,
    pub idx_type: usize,
    pub idx_over: usize,
    pub idx_equal: usize,
    pub idx_drop: usize,
    pub idx__postpone: usize,
    pub idx_to_r: usize,
    pub idx_colon: usize,
    pub idx_does: usize,
    pub idx__does: usize,
}

impl ForwardReferences {
    pub fn new() -> ForwardReferences {
        ForwardReferences {
            idx_lit: 0,
            idx_flit: 0,
            idx_exit: 0,
            idx_zero_branch: 0,
            idx_branch: 0,
            idx_if: 0,
            idx_else: 0,
            idx_then: 0,
            idx_case: 0,
            idx_of: 0,
            idx_endof: 0,
            idx_endcase: 0,
            idx_begin: 0,
            idx_while: 0,
            idx_repeat: 0,
            idx_until: 0,
            idx_again: 0,
            idx_do: 0,
            idx__do: 0,
            idx_qdo: 0,
            idx__qdo: 0,
            idx_loop: 0,
            idx__loop: 0,
            idx_plus_loop: 0,
            idx__plus_loop: 0,
            idx_unloop: 0,
            idx_leave: 0,
            idx_j: 0,
            idx_s_quote: 0,
            idx_type: 0,
            idx_over: 0,
            idx_equal: 0,
            idx_drop: 0,
            idx__postpone: 0,
            idx_to_r: 0,
            idx_colon: 0,
            idx_does: 0,
            idx__does: 0,
        }
    }
}

pub struct State {
    pub is_compiling: bool,
    pub instruction_pointer: usize,
    word_pointer: usize,
    pub source_index: usize,
    pub source_id: isize,
}

impl State {
    pub fn new() -> State {
        State {
            is_compiling: false,
            instruction_pointer: 0,
            word_pointer: 0,
            source_index: 0,
            source_id: 0,
        }
    }

    pub fn word_pointer(&self) -> usize {
        self.word_pointer
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
    fn last_error(&self) -> Option<isize>;
    fn set_error(&mut self, e: Option<isize>);
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
    /// Input source identifier
    ///
    /// > 0: input from source at `self.sources[source_id] and input buffer
    /// `self.lines[source_id]`.
    /// = 0: input from the default user input buffer.
    fn source_id(&self) -> isize;
    /// Get `input_buffer`.
    fn input_buffer(&mut self) -> &mut Option<String>;
    /// Set `input_buffer` to `Some(buffer)`.
    fn set_input_buffer(&mut self, buffer: String);
    fn files(&self) -> &Vec<Option<File>>;
    fn files_mut(&mut self) -> &mut Vec<Option<File>>;
    fn sources(&self) -> &Vec<Option<Source>>;
    fn sources_mut(&mut self) -> &mut Vec<Option<Source>>;
    fn lines(&self) -> &Vec<Option<String>>;
    fn lines_mut(&mut self) -> &mut Vec<Option<String>>;
    fn last_token(&mut self) -> &mut Option<String>;
    fn set_last_token(&mut self, buffer: String);
    fn regs(&mut self) -> &mut [usize; 2];
    fn s_stack(&mut self) -> &mut Stack<isize>;
    fn r_stack(&mut self) -> &mut Stack<isize>;
    fn c_stack(&mut self) -> &mut Stack<Control>;
    fn f_stack(&mut self) -> &mut Stack<f64>;
    /// Last definition, 0 if last define fails.
    fn wordlist_mut(&mut self) -> &mut Wordlist<Self>;
    fn wordlist(&self) -> &Wordlist<Self>;
    fn state(&mut self) -> &mut State;
    fn references(&mut self) -> &mut ForwardReferences;
    fn system_time_ns(&self) -> u64;
    /// Current task
    fn current_task(&self) -> usize;
    /// Set curretn task.
    ///
    /// No operation if there is no task `i`.
    fn set_current_task(&mut self, i: usize);
    /// Is task `i` awake?
    ///
    /// False if there is no task `i`.
    fn awake(&self, i: usize) -> bool;
    /// Set awake to `v`.
    ///
    /// No operation if there is no task `i`.
    fn set_awake(&mut self, i: usize, v: bool);
    /// Bitset to check forward declaration of labels.
    fn forward_bitset(&self) -> &BitSet;
    /// Mutable bitset to check forward declaration of labels.
    fn forward_bitset_mut(&mut self) -> &mut BitSet;
    /// Bitset to check resolved labels.
    fn resolved_bitset(&self) -> &BitSet;
    /// Mutable bitset to check resolved labels.
    fn resolved_bitset_mut(&mut self) -> &mut BitSet;
    /// Labels to support BASIC-like goto, label, call.
    fn labels(&self) -> &Vec<usize>;
    /// Labels to support BASIC-like goto, label, call.
    fn labels_mut(&mut self) -> &mut Vec<usize>;

    /// Add core primitives to self.
    fn add_core(&mut self) {
        self.add_primitive("", Core::noop);
        self.add_primitive("noop", Core::noop);
        self.add_compile_only("exit", Core::exit);
        self.add_compile_only("lit", Core::lit);
        self.add_compile_only("flit", Core::flit);
        self.add_compile_only("_s\"", Core::p_s_quote);
        self.add_compile_only("branch", Core::branch);
        self.add_compile_only("0branch", Core::zero_branch);
        self.add_compile_only("_do", Core::_do);
        self.add_compile_only("_qdo", Core::_qdo);
        self.add_compile_only("_loop", Core::_loop);
        self.add_compile_only("_+loop", Core::_plus_loop);
        self.add_compile_only("unloop", Core::unloop);
        self.add_compile_only("leave", Core::leave);
        self.add_compile_only("i", Core::p_i);
        self.add_compile_only("j", Core::p_j);
        self.add_compile_only(">r", Core::p_to_r);
        self.add_compile_only("r>", Core::r_from);
        self.add_compile_only("r@", Core::r_fetch);
        self.add_compile_only("2>r", Core::two_to_r);
        self.add_compile_only("2r>", Core::two_r_from);
        self.add_compile_only("2r@", Core::two_r_fetch);
        self.add_compile_only("compile,", Core::compile_comma);
        self.add_compile_only("_postpone", Core::_postpone);
        self.add_compile_only("_does", Core::_does);

        self.add_primitive("execute", Core::execute);
        self.add_primitive("dup", Core::dup);
        self.add_primitive("drop", Core::p_drop);
        self.add_primitive("swap", Core::swap);
        self.add_primitive("over", Core::over);
        self.add_primitive("nip", Core::nip);
        self.add_primitive("depth", Core::depth);
        self.add_primitive("?stacks", Core::check_stacks);
        self.add_primitive("0<", Core::zero_less);
        self.add_primitive("=", Core::equals);
        self.add_primitive("<", Core::less_than);
        self.add_primitive("invert", Core::invert);
        self.add_primitive("and", Core::and);
        self.add_primitive("or", Core::or);
        self.add_primitive("xor", Core::xor);
        self.add_primitive("lshift", Core::lshift);
        self.add_primitive("rshift", Core::rshift);
        self.add_primitive("1+", Core::one_plus);
        self.add_primitive("1-", Core::one_minus);
        self.add_primitive("-", Core::minus);
        self.add_primitive("+", Core::plus);
        self.add_primitive("*", Core::star);
        self.add_primitive("/mod", Core::slash_mod);
        self.add_primitive("cell+", Core::cell_plus);
        self.add_primitive("cells", Core::cells);
        self.add_primitive("@", Core::fetch);
        self.add_primitive("!", Core::store);
        self.add_primitive("char+", Core::char_plus);
        self.add_primitive("here", Core::here);
        self.add_primitive("allot", Core::allot);
        self.add_primitive("aligned", Core::aligned);
        self.add_primitive("align", Core::align);
        self.add_primitive("c@", Core::c_fetch);
        self.add_primitive("c!", Core::c_store);
        self.add_primitive("move", Core::p_move);
        self.add_primitive("base", Core::base);
        self.add_primitive("immediate", Core::immediate);
        self.add_primitive("compile-only", Core::compile_only);

        // Immediate words
        self.add_immediate("(", Core::imm_paren);
        self.add_immediate("\\", Core::imm_backslash);
        self.add_immediate("[", Core::left_bracket);
        self.add_immediate_and_compile_only("[']", Core::bracket_tick);
        self.add_immediate_and_compile_only("[char]", Core::bracket_char);
        self.add_immediate_and_compile_only(";", Core::semicolon);
        self.add_immediate_and_compile_only("if", Core::compile_if);
        self.add_immediate_and_compile_only("else", Core::compile_else);
        self.add_immediate_and_compile_only("then", Core::compile_then);
        self.add_immediate_and_compile_only("case", Core::compile_case);
        self.add_immediate_and_compile_only("of", Core::compile_of);
        self.add_immediate_and_compile_only("endof", Core::compile_endof);
        self.add_immediate_and_compile_only("endcase", Core::compile_endcase);
        self.add_immediate_and_compile_only("begin", Core::compile_begin);
        self.add_immediate_and_compile_only("while", Core::compile_while);
        self.add_immediate_and_compile_only("repeat", Core::compile_repeat);
        self.add_immediate_and_compile_only("until", Core::compile_until);
        self.add_immediate_and_compile_only("again", Core::compile_again);
        self.add_immediate("0labels", Core::imm_clear_labels);
        self.add_immediate_and_compile_only("label", Core::imm_label);
        self.add_immediate_and_compile_only("goto", Core::imm_goto);
        self.add_immediate_and_compile_only("call", Core::imm_call);
        self.add_immediate_and_compile_only("recurse", Core::imm_recurse);
        self.add_immediate_and_compile_only("do", Core::compile_do);
        self.add_immediate_and_compile_only("?do", Core::compile_qdo);
        self.add_immediate_and_compile_only("loop", Core::compile_loop);
        self.add_immediate_and_compile_only("+loop", Core::compile_plus_loop);
        self.add_immediate_and_compile_only("postpone", Core::postpone);
        self.add_immediate_and_compile_only("does>", Core::does);

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
        self.add_primitive("pick", Core::pick);
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
        self.add_primitive("_skip", Core::_skip);
        self.add_primitive("_parse", Core::_parse);
        self.add_primitive(":", Core::colon);
        self.add_primitive("constant", Core::constant);
        self.add_primitive("create", Core::create);
        self.add_primitive("'", Core::tick);
        self.add_primitive(">body", Core::to_body);
        self.add_primitive("]", Core::right_bracket);
        self.add_primitive(",", Core::comma);
        self.add_primitive("marker", Core::marker);
        self.add_primitive("handler!", Core::handler_store);
        self.add_primitive("error", Core::error);
        self.add_primitive(".error", Core::dot_error);
        self.add_primitive("0error", Core::clear_error);
        self.add_primitive("0stacks", Core::clear_stacks);
        self.add_primitive("reset", Core::reset);
        self.add_primitive("abort", Core::abort);
        self.add_primitive(">action", Core::to_action);
        self.add_primitive("action>", Core::from_action);
        self.add_primitive(">nfa", Core::to_nfa);
        self.add_primitive(">cfa", Core::to_cfa);
        self.add_primitive(">dfa", Core::to_dfa);
        self.add_primitive("dfa>", Core::from_dfa);
        self.add_primitive(".name", Core::dot_name);
        self.add_primitive("compiling?", Core::p_compiling);
        self.add_primitive("token-empty?", Core::token_empty);
        self.add_primitive(".token", Core::dot_token);
        self.add_primitive("!token", Core::store_token);
        self.add_primitive("compile-token", Core::compile_token);
        self.add_primitive("interpret-token", Core::interpret_token);
        self.add_primitive("source-id", Core::p_source_id);
        self.add_primitive("source-id!", Core::p_set_source_id);
        self.add_primitive("source-idx", Core::p_source_idx);
        self.add_primitive("source-idx!", Core::p_set_source_idx);
        self.add_primitive("-optimization", Core::patch_compilation_semanticses);

        self.references().idx_lit = self.find("lit").expect("lit undefined");
        self.references().idx_flit = self.find("flit").expect("flit undefined");
        self.references().idx_exit = self.find("exit").expect("exit undefined");
        self.references().idx_zero_branch = self.find("0branch").expect("0branch undefined");
        self.references().idx_branch = self.find("branch").expect("branch undefined");
        self.references().idx_if = self.find("if").expect("if undefined");
        self.references().idx_else = self.find("else").expect("else undefined");
        self.references().idx_then = self.find("then").expect("then undefined");
        self.references().idx_case = self.find("case").expect("case undefined");
        self.references().idx_of = self.find("of").expect("of undefined");
        self.references().idx_endof = self.find("endof").expect("endof undefined");
        self.references().idx_endcase = self.find("endcase").expect("endcase undefined");
        self.references().idx_begin = self.find("begin").expect("begin undefined");
        self.references().idx_while = self.find("while").expect("while undefined");
        self.references().idx_repeat = self.find("repeat").expect("repeat undefined");
        self.references().idx_until = self.find("until").expect("until undefined");
        self.references().idx_again = self.find("again").expect("again undefined");
        self.references().idx_do = self.find("do").expect("do undefined");
        self.references().idx__do = self.find("_do").expect("_do undefined");
        self.references().idx_qdo = self.find("?do").expect("?do undefined");
        self.references().idx__qdo = self.find("_qdo").expect("_qdo undefined");
        self.references().idx_loop = self.find("loop").expect("loop undefined");
        self.references().idx__loop = self.find("_loop").expect("_loop undefined");
        self.references().idx_plus_loop = self.find("+loop").expect("+loop undefined");
        self.references().idx__plus_loop = self.find("_+loop").expect("_+loop undefined");
        self.references().idx_unloop = self.find("unloop").expect("unloop undefined");
        self.references().idx_leave = self.find("leave").expect("leave undefined");
        self.references().idx_j = self.find("j").expect("j undefined");
        self.references().idx_over = self.find("over").expect("over undefined");
        self.references().idx_equal = self.find("=").expect("= undefined");
        self.references().idx_drop = self.find("drop").expect("drop undefined");
        self.references().idx__postpone = self.find("_postpone").expect("_postpone undefined");
        self.references().idx_to_r = self.find(">r").expect(">r");
        self.references().idx_colon = self.find(":").expect(":");
        self.references().idx_does = self.find("does>").expect("does>");
        self.references().idx__does = self.find("_does").expect("_does");

        self.patch_compilation_semanticses();

        #[cfg(not(feature = "stc"))]
        {
            // Multitasker
            self.add_compile_only("pause", Core::pause);
            self.add_compile_only("activate", Core::activate);
            self.add_primitive("me", Core::me);
            self.add_primitive("suspend", Core::suspend);
            self.add_primitive("resume", Core::resume);
        }
        self.set_awake(0, true);
    }

    /// Add a primitive word to word list.
    fn add_primitive(&mut self, name: &str, action: primitive! {fn(&mut Self)}) {
        let nfa = self.data_space().compile_str(name);
        self.data_space().align();
        self.code_space().align();
        let word = Word::new(
            WordType::Native,
            action,
            Core::compile_comma,
            nfa,
            self.data_space().here(),
            self.code_space().here(),
        );
        self.wordlist_mut().push(name, word);
    }

    /// Set the last definition immediate.
    primitive! {fn immediate(&mut self) {
        let def = self.wordlist().last;
        self.wordlist_mut()[def].set_immediate(true);
    }}

    /// Add an immediate word to word list.
    fn add_immediate(&mut self, name: &str, action: primitive! {fn(&mut Self)}) {
        self.add_primitive(name, action);
        self.immediate();
    }

    /// Set the last definition compile-only.
    primitive! {fn compile_only(&mut self) {
        let def = self.wordlist().last;
        self.wordlist_mut()[def].set_compile_only(true);
    }}

    /// Add a compile-only word to word list.
    fn add_compile_only(&mut self, name: &str, action: primitive! {fn(&mut Self)}) {
        self.add_primitive(name, action);
        self.compile_only();
    }

    /// Add an immediate and compile-only word to word list.
    fn add_immediate_and_compile_only(&mut self, name: &str, action: primitive! {fn(&mut Self)}) {
        self.add_primitive(name, action);
        self.immediate();
        self.compile_only();
    }

    /// Execute word at position `i`.
    fn execute_word(&mut self, i: usize) {
        self.state().word_pointer = i;
        if i < self.wordlist().len() {
            (self.wordlist()[i].action())(self);
        } else {
            self.abort_with(INVALID_EXECUTION_TOKEN);
        }
    }

    /// Find the word with name `name`.
    /// If not found returns zero.
    fn find(&mut self, name: &str) -> Option<usize> {
        let hash = Wordlist::<Self>::hash(name);
        let mut w = self.wordlist().buckets[hash as usize % BUCKET_SIZE];
        while w != 0 {
            if !self.wordlist()[w].is_hidden() {
                {
                    if self.wordlist()[w].hash == hash {
                        let nfa = self.wordlist()[w].nfa();
                        let w_name = unsafe { self.data_space().get_str(nfa) };
                        if w_name.eq_ignore_ascii_case(name) {
                            return Some(w);
                        }
                    }
                }
            }
            w = self.wordlist()[w].link;
        }
        None
    }

    // -------------------------------
    // Token threaded code
    // -------------------------------

    /// Evaluate a compiled program following self.state().instruction_pointer.
    /// Any exception causes termination of inner loop.
    #[inline(never)]
    fn run(&mut self) {
        let mut ip = self.state().instruction_pointer;
        while self.data_space().start() <= ip
            && ip + mem::size_of::<isize>() <= self.data_space().limit()
        {
            let w = unsafe { self.data_space().get_isize(ip) as usize };
            self.state().instruction_pointer += mem::size_of::<isize>();
            self.execute_word(w);
            ip = self.state().instruction_pointer;
        }
    }

    primitive! {fn compile_var(&mut self) {
        let compile_var_vector = self.data_space().system_variables().compile_var_vector();
        unsafe {
            let compile_var_vector: *const primitive!{fn (&mut Self)} = mem::transmute (compile_var_vector);
            (*compile_var_vector)(self);
        }
    }}

    primitive! {fn compile_const(&mut self) {
        let compile_const_vector = self.data_space().system_variables().compile_const_vector();
        unsafe {
            let compile_const_vector: *const primitive!{fn (&mut Self)} = mem::transmute (compile_const_vector);
            (*compile_const_vector)(self);
        }
    }}

    primitive! {fn compile_fconst(&mut self) {
        let compile_fconst_vector = self.data_space().system_variables().compile_fconst_vector();
        unsafe {
            let compile_fconst_vector: *const primitive!{fn (&mut Self)} = mem::transmute (compile_fconst_vector);
            (*compile_fconst_vector)(self);
        }
    }}

    primitive! {fn lit(&mut self) {
        let ip = self.state().instruction_pointer;
        let v = unsafe{ self.data_space().get_isize(ip) as isize };
        let slen = self.s_stack().len.wrapping_add(1);
        self.s_stack().len = slen;
        self.s_stack()[slen.wrapping_sub(1)] = v;
        self.state().instruction_pointer += mem::size_of::<isize>();
    }}

    /// Compile integer `n`.
    ///
    /// For token-threading.
    ///
    /// Run-time: ( n -- )
    primitive! {fn tt_compile_integer(&mut self) {
        let n = self.s_stack().pop();
        let idx = self.references().idx_lit;
        self.s_stack().push(idx as isize);
        self.compile_comma();
        self.data_space().compile_isize(n as isize);
    }}

    /// Compile integer `i`.
    ///
    /// Run-time: ( n -- )
    primitive! {fn compile_integer(&mut self) {
        let compile_integer_vector = self.data_space().system_variables().compile_integer_vector();
        unsafe {
            let compile_integer_vector: *const primitive!{fn (&mut Self)} = mem::transmute (compile_integer_vector);
            (*compile_integer_vector)(self);
        }
    }}

    primitive! {fn flit(&mut self) {
        let ip = DataSpace::aligned_f64(self.state().instruction_pointer as usize);
        let v = unsafe{ self.data_space().get_f64(ip) };
        let flen = self.f_stack().len.wrapping_add(1);
        self.f_stack().len = flen;
        self.f_stack()[flen.wrapping_sub(1)] = v;
        self.state().instruction_pointer = ip + mem::size_of::<f64>();
    }}

    /// Compile float 'f'.
    ///
    /// The token-threading version.
    ///
    /// Run-time: ( f -- )
    primitive! {fn tt_compile_float(&mut self) {
        let f = self.f_stack().pop();
        let idx_flit = self.references().idx_flit;
        self.s_stack().push(idx_flit as isize);
        self.compile_comma();
        self.data_space().align_f64();
        self.data_space().compile_f64(f);
    }}

    /// Compile float 'f'.
    ///
    /// Run-time: ( f -- )
    primitive! {fn compile_float(&mut self) {
        let compile_float_vector = self.data_space().system_variables().compile_float_vector();
        unsafe {
            let compile_float_vector: *const primitive!{fn (&mut Self)} = mem::transmute (compile_float_vector);
            (*compile_float_vector)(self);
        }
    }}

    /// Runtime of S"
    #[cfg(not(feature = "stc"))]
    primitive! {fn p_s_quote(&mut self) {
        let ip = self.state().instruction_pointer;
        let (addr, cnt) = {
            let s = unsafe{ self.data_space().get_str(ip) };
            (s.as_ptr() as isize, s.len() as isize)
        };
        let slen = self.s_stack().len.wrapping_add(2);
        self.s_stack().len = slen;
        self.s_stack()[slen.wrapping_sub(1)] = cnt;
        self.s_stack()[slen.wrapping_sub(2)] = addr;
        self.state().instruction_pointer = DataSpace::aligned(
            self.state().instruction_pointer + mem::size_of::<isize>() + cnt as usize
        );
    }}

    primitive! {fn patch_compilation_semanticses(&mut self) {
        //  Words with different action
        let idx_if = self.references().idx_if;
        self.wordlist_mut()[idx_if].action = Self::compile_if;
        let idx_else = self.references().idx_else;
        self.wordlist_mut()[idx_else].action = Self::compile_else;
        let idx_then = self.references().idx_then;
        self.wordlist_mut()[idx_then].action = Self::compile_then;
        let idx_case = self.references().idx_case;
        self.wordlist_mut()[idx_case].action = Self::compile_case;
        let idx_of = self.references().idx_of;
        self.wordlist_mut()[idx_of].action = Self::compile_of;
        let idx_endof = self.references().idx_endof;
        self.wordlist_mut()[idx_endof].action = Self::compile_endof;
        let idx_endcase = self.references().idx_endcase;
        self.wordlist_mut()[idx_endcase].action = Self::compile_endcase;
        let idx_begin = self.references().idx_begin;
        self.wordlist_mut()[idx_begin].action = Self::compile_begin;
        let idx_while = self.references().idx_while;
        self.wordlist_mut()[idx_while].action = Self::compile_while;
        let idx_repeat = self.references().idx_repeat;
        self.wordlist_mut()[idx_repeat].action = Self::compile_repeat;
        let idx_until = self.references().idx_until;
        self.wordlist_mut()[idx_until].action = Self::compile_until;
        let idx_again = self.references().idx_again;
        self.wordlist_mut()[idx_again].action = Self::compile_again;
        let idx_do = self.references().idx_do;
        self.wordlist_mut()[idx_do].action = Self::compile_do;
        let idx_qdo = self.references().idx_qdo;
        self.wordlist_mut()[idx_qdo].action = Self::compile_qdo;
        let idx_loop = self.references().idx_loop;
        self.wordlist_mut()[idx_loop].action = Self::compile_loop;
        let idx_plus_loop = self.references().idx_plus_loop;
        self.wordlist_mut()[idx_plus_loop].action = Self::compile_plus_loop;
        let idx_colon = self.references().idx_colon;
        self.wordlist_mut()[idx_colon].action = Self::colon;
        let idx_does = self.references().idx_does;
        self.wordlist_mut()[idx_does].action = Self::does;

        // Words with non default compilation semantics
        let idx_exit = self.references().idx_exit;
        self.wordlist_mut()[idx_exit].compilation_semantics = Self::compile_comma;
        let idx_s_quote = self.references().idx_s_quote;
        self.wordlist_mut()[idx_s_quote].compilation_semantics = Self::compile_comma;
        let idx_unloop = self.references().idx_unloop;
        self.wordlist_mut()[idx_unloop].compilation_semantics = Self::compile_comma;
        let idx_leave = self.references().idx_leave;
        self.wordlist_mut()[idx_leave].compilation_semantics = Self::compile_leave;
        let idx_j = self.references().idx_j;
        self.wordlist_mut()[idx_j].compilation_semantics = Self::compile_comma;

        // Others
        let compile_comma_vector = self.data_space().system_variables().compile_comma_vector();
        let compile_integer_vector = self.data_space().system_variables().compile_integer_vector();
        let compile_var_vector = self.data_space().system_variables().compile_var_vector();
        let compile_const_vector = self.data_space().system_variables().compile_const_vector();
        let compile_fconst_vector = self.data_space().system_variables().compile_fconst_vector();
        let compile_float_vector = self.data_space().system_variables().compile_float_vector();
        unsafe {
            self.data_space()
                .put_isize(Self::comma as isize, compile_comma_vector);
            self.data_space()
                .put_isize(Self::tt_compile_integer as isize, compile_integer_vector);
            self.data_space()
                .put_isize(Self::comma as isize, compile_var_vector);
            self.data_space()
                .put_isize(Self::comma as isize, compile_const_vector);
            self.data_space()
                .put_isize(Self::comma as isize, compile_fconst_vector);
            self.data_space()
                .put_isize(Self::tt_compile_float as isize, compile_float_vector);
        }
    }}

    primitive! {fn branch(&mut self) {
        let ip = self.state().instruction_pointer;
        self.state().instruction_pointer = unsafe{ self.data_space().get_isize(ip) as usize };
    }}

    fn compile_branch(&mut self, destination: usize) -> usize {
        let idx = self.references().idx_branch;
        self.s_stack().push(idx as isize);
        self.compile_comma();
        self.data_space().compile_isize(destination as isize);
        self.data_space().here()
    }

    primitive! {fn zero_branch(&mut self) {
        let v = self.s_stack().pop();
        if v == 0 {
            self.branch();
        } else {
            self.state().instruction_pointer += mem::size_of::<isize>();
        }
    }}

    fn compile_zero_branch(&mut self, destination: usize) -> usize {
        let idx = self.references().idx_zero_branch;
        self.s_stack().push(idx as isize);
        self.compile_comma();
        self.data_space().compile_isize(destination as isize);
        self.data_space().here()
    }

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
    primitive! {fn _do(&mut self) {
        let ip = self.state().instruction_pointer as isize;
        self.r_stack().push(ip);
        self.state().instruction_pointer += mem::size_of::<isize>();
        let (n, t) = self.s_stack().pop2();
        let rt = isize::min_value().wrapping_add(t).wrapping_sub(n);
        let rn = t.wrapping_sub(rt);
        self.r_stack().push2(rn, rt);
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
    primitive! {fn _qdo(&mut self) {
        let (n, t) = self.s_stack().pop2();
        if n == t {
            self.branch();
        } else {
            let ip = self.state().instruction_pointer as isize;
            self.r_stack().push(ip);
            self.state().instruction_pointer += mem::size_of::<isize>();
            let rt = isize::min_value().wrapping_add(t).wrapping_sub(n);
            let rn = t.wrapping_sub(rt);
            self.r_stack().push2(rn, rt);
        }
    }}

    /// Run-time: ( -- ) ( R:  loop-sys1 --  | loop-sys2 )
    ///
    /// An ambiguous condition exists if the loop control parameters are
    /// unavailable. Add one to the loop index. If the loop index is then equal
    /// to the loop limit, discard the loop parameters and continue execution
    /// immediately following the loop. Otherwise continue execution at the
    /// beginning of the loop.
    primitive! {fn _loop(&mut self) {
        let  rt = self.r_stack().pop();
        match rt .checked_add(1) {
            Some(sum) => {
                self.r_stack().push(sum);
                self.branch();
            }
            None => {
                let _ = self.r_stack().pop2();
                self.state().instruction_pointer += mem::size_of::<isize>();
            }
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
    primitive! {fn _plus_loop(&mut self) {
        let rt = self.r_stack().pop();
        let t = self.s_stack().pop();
        match rt.checked_add(t)  {
            Some(sum) => {
                self.r_stack().push(sum);
                self.branch();
            }
            None => {
                let _ = self.r_stack().pop2();
                self.state().instruction_pointer += mem::size_of::<isize>();
            }
        }
    }}

    /// Run-time: ( -- ) ( R: loop-sys -- )
    ///
    /// Discard the loop-control parameters for the current nesting level. An
    /// `UNLOOP` is required for each nesting level before the definition may be
    /// `EXIT`ed. An ambiguous condition exists if the loop-control parameters
    /// are unavailable.
    primitive! {fn unloop(&mut self) {
        let _ = self.r_stack().pop3();
    }}

    primitive! {fn leave(&mut self) {
        let (third, _, _) = self.r_stack().pop3();
        if self.r_stack().underflow() {
            self.abort_with(RETURN_STACK_UNDERFLOW);
            return;
        }
        self.state().instruction_pointer = unsafe{
            self.data_space().get_isize(third as usize) as usize
        };
    }}

    primitive! {fn compile_leave(&mut self) {
        match self.leave_part() {
            Some(leave_part) => {
                self.compile_comma();
            }
            _ => {
                self.abort_with(CONTROL_STRUCTURE_MISMATCH);
                return;
            }
        };
    }}

    primitive! {fn p_j(&mut self) {
        let pos = self.r_stack().len() - 4;
        let jt = self.r_stack().get(pos);
        let jn = self.r_stack().get(pos-1);
        self.s_stack().push(jt.wrapping_add(jn));
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
    primitive! {fn compile_if(&mut self) {
        let here = self.compile_zero_branch(0);
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
    primitive! {fn compile_else(&mut self) {
        let if_part = match self.c_stack().pop() {
            Control::If(if_part) => if_part,
            _ => {
                self.abort_with(CONTROL_STRUCTURE_MISMATCH);
                return;
            }
        };
        if self.c_stack().underflow() {
            self.abort_with(CONTROL_STRUCTURE_MISMATCH);
        } else {
            let here = self.compile_branch(0);
            self.c_stack().push(Control::Else(here));
            unsafe{
                self.data_space()
                .put_isize(here as isize, if_part - mem::size_of::<isize>());
            }
        }
    }}

    primitive! {fn compile_then(&mut self) {
        let branch_part = match self.c_stack().pop() {
            Control::If(branch_part) => branch_part,
            Control::Else(branch_part) => branch_part,
            _ => {
                self.abort_with(CONTROL_STRUCTURE_MISMATCH);
                return;
            }
        };
        if self.c_stack().underflow() {
            self.abort_with(CONTROL_STRUCTURE_MISMATCH);
        } else {
            let here = self.data_space().here();
            unsafe{
                self.data_space()
                    .put_isize(here as isize, branch_part - mem::size_of::<isize>());
            }
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
    primitive! {fn compile_case(&mut self) {
        self.c_stack().push(Control::Case);
    }}

    primitive! {fn compile_of(&mut self) {
        match self.c_stack().pop() {
            Control::Case => {
                self.c_stack().push(Control::Case);
            },
            Control::Endof(n) => {
                self.c_stack().push(Control::Endof(n));
            },
            _ => {
                self.abort_with(CONTROL_STRUCTURE_MISMATCH);
                return;
            }
        };
        if self.c_stack().underflow() {
            self.abort_with(CONTROL_STRUCTURE_MISMATCH);
        } else {
            let idx = self.references().idx_over;
            self.s_stack().push(idx as isize);
            self.compile_comma();
            let idx = self.references().idx_equal;
            self.s_stack().push(idx as isize);
            self.compile_comma();
            let here = self.compile_zero_branch(0);
            self.c_stack().push(Control::Of(here));
            let idx = self.references().idx_drop;
            self.s_stack().push(idx as isize);
            self.compile_comma();
        }
    }}

    primitive! {fn compile_endof(&mut self) {
        let of_part = match self.c_stack().pop() {
            Control::Of(of_part) => {
                of_part
            },
            _ => {
                self.abort_with(CONTROL_STRUCTURE_MISMATCH);
                return;
            }
        };
        if self.c_stack().underflow() {
            self.abort_with(CONTROL_STRUCTURE_MISMATCH);
        } else {
            let here = self.compile_branch(0);
            self.c_stack().push(Control::Endof(here));
            unsafe{
                self.data_space()
                    .put_isize(here as isize, of_part - mem::size_of::<isize>());
            }
        }
    }}

    primitive! {fn compile_endcase(&mut self) {
        let idx = self.references().idx_drop;
        self.s_stack().push(idx as isize);
        self.compile_comma();
        loop {
            let endof_part = match self.c_stack().pop() {
                Control::Case => { break; }
                Control::Endof(endof_part) => {
                    endof_part
                }
                _ => {
                    self.abort_with(CONTROL_STRUCTURE_MISMATCH);
                    return;
                }
            };
            if self.c_stack().underflow() {
                self.abort_with(CONTROL_STRUCTURE_MISMATCH);
            } else {
                let here = self.data_space().here();
                unsafe{
                    self.data_space()
                        .put_isize(here as isize, endof_part - mem::size_of::<isize>());
                }
            }
        }
        if self.c_stack().underflow() {
            self.abort_with(CONTROL_STRUCTURE_MISMATCH);
        }
    }}

    /// Begin a structure that is terminated by `repeat`, `until`, or `again`. `begin ( -- )`.
    primitive! {fn compile_begin(&mut self) {
        let here = self.data_space().here();
        self.c_stack().push(Control::Begin(here));
    }}

    /// Begin the conditional part of a `begin ... while ... repeat` structure. `while ( flag -- )`.
    ///
    /// If all bits of `flag` are zero, continue execution at the location following `repeat`.
    ///
    /// begin A while B repeat C
    ///
    ///                +--------------------+
    ///                |                    |
    ///                |                    v
    /// +---+---------+---+---+--------+---+---+
    /// | A | 0branch | x | B | branch | x | C |
    /// +---+---------+---+---+--------+---+---+
    ///   ^                              |
    ///   |                              |
    ///   +------------------------------+
    ///
    primitive! {fn compile_while(&mut self) {
        let here = self.compile_zero_branch(0);
        self.c_stack().push(Control::While(here));
    }}

    /// Terminate a `begin ... while ... repeat` structure. `repeat ( -- )`.
    ///
    /// Continue execution at the location following `begin`.
    primitive! {fn compile_repeat(&mut self) {
        let (begin_part, while_part) = match self.c_stack().pop2() {
            (Control::Begin(begin_part), Control::While(while_part)) => {
                (begin_part, while_part)
            },
            _ => {
                self.abort_with(CONTROL_STRUCTURE_MISMATCH);
                return;
            }
        };
        if self.c_stack().underflow() {
            self.abort_with(CONTROL_STRUCTURE_MISMATCH);
        } else {
            let here = self.compile_branch(begin_part);
            unsafe{
                self.data_space()
                    .put_isize(here as isize, while_part - mem::size_of::<isize>());
            }
        }
    }}

    /// Terminate a `begin ... until` structure. `until ( flag -- )`.
    ///
    /// If all bits of `flag` are zero, continue execution at the location following `begin`.
    ///
    /// begin A until C
    ///
    /// +---+---------+---+---+
    /// | A | 0branch | x | C |
    /// +---+---------+---+---+
    ///   ^             |
    ///   |             |
    ///   +-------------+
    ///
    primitive! {fn compile_until(&mut self) {
        let begin_part = match self.c_stack().pop() {
            Control::Begin(begin_part) => begin_part,
            _ => {
                self.abort_with(CONTROL_STRUCTURE_MISMATCH);
                return;
            }
        };
        if self.c_stack().underflow() {
            self.abort_with(CONTROL_STRUCTURE_MISMATCH);
        } else {
            self.compile_zero_branch(begin_part);
        }
    }}

    /// Terminate a `begin ... again` structure. `again ( -- )`.
    ///
    /// Continue execution at the location following `begin`.
    ///
    /// begin A again C
    ///
    /// +---+--------+---+---+
    /// | A | branch | x | C |
    /// +---+--------+---+---+
    ///   ^            |
    ///   |            |
    ///   +------------+
    ///
    primitive! {fn compile_again(&mut self) {
        let begin_part = match self.c_stack().pop() {
            Control::Begin(begin_part) => begin_part,
            _ => {
                self.abort_with(CONTROL_STRUCTURE_MISMATCH);
                return;
            }
        };
        if self.c_stack().underflow() {
            self.abort_with(CONTROL_STRUCTURE_MISMATCH);
        } else {
            self.compile_branch(begin_part);
        }
    }}

    /// Clear labels, `0labels ( -- )`
    #[cfg(not(feature = "stc"))]
    primitive! {fn imm_clear_labels(&mut self) {
        self.forward_bitset_mut().clear();
        self.resolved_bitset_mut().clear();
    }}

    /// Create a label `n`, `label ( n -- )`
    ///
    /// Valid `n`: `0 < n < labels.capacity()`.
    #[cfg(not(feature = "stc"))]
    primitive! {fn imm_label(&mut self) {
        let n = self.s_stack().pop() as usize;
        if 0 < n && n < self.labels().capacity() {
            let here = self.data_space().here();
            if self.forward_bitset().contains(n as u32) {
                // Resolve forward references.
                let mut p = self.labels()[n];
                loop {
                    let last = unsafe{ self.data_space().get_usize(p) };
                    unsafe{
                        self.data_space()
                            .put_isize(here as isize, p as usize);
                    }
                    if last == 0 { break; }
                    p = last;
                }
                self.labels_mut()[n] = here;
                self.forward_bitset_mut().remove(n as u32);
                self.resolved_bitset_mut().add(n as u32);
            } else if self.resolved_bitset().contains(n as u32) {
                    self.abort_with(INVALID_NUMERIC_ARGUMENT);
            } else {
                self.labels_mut()[n] = here;
                self.resolved_bitset_mut().add(n as u32);
            }
        } else {
            self.abort_with(INVALID_NUMERIC_ARGUMENT);
        }
    }}

    /// goto ( n -- )
    ///
    /// Go to label `n`.
    ///
    /// +------------+-------------------------+--+---
    /// | BRANCH | data ptr of label n | ... |  addr at label n
    /// +------------+-------------------------+--+---
    ///                         |                                               ^
    ///                         +-----------------------------+
    ///
    /// ```
    /// [ n ] goto ... [ n ] label ...
    /// [ n ] label ... [ n ]  goto
    /// ```
    #[cfg(not(feature = "stc"))]
    primitive! {fn imm_goto(&mut self) {
        let n = self.s_stack().pop() as usize;
        if 0 < n && n < self.labels().capacity() {
            if self.forward_bitset().contains(n as u32) {
                let p = self.labels()[n];
                let to_patch = self.compile_branch(p) - mem::size_of::<isize>();
                self.labels_mut()[n] = to_patch;
            } else if self.resolved_bitset().contains(n as u32) {
                let p = self.labels()[n];
                let _ = self.compile_branch(p);
            } else {
                let to_patch = self.compile_branch(0) - mem::size_of::<isize>();
                self.labels_mut()[n] = to_patch;
                self.forward_bitset_mut().add(n as u32);
            }
        } else {
            self.abort_with(INVALID_NUMERIC_ARGUMENT);
        }
    }}

    /// call ( n -- )
    ///
    /// Call subroutine at label `n`.
    ///
    /// +----+-----------------+----+------------+-------------------------+--+------+
    /// | LIT | return_addr | >R |  BRANCH | data ptr of label n | ... | EXIT |
    /// +----+-----------------+----+------------+-------------------------+--+------+
    ///                       |                                                                                       ^
    ///                       +-------------------------------------------------------+
    ///
    /// Usage:
    ///
    /// ```
    /// [ n ] call ... [ n ] label ... exit ...
    /// [ n ] label .. exit ... [ n ] call ...
    /// ```
    #[cfg(not(feature = "stc"))]
    primitive! {fn imm_call(&mut self) {
         let return_addr = self.data_space().here() + 5 * mem::size_of::<isize>();
         self.s_stack().push(return_addr as _);
         self.compile_integer();
         let idx_to_r = self.references().idx_to_r;
         self.s_stack().push(idx_to_r as isize);
         self.compile_comma();
         self.imm_goto();
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
    primitive! {fn compile_do(&mut self) {
        let idx = self.references().idx__do;
        self.s_stack().push(idx as isize);
        self.compile_comma();
        self.data_space().compile_isize(0);
        let here = self.data_space().here();
        self.c_stack().push(Control::Do(here,here));
    }}

    primitive! {fn imm_recurse(&mut self) {
        let last = self.wordlist().len() - 1;
        self.s_stack().push(last as isize);
        self.compile_comma();
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
    primitive! {fn compile_qdo(&mut self) {
        let idx = self.references().idx__qdo;
        self.s_stack().push(idx as isize);
        self.compile_comma();
        self.data_space().compile_isize(0);
        let here = self.data_space().here();
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
    primitive! {fn compile_loop(&mut self) {
        let do_part = match self.c_stack().pop() {
            Control::Do(do_part,_) => do_part,
            _ => {
                self.abort_with(CONTROL_STRUCTURE_MISMATCH);
                return;
            }
        };
        if self.c_stack().underflow() {
            self.abort_with(CONTROL_STRUCTURE_MISMATCH);
        } else {
            let idx = self.references().idx__loop;
            self.s_stack().push(idx as isize);
            self.compile_comma();
            self.data_space().compile_isize(do_part as isize);
            let here = self.data_space().here();
            unsafe{
                self.data_space()
                    .put_isize(here as isize, (do_part - mem::size_of::<isize>()) as usize);
            }
        }
    }}

    /// Run-time: ( a-addr -- )
    ///
    /// Append the run-time semantics of `_+LOOP` to the current definition.
    /// Resolve the destination of all unresolved occurrences of `LEAVE` between
    /// the location given by do-sys and the next location for a transfer of
    /// control, to execute the words following `+LOOP`.
    primitive! {fn compile_plus_loop(&mut self) {
        let do_part = match self.c_stack().pop() {
            Control::Do(do_part,_) => do_part,
            _ => {
                self.abort_with(CONTROL_STRUCTURE_MISMATCH);
                return;
            }
        };
        if self.c_stack().underflow() {
            self.abort_with(CONTROL_STRUCTURE_MISMATCH);
        } else {
            let idx = self.references().idx__plus_loop;
            self.s_stack().push(idx as isize);
            self.compile_comma();
            self.data_space().compile_isize(do_part as isize);
            let here = self.data_space().here();
            unsafe{
                self.data_space()
                    .put_isize(here as isize, (do_part - mem::size_of::<isize>()) as usize);
            }
        }
    }}

    #[cfg(not(feature = "stc"))]
    primitive! {fn activate(&mut self) {
        let i = (self.s_stack().pop() - 1) as usize;
        if i < NUM_TASKS {
            // Wake task `i`.
            self.set_awake(i, true);
            // Reset task `i` and Assign the code following ACTIVATE to task `i`
            let current_task = self.current_task();
            let ip = self.state().instruction_pointer;
            self.set_current_task(i);
            self.reset();
            self.state().instruction_pointer = ip;
            self.set_current_task(current_task);
            // Return to caller.
            let ip = self.r_stack().pop() as usize;
            self.state().instruction_pointer = ip;
        } else {
            let ip = self.r_stack().pop() as usize;
            self.state().instruction_pointer = ip;
            self.abort_with(INVALID_NUMERIC_ARGUMENT);
        }
    }}

    primitive! {fn p_i(&mut self) {
        let it = self.r_stack().last();
        let next = self.r_stack().len - 2;
        let inext = self.r_stack().get(next);
        self.s_stack().push(it.wrapping_add(inext));
    }}

    // -----------
    // Evaluation
    // -----------

    primitive! {fn left_bracket(&mut self) {
        self.state().is_compiling = false;
    }}

    primitive! {fn right_bracket(&mut self) {
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
    primitive! {fn parse_word(&mut self) {
        let mut last_token = self.last_token().take().expect("token");
        last_token.clear();
        if let Some(input_buffer) = self.input_buffer().take() {
            if self.state().source_index < input_buffer.len() {
                let source = &input_buffer[self.state().source_index..];
                let mut cnt = source.len();
                let mut char_indices = source.char_indices();
                loop {
                    match char_indices.next() {
                        Some((idx, ch)) => {
                            match ch {
                                '\t' | '\n' | '\r' | ' ' => {
                                    if !last_token.is_empty() {
                                        cnt = idx;
                                        break;
                                    }
                                }
                                _ => last_token.push(ch),
                            };
                        }
                        None => {
                            break;
                        }
                    }
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
    primitive! {fn char(&mut self) {
        self.parse_word();
        let last_token = self.last_token().take().expect("token");
        match last_token.chars().nth(0) {
            Some(c) => {
                self.set_last_token(last_token);
                self.s_stack().push(c as isize);
            }
            None => {
                self.set_last_token(last_token);
                self.abort_with(UNEXPECTED_END_OF_FILE);
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
    primitive! {fn bracket_char(&mut self) {
        self.char();
        if self.last_error().is_some() {
            return;
        }
        self.compile_integer();
    }}

    /// Run-time: ( char "ccc&lt;char&gt;" -- )
    ///
    /// Parse ccc delimited by the delimiter char.
    primitive! {fn _parse(&mut self) {
        let input_buffer = self.input_buffer().take().expect("input buffer");
        let v = self.s_stack().pop();
        let mut last_token = self.last_token().take().expect("token");
        last_token.clear();
        {

            let source = &input_buffer[self.state().source_index..];
            let mut cnt = source.len();
            let mut char_indices = source.char_indices();
            loop {
                match char_indices.next() {
                    Some((idx, ch)) => {
                        if ch as isize == v {
                            match char_indices.next() {
                                Some((idx, _)) => {
                                    cnt = idx;
                                }
                                None => {}
                            }
                            break;
                        } else {
                            last_token.push(ch);
                        }
                    }
                    None => {
                        break;
                    }
                }
            }
            self.state().source_index = self.state().source_index + cnt;
        }
        self.set_last_token(last_token);
        self.set_input_buffer(input_buffer);
    }}

    /// Run-time: ( char "ccc" -- )
    ///
    /// Skip all of the delimiter char.
    primitive! {fn _skip(&mut self) {
        let input_buffer = self.input_buffer().take().expect("input buffer");
        let v = self.s_stack().pop();
        {
            let source = &input_buffer[self.state().source_index..];
            let mut cnt = 0;
            let mut char_indices = source.char_indices();
            loop {
                match char_indices.next() {
                    Some((idx, ch)) => {
                        if ch as isize == v {
                            cnt += 1;
                        } else {
                            break;
                        }
                    }
                    None => {
                        break;
                    }
                }
            }
            self.state().source_index = self.state().source_index + cnt;
        }
        self.set_input_buffer(input_buffer);
    }}

    primitive! {fn imm_paren(&mut self) {
        self.s_stack().push(')' as isize);
        self._parse();
    }}

    /// Begin a comment that includes the entire remainder of the current line.
    primitive! {fn imm_backslash(&mut self) {
        self.s_stack().push('\n' as isize);
        self._parse();
    }}

    /// postpone ( "<spaces>name" -- )
    ///
    /// Skip leading space delimiters. Parse name delimited by a space.
    /// Find name. Append the compilation semantics of name to the current definition.
    ///
    /// For example:
    ///
    /// : does>   postpone _does  postpone exit ;
    ///
    /// +------+-------------+-----------+------+------------+-----------+
    /// | _lit | xt of _does | _postpone | _lit | xt of exit | _postpone |
    /// +--------------------+-----------+------+------------+-----------+
    primitive! {fn postpone(&mut self) {
        self.parse_word();
        let last_token = self.last_token().take().expect("token");
        if last_token.is_empty() {
            self.set_last_token(last_token);
            self.abort_with(UNEXPECTED_END_OF_FILE);
        } else {
            match self.find(&last_token) {
                Some(xt) => {
                    self.set_last_token(last_token);
                    self.s_stack().push(xt as _);
                    self.compile_integer();
                    let idx = self.references().idx__postpone;
                    self.s_stack().push(idx as isize);
                    self.compile_comma();
                }
                None => {
                    self.set_last_token(last_token);
                    self.abort_with(UNDEFINED_WORD);
                }
            }
        }
    }}

    /// _POSTPONE ( xt -- )
    ///
    /// Execute the compilation semantics of an xt on stack.
    ///
    /// _POSTPONE is a hidden word which is only compiled by POSTPONE.
    primitive! {fn _postpone(&mut self) {
        // : A ;
        // : B   POSTPONE A ;
        // which generate
        // --+------+---------+-----------+--
        //   | _lit | xt of A | _postpone |
        // --+------+---------+-----------+--
        // Because B comes after A, the xt of A is valid during execution of B.
        let xt = self.s_stack().last() as usize;
        let compilation_semantics = self.wordlist()[xt].compilation_semantics;
        compilation_semantics(self);
    }}

    primitive! {fn compile_token(&mut self) {
        let last_token = self.last_token().take().expect("token");
        match self.find(&last_token) {
            Some(found_index) => {
                self.set_last_token(last_token);
                let compilation_semantics = self.wordlist()[found_index].compilation_semantics;
                if !self.wordlist()[found_index].is_immediate() {
                    self.s_stack().push(found_index as isize);
                    compilation_semantics(self);
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
                self.set_last_token(last_token);
                if done {
                    /* Do nothing. */
                } else {
                    self.abort_with(UNDEFINED_WORD);
                }
            }
        }
    }}

    primitive! {fn interpret_token(&mut self) {
        let last_token = self.last_token().take().expect("last token");
        match self.find(&last_token) {
            Some(found_index) => {
                self.set_last_token(last_token);
                if self.wordlist()[found_index].is_compile_only() {
                    self.abort_with(INTERPRETING_A_COMPILE_ONLY_WORD);
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
                self.set_last_token(last_token);
                if done {
                    /* Do nothing. */
                } else {
                    self.abort_with(UNDEFINED_WORD);
                }
            }
        }
    }}

    primitive! {fn p_compiling(&mut self) {
        let value = if self.state().is_compiling {
            TRUE
        } else {
            FALSE
        };
        self.s_stack().push(value);
    }}

    /// Is token empty? `token-empty? ( -- f )
    primitive! {fn token_empty(&mut self) {
        let value = match self.last_token().as_ref() {
            Some(ref t) => if t.is_empty() { TRUE } else { FALSE },
            None => TRUE,
        };
        self.s_stack().push(value);
    }}

    /// Print token. `.token ( -- )`
    primitive! {fn dot_token(&mut self) {
        match self.last_token().take() {
            Some(t) => {
                match self.output_buffer().as_mut() {
                    Some(buf) => {
                        write!(buf, "{}", t).expect("write token");
                    }
                    None => {}
                }
                self.set_last_token(t);
            }
            None => {}
        }
    }}

    /// Store token. `!token ( c-addr -- )
    ///
    /// Store token in counted string at `c-addr`.`
    primitive! {fn store_token(&mut self) {
        let c_addr = self.s_stack().pop() as usize;
        if self.data_space().start() <= c_addr {
            match self.last_token().take() {
                Some(mut t) => {
                    self.data_space().put_cstr(&t, c_addr);
                    t.clear();
                    self.set_last_token(t);
                }
                None => {
                    unsafe{
                        if c_addr < self.data_space().limit() {
                            self.data_space().put_u8(0, c_addr);
                        } else {
                            panic!("Error: store_token while space is full.");
                        }
                    }
                }
            }
        } else {
            self.abort_with(INVALID_MEMORY_ADDRESS);
        }

    }}

    fn evaluate_input(&mut self) {
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

    primitive! {fn base(&mut self) {
        let base_addr = self.data_space().system_variables().base_addr();
        self.s_stack().push(base_addr as isize);
    }}

    fn evaluate_integer(&mut self, token: &str) {
        let base_addr = self.data_space().system_variables().base_addr();
        let default_base = unsafe { self.data_space().get_isize(base_addr) };
        match parser::quoted_char(&token.as_bytes()) {
            parser::IResult::Done(_bytes, c) => {
                if self.state().is_compiling {
                    self.s_stack().push(c as _);
                    self.compile_integer();
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
                            self.set_error(Some(INTEGER_UNIDENTIFIED_FAULT));
                        } else {
                            if self.state().is_compiling {
                                self.s_stack().push(sign.wrapping_mul(value));
                                self.compile_integer();
                            } else {
                                self.s_stack().push(sign.wrapping_mul(value));
                            }
                        }
                    }
                    parser::IResult::Err(e) => self.set_error(Some(INTEGER_UNIDENTIFIED_FAULT)),
                },
                parser::IResult::Err(e) => {
                    self.set_error(Some(INTEGER_UNIDENTIFIED_FAULT));
                }
            },
            parser::IResult::Err(e) => {
                self.set_error(Some(INTEGER_UNIDENTIFIED_FAULT));
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
                self.set_error(Some(FLOATING_POINT_UNIDENTIFIED_FAULT));
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
                self.set_error(Some(FLOATING_POINT_UNIDENTIFIED_FAULT));
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
                    self.set_error(Some(FLOATING_POINT_UNIDENTIFIED_FAULT));
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
                                self.set_error(Some(FLOATING_POINT_UNIDENTIFIED_FAULT));
                                return;
                            }
                        }
                        match parser::uint(bytes) {
                            parser::IResult::Done(input, value) => {
                                exponent_part = value;
                                bytes = input;
                            }
                            parser::IResult::Err(e) => {
                                self.set_error(Some(FLOATING_POINT_UNIDENTIFIED_FAULT));
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
                                            self.set_error(Some(FLOATING_POINT_UNIDENTIFIED_FAULT));
                                            return;
                                        }
                                    }
                                    match parser::uint(bytes) {
                                        parser::IResult::Done(input, value) => {
                                            exponent_part = value;
                                            bytes = input;
                                        }
                                        parser::IResult::Err(e) => {
                                            self.set_error(Some(FLOATING_POINT_UNIDENTIFIED_FAULT));
                                            return;
                                        }
                                    }
                                }
                            }
                            parser::IResult::Err(e) => {
                                self.set_error(Some(FLOATING_POINT_UNIDENTIFIED_FAULT));
                                return;
                            }
                        }
                    }
                }
                parser::IResult::Err(e) => {
                    self.set_error(Some(FLOATING_POINT_UNIDENTIFIED_FAULT));
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
            self.set_error(Some(FLOATING_POINT_UNIDENTIFIED_FAULT))
        } else {
            if self.references().idx_flit == 0 {
                self.set_error(Some(FLOATING_POINT_UNIDENTIFIED_FAULT));
            } else {
                let value = (significand_sign as f64)
                    * (integer_part as f64 + fraction_part)
                    * ((10.0f64).powi((exponent_sign.wrapping_mul(exponent_part)) as i32) as f64);
                if self.state().is_compiling {
                    self.f_stack().push(value);
                    self.compile_float();
                } else {
                    self.f_stack().push(value);
                }
            }
        }
    }

    // -----------------------
    // High level definitions
    // -----------------------

    primitive! {fn nest(&mut self) {
        let rlen = self.r_stack().len.wrapping_add(1);
        self.r_stack().len = rlen;
        self.r_stack()[rlen.wrapping_sub(1)] = self.state().instruction_pointer as isize;
        let wp = self.state().word_pointer;
        self.state().instruction_pointer = self.wordlist()[wp].dfa();
    }}

    primitive! {fn p_var(&mut self) {
        let wp = self.state().word_pointer;
        let dfa = self.wordlist()[wp].dfa() as isize;
        self.s_stack().push(dfa);
    }}

    primitive! {fn p_const(&mut self) {
        let wp = self.state().word_pointer;
        let dfa = self.wordlist()[wp].dfa();
        let value = unsafe{ self.data_space().get_isize(dfa) as isize };
        self.s_stack().push(value);
    }}

    fn define(
        &mut self,
        word_type: WordType,
        action: primitive! {fn(&mut Self)},
        compilation_semantics: primitive! { fn(&mut Self) },
    ) {
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
            self.set_last_token(last_token);
            self.abort_with(UNEXPECTED_END_OF_FILE);
        } else {
            let nfa = self.data_space().compile_str(&last_token);
            self.data_space().align();
            self.code_space().align();
            let word = Word::new(
                word_type,
                action,
                compilation_semantics,
                nfa,
                self.data_space().here(),
                self.code_space().here(),
            );
            self.wordlist_mut().push(&last_token, word);
            self.set_last_token(last_token);
        }
    }

    primitive! {fn colon(&mut self) {
        self.define(WordType::Nest, Core::nest, Core::compile_comma);
        if self.last_error().is_none() {
            let def = self.wordlist().last;
            self.wordlist_mut()[def].set_hidden(true);
            self.right_bracket();
        }
    }}

    primitive! {fn semicolon(&mut self) {
        if self.c_stack().len != 0 {
            self.abort_with(CONTROL_STRUCTURE_MISMATCH);
        } else if self.forward_bitset().layer3() != 0 {
            self.abort_with(CONTROL_STRUCTURE_MISMATCH);
        } else {
            let idx = self.references().idx_exit;
            let compile_exit = self.wordlist()[idx].compilation_semantics;
            self.s_stack().push(idx as isize);
            compile_exit(self);
            let def = self.wordlist().last;
            self.wordlist_mut()[def].set_hidden(false);
        }
        self.left_bracket();
    }}

    primitive! {fn create(&mut self) {
        self.define(WordType::Var, Core::p_var, Core::compile_var);
    }}

    primitive! {fn constant(&mut self) {
        let v = self.s_stack().pop();
        self.define(WordType::Const, Core::p_const, Core::compile_const);
        if self.last_error().is_none() {
            self.data_space().compile_isize(v as isize);
        }
    }}

    primitive! {fn unmark(&mut self) {
        let wp = self.state().word_pointer;
        let (nfa, cfa, mut dfa) = {
            let w = &self.wordlist()[wp];
            (w.nfa(), w.cfa(), w.dfa())
        };
        let x = unsafe{ self.data_space().get_usize(dfa) };
        self.wordlist_mut().last = x;
        for i in 0..BUCKET_SIZE {
            dfa += mem::size_of::<usize>();
            let x = unsafe{ self.data_space().get_usize(dfa) };
            self.wordlist_mut().buckets[i] = x;
        }
        self.data_space().truncate(nfa);
        self.code_space().truncate(cfa);
        self.wordlist_mut().truncate(wp);
    }}

    /// Example:
    /// ```text
    /// marker -work
    ///
    /// DFA of -work
    /// +------+--------+
    /// | last | b0-b63 |
    /// +------+--------+
    /// ```
    primitive! {fn marker(&mut self) {
        let x = self.wordlist().last;
        self.wordlist_mut().temp_buckets = self.wordlist().buckets;
        self.define(WordType::Marker, Core::unmark, Core::compile_comma);
        self.data_space().compile_usize(x);
        for i in 0..BUCKET_SIZE {
            let x = self.wordlist().temp_buckets[i];
            self.data_space().compile_usize(x);
        }
    }}

    /// Run time behavior of words created by `create` ... `does>`.
    ///
    /// Token threaded version.
    ///
    /// Example of does>
    /// ```text
    /// : 2constant   create , , does> 2@ ;
    /// 4 40 2constant range
    /// ```
    ///
    /// 2constant
    /// +--------+---+---+-------+------+----+------+
    /// | create | , | , | _does | exit | 2@ | exit |
    /// +--------+---+---+-------+------+----+------+
    ///                                   ^
    /// range                             |
    ///                                   |
    ///   action                          |
    ///   +-------+                       |
    ///   | xdoes |                       |
    ///   +-------+                       |
    ///                                   |
    ///   cfa                             |
    ///   +------+                        |
    ///   | x    |------------------------+
    ///   +------+
    ///
    ///   dfa
    ///   +---+----+
    ///   | 4 | 40 |
    ///   +---+----+
    ///
    primitive!{fn does(&mut self) {
        let idx = self.references().idx__does;
        self.s_stack().push(idx as isize);
        self.compile_comma();
        let idx = self.references().idx_exit;
        self.s_stack().push(idx as isize);
        self.compile_comma();
    }}

    primitive! {fn xdoes(&mut self) {
        // Push DFA.
        let wp = self.state().word_pointer;
        let dfa = self.wordlist()[wp].dfa();
        let cfa = self.wordlist()[wp].cfa();
        self.s_stack().push(dfa as isize);
        // Execute words behind DOES>.
        let doer = unsafe{ self.code_space().get_usize(cfa) };
        let ip = self.state().instruction_pointer as isize;
        self.r_stack().push(ip);
        self.state().instruction_pointer = doer;
    }}

    /// Run time behavior of does>.
    primitive! {fn _does(&mut self) {
        let doer = self.state().instruction_pointer + mem::size_of::<isize>();
        self.code_space().compile_usize(doer);
        let def = self.wordlist().last;
        let word = &mut self.wordlist_mut()[def];
        word.action = Core::xdoes;
    }}

    // -----------
    // Primitives
    // -----------

    /// Run-time: ( -- )
    ///
    /// No operation
    primitive! {fn noop(&mut self) {
        // Do nothing
    }}

    /// Run-time: ( -- true )
    ///
    /// Return a true flag, a single-cell value with all bits set.
    primitive! {fn p_true(&mut self) {
        self.s_stack().push(TRUE);
    }}

    /// Run-time: ( -- false )
    ///
    /// Return a false flag.
    primitive! {fn p_false(&mut self) {
        self.s_stack().push(FALSE);
    }}

    /// Run-time: ( c-addr1 -- c-addr2 )
    ///
    ///Add the size in address units of a character to `c-addr1`, giving `c-addr2`.
    primitive! {fn char_plus(&mut self) {
        let v = self.s_stack().pop();
        self.s_stack().push(v + mem::size_of::<u8>() as isize);
    }}

    /// Run-time: ( a-addr1 -- a-addr2 )
    ///
    /// Add the size in address units of a cell to `a-addr1`, giving `a-addr2`.
    primitive! {fn cell_plus(&mut self) {
        let v = self.s_stack().pop();
        self.s_stack().push(v + mem::size_of::<isize>() as isize);
    }}

    /// Run-time: ( n1 -- n2 )
    ///
    /// `n2` is the size in address units of `n1` cells.
    primitive! {fn cells(&mut self) {
        let v = self.s_stack().pop();
        self.s_stack().push(v * mem::size_of::<isize>() as isize);
    }}

    primitive! {fn swap(&mut self) {
        let slen = self.s_stack().len;
        let t = self.s_stack()[slen.wrapping_sub(1)];
        let n = self.s_stack()[slen.wrapping_sub(2)];
        self.s_stack()[slen.wrapping_sub(1)] = n;
        self.s_stack()[slen.wrapping_sub(2)] = t;
    }}

    primitive! {fn dup(&mut self) {
        let slen = self.s_stack().len.wrapping_add(1);
        self.s_stack().len = slen;
        self.s_stack()[slen.wrapping_sub(1)] = self.s_stack()[slen.wrapping_sub(2)];
    }}

    primitive! {fn p_drop(&mut self) {
        let slen = self.s_stack().len.wrapping_sub(1);
        self.s_stack().len = slen;
    }}

    primitive! {fn pop_s_stack(&mut self) -> isize {
        let slen = self.s_stack().len.wrapping_sub(1);
        let t = self.s_stack()[slen];
        self.s_stack().len = slen;
        t
    }}

    primitive! {fn nip(&mut self) {
        let slen = self.s_stack().len.wrapping_sub(1);
        let t = self.s_stack()[slen];
        self.s_stack().len = slen;
        self.s_stack()[slen.wrapping_sub(1)] = t;
    }}

    primitive! {fn over(&mut self) {
        let slen = self.s_stack().len.wrapping_add(1);
        self.s_stack().len = slen;
        self.s_stack()[slen.wrapping_sub(1)] = self.s_stack()[slen.wrapping_sub(3)];
    }}

    primitive! {fn rot(&mut self) {
        let slen = self.s_stack().len;
        let t = self.s_stack()[slen.wrapping_sub(1)];
        let n = self.s_stack()[slen.wrapping_sub(2)];
        self.s_stack()[slen.wrapping_sub(1)] = self.s_stack()[slen.wrapping_sub(3)];
        self.s_stack()[slen.wrapping_sub(2)] = t;
        self.s_stack()[slen.wrapping_sub(3)] = n;
    }}

    primitive! {fn minus_rot(&mut self) {
        let slen = self.s_stack().len;
        let t = self.s_stack()[slen.wrapping_sub(1)];
        let n = self.s_stack()[slen.wrapping_sub(2)];
        self.s_stack()[slen.wrapping_sub(2)] = self.s_stack()[slen.wrapping_sub(3)];
        self.s_stack()[slen.wrapping_sub(3)] = t;
        self.s_stack()[slen.wrapping_sub(1)] = n;
    }}

    /// Place a copy of the nth stack entry on top of the stack. `pick ( ... n -- x )`
    ///
    /// `0 pick` is equivalent to `dup`.
    primitive! {fn pick(&mut self) {
        let slen = self.s_stack().len;
        let t = self.s_stack()[slen.wrapping_sub(1)] as u8;
        let x = self.s_stack()[slen.wrapping_sub(t.wrapping_add(2))];
        self.s_stack()[slen.wrapping_sub(1)] = x;
    }}

    primitive! {fn two_drop(&mut self) {
        let slen = self.s_stack().len.wrapping_sub(2);
        self.s_stack().len = slen;
    }}

    primitive! {fn two_dup(&mut self) {
        let slen = self.s_stack().len.wrapping_add(2);
        self.s_stack().len = slen;
        self.s_stack()[slen.wrapping_sub(1)] = self.s_stack()[slen.wrapping_sub(3)];
        self.s_stack()[slen.wrapping_sub(2)] = self.s_stack()[slen.wrapping_sub(4)];
    }}

    primitive! {fn two_swap(&mut self) {
        let slen = self.s_stack().len;
        let t = self.s_stack()[slen.wrapping_sub(1)];
        let n = self.s_stack()[slen.wrapping_sub(2)];
        self.s_stack()[slen.wrapping_sub(1)] = self.s_stack()[slen.wrapping_sub(3)];
        self.s_stack()[slen.wrapping_sub(2)] = self.s_stack()[slen.wrapping_sub(4)];
        self.s_stack()[slen.wrapping_sub(3)] = t;
        self.s_stack()[slen.wrapping_sub(4)] = n;
    }}

    primitive! {fn two_over(&mut self) {
        let slen = self.s_stack().len.wrapping_add(2);
        self.s_stack().len = slen;
        self.s_stack()[slen.wrapping_sub(1)] = self.s_stack()[slen.wrapping_sub(5)];
        self.s_stack()[slen.wrapping_sub(2)] = self.s_stack()[slen.wrapping_sub(6)];
    }}

    primitive! {fn depth(&mut self) {
        let len = self.s_stack().len;
        self.s_stack().push(len as isize);
    }}

    primitive! {fn one_plus(&mut self) {
        let slen = self.s_stack().len;
        let t = self.s_stack()[slen.wrapping_sub(1)];
        self.s_stack()[slen.wrapping_sub(1)] = t.wrapping_add(1);
    }}

    primitive! {fn one_minus(&mut self) {
        let slen = self.s_stack().len;
        let t = self.s_stack()[slen.wrapping_sub(1)];
        self.s_stack()[slen.wrapping_sub(1)] = t.wrapping_sub(1);
    }}

    primitive! {fn plus(&mut self) {
        let slen = self.s_stack().len;
        let t = self.s_stack()[slen.wrapping_sub(1)];
        let n = self.s_stack()[slen.wrapping_sub(2)];
        self.s_stack()[slen.wrapping_sub(2)] = n.wrapping_add(t);
        self.s_stack().len = slen.wrapping_sub(1);
    }}

    primitive! {fn minus(&mut self) {
        let slen = self.s_stack().len;
        let t = self.s_stack()[slen.wrapping_sub(1)];
        let n = self.s_stack()[slen.wrapping_sub(2)];
        self.s_stack()[slen.wrapping_sub(2)] = n.wrapping_sub(t);
        self.s_stack().len = slen.wrapping_sub(1);
    }}

    primitive! {fn star(&mut self) {
        let slen = self.s_stack().len;
        let t = self.s_stack()[slen.wrapping_sub(1)];
        let n = self.s_stack()[slen.wrapping_sub(2)];
        self.s_stack()[slen.wrapping_sub(2)] = n.wrapping_mul(t);
        self.s_stack().len = slen.wrapping_sub(1);
    }}

    primitive! {fn slash(&mut self) {
        let slen = self.s_stack().len;
        let t = self.s_stack()[slen.wrapping_sub(1)];
        let n = self.s_stack()[slen.wrapping_sub(2)];
        if t == 0 {
            self.abort_with(DIVISION_BY_ZERO);
        } else {
            self.s_stack()[slen.wrapping_sub(2)] = n.wrapping_div(t);
            self.s_stack().len = slen.wrapping_sub(1);
        }
    }}

    primitive! {fn p_mod(&mut self) {
        let slen = self.s_stack().len;
        let t = self.s_stack()[slen.wrapping_sub(1)];
        let n = self.s_stack()[slen.wrapping_sub(2)];
        if t == 0 {
            self.abort_with(DIVISION_BY_ZERO);
        } else {
            self.s_stack()[slen.wrapping_sub(2)] = n.wrapping_rem(t);
            self.s_stack().len = slen.wrapping_sub(1);
        }
    }}

    primitive! {fn slash_mod(&mut self) {
        let slen = self.s_stack().len;
        let t = self.s_stack()[slen.wrapping_sub(1)];
        let n = self.s_stack()[slen.wrapping_sub(2)];
        if t == 0 {
            self.abort_with(DIVISION_BY_ZERO);
        } else {
            self.s_stack()[slen.wrapping_sub(2)] = n.wrapping_rem(t);
            self.s_stack()[slen.wrapping_sub(1)] = n.wrapping_div(t);
        }
    }}

    primitive! {fn abs(&mut self) {
        let t = self.s_stack().pop();
        self.s_stack().push(t.wrapping_abs());
    }}

    primitive! {fn negate(&mut self) {
        let t = self.s_stack().pop();
        self.s_stack().push(t.wrapping_neg());
    }}

    primitive! {fn zero_less(&mut self) {
        let t = self.s_stack().pop();
        self.s_stack().push(if t < 0 { TRUE } else { FALSE });
    }}

    primitive! {fn zero_equals(&mut self) {
        let t = self.s_stack().pop();
        self.s_stack().push(if t == 0 { TRUE } else { FALSE });
    }}

    primitive! {fn zero_greater(&mut self) {
        let t = self.s_stack().pop();
        self.s_stack().push(if t > 0 { TRUE } else { FALSE });
    }}

    primitive! {fn zero_not_equals(&mut self) {
        let t = self.s_stack().pop();
        self.s_stack().push(if t == 0 { FALSE } else { TRUE });
    }}

    primitive! {fn equals(&mut self) {
        let (n, t) = self.s_stack().pop2();
        self.s_stack().push(if t == n { TRUE } else { FALSE });
    }}

    primitive! {fn less_than(&mut self) {
        let (n, t) = self.s_stack().pop2();
        self.s_stack().push(if n < t { TRUE } else { FALSE });
    }}

    primitive! {fn greater_than(&mut self) {
        let (n, t) = self.s_stack().pop2();
        self.s_stack().push(if n > t { TRUE } else { FALSE });
    }}

    primitive! {fn not_equals(&mut self) {
        let (n, t) = self.s_stack().pop2();
        self.s_stack().push(if n == t { FALSE } else { TRUE });
    }}

    /// `within` ( n1 n2 n3 -- flag )  true if n2 <= n1 and n1 < n3.
    ///
    /// Note: implmenetation incompatible with Forth 2012 standards
    /// when n2 > n3.
    primitive! {fn within(&mut self) {
        let (x1, x2, x3) = self.s_stack().pop3();
        self.s_stack()
            .push(if x2 <= x1 && x1 < x3 { TRUE } else { FALSE });
    }}

    primitive! {fn invert(&mut self) {
        let t = self.s_stack().pop();
        self.s_stack().push(!t);
    }}

    primitive! {fn and(&mut self) {
        let (n, t) = self.s_stack().pop2();
        self.s_stack().push(t & n);
    }}

    primitive! {fn or(&mut self) {
        let (n, t) = self.s_stack().pop2();
        self.s_stack().push(t | n);
    }}

    primitive! {fn xor(&mut self) {
        let (n, t) = self.s_stack().pop2();
        self.s_stack().push(t ^ n);
    }}

    /// Run-time: ( x1 u -- x2 )
    ///
    /// Perform a logical left shift of `u` bit-places on `x1`, giving `x2`. Put
    /// zeroes into the least significant bits vacated by the shift. An
    /// ambiguous condition exists if `u` is greater than or equal to the number
    /// of bits in a cell.
    primitive! {fn lshift(&mut self) {
        let (n, t) = self.s_stack().pop2();
        self.s_stack().push(n.wrapping_shl(t as u32));
    }}

    /// Run-time: ( x1 u -- x2 )
    ///
    /// Perform a logical right shift of `u` bit-places on `x1`, giving `x2`. Put
    /// zeroes into the most significant bits vacated by the shift. An
    /// ambiguous condition exists if `u` is greater than or equal to the number
    /// of bits in a cell.
    primitive! {fn rshift(&mut self) {
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
    primitive! {fn exit(&mut self) {
        let rlen = self.r_stack().len.wrapping_sub(1);
        self.state().instruction_pointer = self.r_stack()[rlen] as usize;
        self.r_stack().len = rlen;
    }}

    /// Run-time: ( a-addr -- x )
    ///
    /// `x` is the value stored at `a-addr`.
    primitive! {fn fetch(&mut self) {
        let t = self.s_stack().pop() as usize;
        if self.data_space().start() <= t &&
            t + mem::size_of::<isize>() <=self.data_space().limit()
        {
            let value = unsafe{ self.data_space().get_isize(t as usize) as isize };
            self.s_stack().push(value);
        } else if self.code_space().start() <= t &&
            t + mem::size_of::<isize>() <= self.code_space().limit()
        {
            let value = unsafe{ self.code_space().get_isize(t as usize) as isize };
            self.s_stack().push(value);
        } else {
            self.abort_with(INVALID_MEMORY_ADDRESS);
        }
    }}

    /// Run-time: ( x a-addr -- )
    ///
    /// Store `x` at `a-addr`.
    primitive! {fn store(&mut self) {
        let (n, t) = self.s_stack().pop2();
        let t = t as usize;
        if self.data_space().start() < t &&
            t + mem::size_of::<isize>() <= self.data_space().limit()
        {
            unsafe{ self.data_space().put_isize(n as isize, t as usize) };
        } else {
            self.abort_with(INVALID_MEMORY_ADDRESS);
        }
    }}

    /// Run-time: ( c-addr -- char )
    ///
    /// Fetch the character stored at `c-addr`. When the cell size is greater than
    /// character size, the unused high-order bits are all zeroes.
    primitive! {fn c_fetch(&mut self) {
        let t = self.s_stack().pop() as usize;
        if self.data_space().start() <= t &&
            t < self.data_space().limit()
        {
            let value = unsafe{ self.data_space().get_u8(t as usize) as isize };
            self.s_stack().push(value);
        } else if self.code_space().start() <= t &&
            t < self.code_space().limit()
        {
            let value = unsafe{ self.code_space().get_u8(t as usize) as isize };
            self.s_stack().push(value);
        } else {
            self.abort_with(INVALID_MEMORY_ADDRESS);
        }
    }}

    /// Run-time: ( char c-addr -- )
    ///
    /// Store `char` at `c-addr`. When character size is smaller than cell size,
    /// only the number of low-order bits corresponding to character size are
    /// transferred.
    primitive! {fn c_store(&mut self) {
        let (n, t) = self.s_stack().pop2();
        let t = t as usize;
        if self.data_space().start() < t && t  < self.data_space().limit() {
            unsafe{ self.data_space().put_u8(n as u8, t as usize) };
        } else {
            self.abort_with(INVALID_MEMORY_ADDRESS);
        }
    }}

    /// Run-time: ( addr1 addr2 u -- )
    ///
    /// If u is greater than zero, copy the contents of u consecutive address
    /// units at addr1 to the u consecutive address units at addr2. After MOVE
    /// completes, the u consecutive address units at addr2 contain exactly
    /// what the u consecutive address units at addr1 contained before the
    /// move.
    primitive! {fn p_move(&mut self) {
        let (addr1, addr2, u) = self.s_stack().pop3();
        if u > 0 {
            let u = u as usize;
            let addr1 = addr1 as usize;
            let addr2 = addr2 as usize;
            if
                self.data_space().start() < addr1
                && addr1 + u  <= self.data_space().limit()
                && self.data_space().start() < addr2
                && addr2 + u  <= self.data_space().limit()
            {
                unsafe{
                    if addr1 < addr2 {
                        for p in (addr1..(addr1+u)).into_iter().zip(addr2..(addr2+u)).rev() {
                            let value = self.data_space().get_u8(p.0);
                            self.data_space().put_u8(value, p.1);
                        }
                    } else {
                        for p in (addr1..(addr1+u)).into_iter().zip(addr2..(addr2+u)) {
                            let value = self.data_space().get_u8(p.0);
                            self.data_space().put_u8(value, p.1);
                        }
                    }
                }
            } else {
                self.abort_with(INVALID_MEMORY_ADDRESS);
            }
        }
    }}

    /// Run-time: ( "<spaces>name" -- xt )
    ///
    /// Skip leading space delimiters. Parse name delimited by a space. Find
    /// `name` and return `xt`, the execution token for name. An ambiguous
    /// condition exists if name is not found.
    primitive! {fn tick(&mut self) {
        self.parse_word();
        let last_token = self.last_token().take().expect("last token");
        if last_token.is_empty() {
            self.set_last_token(last_token);
            self.abort_with(UNEXPECTED_END_OF_FILE);
        } else {
            match self.find(&last_token) {
                Some(found_index) => {
                    self.s_stack().push(found_index as isize);
                    self.set_last_token(last_token);
                }
                None => {
                    self.set_last_token(last_token);
                    self.abort_with(UNDEFINED_WORD);
                }
            }
        }
    }}

    /// ( xt -- a-addr )
    /// a-addr is the data-field address corresponding to xt. An ambiguous
    /// condition exists if xt is not for a word defined via CREATE.
    primitive! {fn to_body(&mut self) {
        let t = self.s_stack().pop() as usize;
        if t < self.wordlist().len() {
            let dfa = self.wordlist()[t].dfa() as isize;
            self.s_stack().push(dfa);
        } else {
            self.abort_with(INVALID_NUMERIC_ARGUMENT);
        }
    }}

    /// Run-time: ( i*x xt -- j*x )
    ///
    /// Remove `xt` from the stack and perform the semantics identified by it.
    /// Other stack effects are due to the word `EXECUTE`d.
    primitive! {fn execute(&mut self) {
        let t = self.s_stack().pop();
        self.execute_word(t as usize);
    }}

    /// Compilation: ( "<spaces>name" -- )
    /// Run-time: ( -- xt )
    ///
    /// Forth 2012 6.1.2510
    primitive! {fn bracket_tick(&mut self) {
        self.parse_word();
        let last_token = self.last_token().take().expect("last token");
        if last_token.is_empty() {
            self.set_last_token(last_token);
            self.abort_with(UNEXPECTED_END_OF_FILE);
        } else {
            match self.find(&last_token) {
                Some(found_index) => {
                    self.s_stack().push(found_index as _);
                    self.compile_integer();
                    self.set_last_token(last_token);
                }
                None => {
                    self.set_last_token(last_token);
                    self.abort_with(UNDEFINED_WORD);
                }
            }
        }
    }}

    /// Execution: ( xt -- )
    ///
    /// Forth 2012 6.2.0945
    /// Append the execution semantics of the definition represented by xt to the execution semantics of the current definition.
    primitive! {fn compile_comma(&mut self) {
        let compile_comma_vector = self.data_space().system_variables().compile_comma_vector();
        unsafe {
            let compile_comma_vector: *const primitive!{fn (&mut Self)} = mem::transmute (compile_comma_vector);
            (*compile_comma_vector)(self);
        }
    }}

    /// Run-time: ( -- addr )
    ///
    /// `addr` is the data-space pointer.
    primitive! {fn here(&mut self) {
        let here = self.data_space().here() as isize;
        self.s_stack().push(here);
    }}

    /// Run-time: ( n -- )
    ///
    /// If `n` is greater than zero, reserve n address units of data space. If `n`
    /// is less than zero, release `|n|` address units of data space. If `n` is
    /// zero, leave the data-space pointer unchanged.
    primitive! {fn allot(&mut self) {
        let v = self.s_stack().pop();
        self.data_space().allot(v);
    }}

    /// Run-time: ( addr -- a-addr )
    ///
    /// Return `a-addr`, the first aligned address greater than or equal to `addr`.
    primitive! {fn aligned(&mut self) {
        let pos = self.s_stack().pop();
        let pos = DataSpace::aligned(pos as usize);
        self.s_stack().push(pos as isize);
    }}

    /// Run-time: ( -- )
    ///
    /// If the data-space pointer is not aligned, reserve enough space to align it.
    primitive! {fn align(&mut self) {
        self.data_space().align();
    }}

    /// Run-time: ( x -- )
    ///
    /// Reserve one cell of data space and store `x` in the cell. If the
    /// data-space pointer is aligned when `,` begins execution, it will remain
    /// aligned when `,` finishes execution. An ambiguous condition exists if the
    /// data-space pointer is not aligned prior to execution of `,`.
    primitive! {fn comma(&mut self) {
        let v = self.s_stack().pop();
        self.data_space().compile_isize(v as isize);
    }}

    primitive! {fn p_to_r(&mut self) {
        let slen = self.s_stack().len;
        let rlen = self.r_stack().len.wrapping_add(1);
        self.r_stack().len = rlen;
        self.r_stack()[rlen.wrapping_sub(1)] = self.s_stack()[slen.wrapping_sub(1)];
        self.s_stack().len = slen.wrapping_sub(1);
    }}

    primitive! {fn r_from(&mut self) {
        let slen = self.s_stack().len.wrapping_add(1);
        let rlen = self.r_stack().len;
        self.s_stack().len = slen;
        self.s_stack()[slen.wrapping_sub(1)] = self.r_stack()[rlen.wrapping_sub(1)];
        self.r_stack().len = rlen.wrapping_sub(1);
    }}

    primitive! {fn r_fetch(&mut self) {
        let slen = self.s_stack().len.wrapping_add(1);
        let rlen = self.r_stack().len;
        self.s_stack().len = slen;
        self.s_stack()[slen.wrapping_sub(1)] = self.r_stack()[rlen.wrapping_sub(1)];
    }}

    primitive! {fn two_to_r(&mut self) {
        let slen = self.s_stack().len;
        let rlen = self.r_stack().len.wrapping_add(2);
        self.r_stack().len = rlen;
        self.r_stack()[rlen.wrapping_sub(2)] = self.s_stack()[slen.wrapping_sub(2)];
        self.r_stack()[rlen.wrapping_sub(1)] = self.s_stack()[slen.wrapping_sub(1)];
        self.s_stack().len = slen.wrapping_sub(2);
    }}

    primitive! {fn two_r_from(&mut self) {
        let slen = self.s_stack().len.wrapping_add(2);
        let rlen = self.r_stack().len;
        self.s_stack().len = slen;
        self.s_stack()[slen.wrapping_sub(2)] = self.r_stack()[rlen.wrapping_sub(2)];
        self.s_stack()[slen.wrapping_sub(1)] = self.r_stack()[rlen.wrapping_sub(1)];
        self.r_stack().len = rlen.wrapping_sub(2);
    }}

    primitive! {fn two_r_fetch(&mut self) {
        let slen = self.s_stack().len.wrapping_add(2);
        let rlen = self.r_stack().len;
        self.s_stack().len = slen;
        self.s_stack()[slen.wrapping_sub(2)] = self.r_stack()[rlen.wrapping_sub(2)];
        self.s_stack()[slen.wrapping_sub(1)] = self.r_stack()[rlen.wrapping_sub(1)];
    }}

    // ----------------
    // Error handlling
    // ----------------

    primitive! {fn check_stacks(&mut self) {
        if self.s_stack().overflow() {
            self.abort_with(STACK_OVERFLOW);
        } else if self.s_stack().underflow() {
            self.abort_with(STACK_UNDERFLOW);
        } else if self.r_stack().overflow() {
            self.abort_with(RETURN_STACK_OVERFLOW);
        } else if self.r_stack().underflow() {
            self.abort_with(RETURN_STACK_UNDERFLOW);
        } else if self.c_stack().overflow() {
            self.abort_with(CONTROL_STRUCTURE_MISMATCH);
        } else if self.c_stack().underflow() {
            self.abort_with(CONTROL_STRUCTURE_MISMATCH);
        } else if self.f_stack().overflow() {
            self.abort_with(FLOATING_POINT_STACK_OVERFLOW);
        } else if self.f_stack().underflow() {
            self.abort_with(FLOATING_POINT_STACK_UNDERFLOW);
        }
    }}

    primitive! {fn handler_store(&mut self) {
        let t = self.s_stack().pop();
        self.set_handler(t as usize);
    }}

    /// Error code `error ( -- n )`
    primitive! {fn error(&mut self) {
        match self.last_error() {
            Some(e) => {
                self.s_stack().push(e as isize);
            }
            None => {
                self.s_stack().push(0);
            }
        }
    }}

    /// Clear error. `0error ( -- )`
    primitive! {fn clear_error(&mut self) {
        self.set_error(None);
    }}

    /// Print error description. `.error ( -- )`
    primitive! {fn dot_error(&mut self) {
        match self.last_error() {
            Some(e) => {
                match self.output_buffer().as_mut() {
                    Some(buf) => {
                        write!(buf, "{}", exception::description(e)).expect("write");
                    }
                    None => {}
                }
            }
            None => {}
        }
    }}

    /// Clear data, floating point, and control stacks.
    /// Called by VM's client upon ABORT.
    primitive! {fn clear_stacks(&mut self) {
        self.s_stack().reset();
        self.f_stack().reset();
        self.c_stack().reset();
    }}

    /// ( -- source-id )
    ///
    /// Current source id.
    primitive! {fn p_source_id(&mut self) {
        let source_id = self.source_id();
        self.s_stack().push(source_id);
    }}

    /// ( source-id -- )
    ///
    /// Set source id.
    primitive! {fn p_set_source_id(&mut self) {
        let id = self.s_stack().pop();
        self.set_source_id(id);
    }}

    /// Set source id.
    fn set_source_id(&mut self, id: isize) {
        if id > 0 {
            // File source
            if id - 1 < self.sources().len() as isize
                && self.sources()[id as usize - 1].is_some()
                && self.lines()[id as usize - 1].is_some()
            {
                self.state().source_id = id;
            } else {
                self.abort_with(INVALID_NUMERIC_ARGUMENT);
            }
        } else if id == 0 {
            self.state().source_id = id;
        } else {
            self.abort_with(INVALID_NUMERIC_ARGUMENT);
        }
    }

    /// ( -- source-idx )
    ///
    /// Current source index.
    primitive! {fn p_source_idx(&mut self) {
        let source_idx = self.state().source_index as isize;
        self.s_stack().push(source_idx);
    }}

    /// ( source-idx -- )
    ///
    /// Set source index.
    primitive! {fn p_set_source_idx(&mut self) {
        let idx = self.s_stack().pop() as usize;
        self.state().source_index = idx;
    }}

    /// Reset VM, do not clear data stack, floating point and control stack.
    /// Called by VM's client upon Quit.
    primitive! {fn reset(&mut self) {
        self.r_stack().reset();
        self.set_source_id(0);
        if let Some(ref mut buf) = *self.input_buffer() {
            buf.clear()
        }
        self.state().source_index = 0;
        self.left_bracket();
        self.set_error(None);
    }}

    primitive! {fn _regs(&mut self) -> &mut [usize; 2] {
        self.regs()
    }}

    #[cfg(all(feature = "stc", target_arch = "x86"))]
    fn compile_reset(&mut self, _: usize) {
        //      ; Make a copy of vm in %esi because %ecx may be destroyed by
        //      ; subroutine call.
        //      ; Note that RESET is the first Forth instruction in QUIT.
        //      ; Also note that %esi is not saved here because QUIT doesn't
        //      ; return to caller.
        //      89 ce            mov    %ecx,%esi
        //      ; 判斷之前是否執行過 set_regs。
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
        self.code_space().compile_u8(0xce);
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

    /// ABORT the inner loop with an exception`e`, reset VM and clears stacks.
    fn abort_with(&mut self, e: isize) {
        self.clear_stacks();
        self.set_error(Some(e));
        let h = self.handler();
        self.execute_word(h);
    }

    /// ABORT the inner loop with an exception, reset VM and clears stacks.
    primitive! {fn abort(&mut self) {
        self.abort_with(ABORT);
    }}

    /// Data field address of `xt`. `>DFA ( xt -- dfa )`
    primitive! {fn to_dfa(&mut self) {
        let xt = self.s_stack().pop();
        let dfa = self.wordlist()[xt as usize].dfa();
        self.s_stack().push(dfa as isize);
    }}

    /// Code field address of `xt`. `>CFA ( xt -- cfa )`
    primitive! {fn to_cfa(&mut self) {
        let xt = self.s_stack().pop();
        let cfa = self.wordlist()[xt as usize].cfa();
        self.s_stack().push(cfa as isize);
    }}

    /// Name field address of `xt`. `>NFA ( xt -- nfa )`
    primitive! {fn to_nfa(&mut self) {
        let xt = self.s_stack().pop();
        let nfa = self.wordlist()[xt as usize].nfa();
        self.s_stack().push(nfa as isize);
    }}

    /// Interpretation semantics of `xt`. `>ACTION ( xt -- 'action )`
    primitive! {fn to_action(&mut self) {
        let xt = self.s_stack().pop();
        let action = self.wordlist()[xt as usize].action() as *mut u8 as usize;
        self.s_stack().push(action as isize);
    }}

    /// Execution token with interpretation semantics of `action`. `ACTION> ( action -- xt )`
    ///
    /// Return 0 if not found.
    primitive! {fn from_action(&mut self) {
        let action = self.s_stack().pop();
        let mut xt = 0;
        for w in (0..self.wordlist().len()).rev() {
            if self.wordlist()[w].action() as *mut u8 as isize == action {
                xt = w;
                break;
            }
        }
        self.s_stack().push(xt as isize);
    }}

    /// Execution token with data field address of `dfa`. `DFA> ( dfa -- xt )`
    ///
    /// Return 0 if not found.
    primitive! {fn from_dfa(&mut self) {
        let dfa = self.s_stack().pop();
        let mut xt = 0;
        for w in (0..self.wordlist().len()).rev() {
            if self.wordlist()[w].dfa() as *mut u8 as isize == dfa {
                xt = w;
                break;
            }
        }
        self.s_stack().push(xt as isize);
    }}

    /// Print name of execution token `xt`. `.name ( xt -- )`
    primitive! {fn dot_name(&mut self) {
        let xt = self.s_stack().pop() as usize;
        if 0 < xt && xt < self.wordlist().len() {
            match self.output_buffer().take() {
                Some(mut buf) => {
                    let nfa = self.wordlist()[xt].nfa();
                    {
                        let name = unsafe{ self.data_space().get_str(nfa) };
                        write!(buf, "{}", name);
                    }
                    self.set_output_buffer(buf);
                },
                None => { /* Do nothing */ }
            }
        }
    }}

    /// Pause the current task and resume the next task which is awake.
    primitive! {fn pause(&mut self) {
        let mut i = self.current_task();
        loop {
            i = (i + 1) % NUM_TASKS;
            if self.awake(i) {
                self.set_current_task(i);
                break;
            }
        }
    }}

    /// Current task ID
    primitive! {fn me(&mut self) {
        let me = self.current_task() + 1;
        self.s_stack().push(me as isize);
    }}

    /// Suspend task `i`. `suspend ( i -- )`
    primitive! {fn suspend(&mut self) {
        let i = (self.s_stack().pop() - 1) as usize;
        if i < NUM_TASKS {
            self.set_awake(i as usize, false);
        } else {
            self.abort_with(INVALID_NUMERIC_ARGUMENT);
        }
    }}

    /// Resume task `i`. `resume ( i -- )`
    primitive! {fn resume(&mut self) {
        let i = (self.s_stack().pop() - 1) as usize;
        if i < NUM_TASKS {
            self.set_awake(i as usize, true);
        } else {
            self.abort_with(INVALID_NUMERIC_ARGUMENT);
        }
    }}
}

#[cfg(test)]
mod tests {
    use super::{Core, Memory};
    use exception::{
        ABORT, CONTROL_STRUCTURE_MISMATCH, INTERPRETING_A_COMPILE_ONLY_WORD,
        INVALID_MEMORY_ADDRESS, RETURN_STACK_UNDERFLOW, STACK_UNDERFLOW, UNDEFINED_WORD,
        UNEXPECTED_END_OF_FILE, INVALID_EXECUTION_TOKEN,
    };
    use mock_vm::VM;
    use std::mem;

    #[test]
    fn test_find() {
        let vm = &mut VM::new(16, 16);
        assert!(vm.find("").is_none());
        assert!(vm.find("word-not-exist").is_none());
        vm.find("noop").expect("noop not found");
    }

    #[test]
    fn test_drop() {
        let vm = &mut VM::new(16, 16);
        vm.p_drop();
        vm.check_stacks();
        assert_eq!(vm.last_error(), Some(STACK_UNDERFLOW));
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(1);
        vm.p_drop();
        vm.check_stacks();
        assert!(vm.s_stack().is_empty());
        assert!(vm.last_error().is_none());
    }

    #[test]
    fn test_nip() {
        let vm = &mut VM::new(16, 16);
        vm.nip();
        vm.check_stacks();
        assert_eq!(vm.last_error(), Some(STACK_UNDERFLOW));
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(1);
        vm.nip();
        vm.check_stacks();
        assert_eq!(vm.last_error(), Some(STACK_UNDERFLOW));
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(1);
        vm.s_stack().push(2);
        vm.nip();
        vm.check_stacks();
        assert!(vm.last_error().is_none());
        assert!(vm.s_stack().len() == 1);
        assert!(vm.s_stack().last() == 2);
    }

    #[test]
    fn test_swap() {
        let vm = &mut VM::new(16, 16);
        vm.swap();
        vm.check_stacks();
        // check_stacks() cannot detect this kind of underflow.
        // assert_eq!(vm.last_error(), Some(STACK_UNDERFLOW));
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(1);
        vm.swap();
        vm.check_stacks();
        assert_eq!(vm.last_error(), Some(STACK_UNDERFLOW));
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

    #[test]
    fn test_dup() {
        let vm = &mut VM::new(16, 16);
        vm.dup();
        vm.check_stacks();
        // check_stacks can not detect this underflow();
        //        assert_eq!(vm.last_error(), Some(STACK_UNDERFLOW));
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

    #[test]
    fn test_over() {
        let vm = &mut VM::new(16, 16);
        vm.over();
        vm.check_stacks();
        // check_stacks() cannot detect stack underflow of over().
        // assert_eq!(vm.last_error(), Some(STACK_UNDERFLOW));
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(1);
        vm.check_stacks();
        vm.over();
        // check_stacks() cannot detect stack underflow of over().
        // assert_eq!(vm.last_error(), Some(STACK_UNDERFLOW));
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

    #[test]
    fn test_rot() {
        let vm = &mut VM::new(16, 16);
        vm.rot();
        vm.check_stacks();
        // check_stacks() cannot detect this kind of stack underflow of over().
        // assert_eq!(vm.last_error(), Some(STACK_UNDERFLOW));
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(1);
        vm.rot();
        vm.check_stacks();
        assert_eq!(vm.last_error(), Some(STACK_UNDERFLOW));
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(1);
        vm.s_stack().push(2);
        vm.rot();
        vm.check_stacks();
        assert_eq!(vm.last_error(), Some(STACK_UNDERFLOW));
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

    #[test]
    fn test_pick() {
        let vm = &mut VM::new(16, 16);
        vm.s_stack().push(0);
        vm.s_stack().push(0);
        vm.pick();
        vm.check_stacks();
        assert_eq!(vm.last_error(), None);
        assert_eq!(vm.s_stack().as_slice(), [0, 0]);

        let vm = &mut VM::new(16, 16);
        vm.s_stack().push(1);
        vm.s_stack().push(0);
        vm.s_stack().push(1);
        vm.pick();
        vm.check_stacks();
        assert_eq!(vm.last_error(), None);
        assert_eq!(vm.s_stack().as_slice(), [1, 0, 1]);

        let vm = &mut VM::new(16, 16);
        vm.s_stack().push(2);
        vm.s_stack().push(1);
        vm.s_stack().push(0);
        vm.s_stack().push(2);
        vm.pick();
        vm.check_stacks();
        assert_eq!(vm.last_error(), None);
        assert_eq!(vm.s_stack().as_slice(), [2, 1, 0, 2]);
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
        assert_eq!(vm.last_error(), Some(STACK_UNDERFLOW));
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(1);
        vm.one_plus();
        vm.check_stacks();
        assert!(vm.last_error().is_none());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), 2);
    }

    #[test]
    fn test_one_minus() {
        let vm = &mut VM::new(16, 16);
        vm.one_minus();
        vm.check_stacks();
        assert_eq!(vm.last_error(), Some(STACK_UNDERFLOW));
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(2);
        vm.one_minus();
        vm.check_stacks();
        assert!(vm.last_error().is_none());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), 1);
    }

    #[test]
    fn test_minus() {
        let vm = &mut VM::new(16, 16);
        vm.minus();
        vm.check_stacks();
        assert_eq!(vm.last_error(), Some(STACK_UNDERFLOW));
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(5);
        vm.minus();
        vm.check_stacks();
        assert_eq!(vm.last_error(), Some(STACK_UNDERFLOW));
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

    #[test]
    fn test_plus() {
        let vm = &mut VM::new(16, 16);
        vm.plus();
        vm.check_stacks();
        assert_eq!(vm.last_error(), Some(STACK_UNDERFLOW));
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(5);
        vm.plus();
        vm.check_stacks();
        assert_eq!(vm.last_error(), Some(STACK_UNDERFLOW));
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

    #[test]
    fn test_star() {
        let vm = &mut VM::new(16, 16);
        vm.star();
        vm.check_stacks();
        assert_eq!(vm.last_error(), Some(STACK_UNDERFLOW));
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(5);
        vm.star();
        vm.check_stacks();
        assert_eq!(vm.last_error(), Some(STACK_UNDERFLOW));
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

    #[test]
    fn test_slash() {
        let vm = &mut VM::new(16, 16);
        vm.slash();
        vm.check_stacks();
        assert_eq!(vm.last_error(), Some(STACK_UNDERFLOW));
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(30);
        vm.slash();
        vm.check_stacks();
        assert_eq!(vm.last_error(), Some(STACK_UNDERFLOW));
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

    #[test]
    fn test_mod() {
        let vm = &mut VM::new(16, 16);
        vm.p_mod();
        vm.check_stacks();
        assert_eq!(vm.last_error(), Some(STACK_UNDERFLOW));
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(30);
        vm.p_mod();
        vm.check_stacks();
        assert_eq!(vm.last_error(), Some(STACK_UNDERFLOW));
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

    #[test]
    fn test_slash_mod() {
        let vm = &mut VM::new(16, 16);
        vm.slash_mod();
        vm.check_stacks();
        assert_eq!(vm.last_error(), Some(STACK_UNDERFLOW));
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(30);
        vm.slash_mod();
        vm.check_stacks();
        assert_eq!(vm.last_error(), Some(STACK_UNDERFLOW));
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
        assert_eq!(vm.last_error(), Some(STACK_UNDERFLOW));
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
        assert_eq!(vm.last_error(), Some(STACK_UNDERFLOW));
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
        assert_eq!(vm.last_error(), Some(STACK_UNDERFLOW));
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
        assert_eq!(vm.last_error(), Some(STACK_UNDERFLOW));
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
        assert_eq!(vm.last_error(), Some(STACK_UNDERFLOW));
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
        assert_eq!(vm.last_error(), Some(STACK_UNDERFLOW));
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(-1);
        vm.less_than();
        vm.check_stacks();
        assert_eq!(vm.last_error(), Some(STACK_UNDERFLOW));
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
        assert_eq!(vm.last_error(), Some(STACK_UNDERFLOW));
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(0);
        vm.equals();
        vm.check_stacks();
        assert_eq!(vm.last_error(), Some(STACK_UNDERFLOW));
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
        assert_eq!(vm.last_error(), Some(STACK_UNDERFLOW));
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(1);
        vm.greater_than();
        vm.check_stacks();
        assert_eq!(vm.last_error(), Some(STACK_UNDERFLOW));
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
        assert_eq!(vm.last_error(), Some(STACK_UNDERFLOW));
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(0);
        vm.not_equals();
        vm.check_stacks();
        assert_eq!(vm.last_error(), Some(STACK_UNDERFLOW));
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
        assert_eq!(vm.last_error(), Some(STACK_UNDERFLOW));
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(1);
        vm.within();
        vm.check_stacks();
        assert_eq!(vm.last_error(), Some(STACK_UNDERFLOW));
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(1);
        vm.s_stack().push(1);
        vm.within();
        vm.check_stacks();
        assert_eq!(vm.last_error(), Some(STACK_UNDERFLOW));
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
        assert_eq!(vm.last_error(), Some(STACK_UNDERFLOW));
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
        assert_eq!(vm.last_error(), Some(STACK_UNDERFLOW));
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(707);
        vm.or();
        vm.check_stacks();
        assert_eq!(vm.last_error(), Some(STACK_UNDERFLOW));
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
        assert_eq!(vm.last_error(), Some(STACK_UNDERFLOW));
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(707);
        vm.xor();
        vm.check_stacks();
        assert_eq!(vm.last_error(), Some(STACK_UNDERFLOW));
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
        assert_eq!(vm.last_error(), Some(STACK_UNDERFLOW));
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(1);
        vm.lshift();
        vm.check_stacks();
        assert_eq!(vm.last_error(), Some(STACK_UNDERFLOW));
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
        assert_eq!(vm.last_error(), Some(STACK_UNDERFLOW));
        vm.reset();
        vm.clear_stacks();
        vm.s_stack().push(8);
        vm.rshift();
        vm.check_stacks();
        assert_eq!(vm.last_error(), Some(STACK_UNDERFLOW));
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
        assert_eq!(vm.state().source_index, 5);
        vm.parse_word();
        assert_eq!(vm.last_token().clone().unwrap(), "world");
        assert_eq!(vm.state().source_index, 11);
        vm.parse_word();
        assert_eq!(vm.last_token().clone().unwrap(), "\"");
    }

    #[test]
    fn test_evaluate_input() {
        let vm = &mut VM::new(16, 16);
        // >r
        vm.set_source(">r");
        vm.evaluate_input();
        assert_eq!(vm.last_error(), Some(INTERPRETING_A_COMPILE_ONLY_WORD));
        vm.reset();
        vm.clear_stacks();
        // drop
        vm.set_source("drop");
        vm.evaluate_input();
        assert_eq!(vm.last_error(), Some(STACK_UNDERFLOW));
        vm.reset();
        vm.clear_stacks();
        // error in colon definition: 4drop
        vm.set_source(": 4drop drop drop drop drop ; 4drop");
        vm.evaluate_input();
        assert_eq!(vm.last_error(), Some(STACK_UNDERFLOW));
        vm.reset();
        vm.clear_stacks();
        // undefined word
        vm.set_source("xdrop");
        vm.evaluate_input();
        assert_eq!(vm.last_error(), Some(UNDEFINED_WORD));
        vm.reset();
        vm.clear_stacks();
        // false true dup 1+ 2 -3
        vm.set_source("false true dup 1+ 2 -3");
        vm.evaluate_input();
        assert!(vm.last_error().is_none());
        assert_eq!(vm.s_stack().len(), 5);
        assert_eq!(vm.s_stack().pop(), -3);
        assert_eq!(vm.s_stack().pop(), 2);
        assert_eq!(vm.s_stack().pop(), 0);
        assert_eq!(vm.s_stack().pop(), -1);
        assert_eq!(vm.s_stack().pop(), 0);
    }

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
        vm.evaluate_input();
        assert_eq!(vm.last_error(), Some(UNEXPECTED_END_OF_FILE));
        vm.reset();
        vm.clear_stacks();
        // : 2+3 2 3 + ; 2+3
        vm.set_source(": 2+3 2 3 + ; 2+3");
        vm.evaluate_input();
        assert!(vm.last_error().is_none());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), 5);
    }

    #[test]
    fn test_constant() {
        let vm = &mut VM::new(16, 16);
        // constant x
        vm.set_source("constant");
        vm.evaluate_input();
        // Note: cannot detect underflow.
        // assert_eq!(vm.last_error(), Some(STACK_UNDERFLOW));
        vm.reset();
        vm.clear_stacks();
        // 5 constant x x x
        vm.set_source("5 constant x x x");
        vm.evaluate_input();
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
        vm.evaluate_input();
        vm.run();
        assert_eq!(vm.s_stack().pop(), 154);
        assert_eq!(vm.s_stack().len, 0);
    }

    #[test]
    fn test_create_and_store_fetch() {
        let vm = &mut VM::new(16, 16);
        // @
        vm.set_source("@");
        vm.evaluate_input();
        assert_eq!(vm.last_error(), Some(INVALID_MEMORY_ADDRESS));
        vm.reset();
        vm.clear_stacks();
        // !
        vm.set_source("!");
        vm.evaluate_input();
        assert_eq!(vm.last_error(), Some(INVALID_MEMORY_ADDRESS));
        vm.reset();
        vm.clear_stacks();
        // create x  1 cells allot  x !
        vm.set_source("create x  1 cells allot  x !");
        vm.evaluate_input();
        assert_eq!(vm.last_error(), Some(STACK_UNDERFLOW));
        vm.reset();
        vm.clear_stacks();
        // create x  1 cells allot  x @  3 x !  x @
        vm.set_source("create x  1 cells allot  x @  3 x !  x @");
        vm.evaluate_input();
        assert!(vm.last_error().is_none());
        assert_eq!(vm.s_stack().len(), 2);
        assert_eq!(vm.s_stack().pop(), 3);
        assert_eq!(vm.s_stack().pop(), 0);
    }

    #[test]
    fn test_create_and_fetch_in_colon() {
        let vm = &mut VM::new(16, 16);
        // create x  1 cells allot
        // 7 x !
        // : x@ x @ ; x@
        vm.set_source("create x  1 cells allot  7 x !  : x@ x @ ;  x@");
        vm.evaluate_input();
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
        vm.evaluate_input();
        vm.run();
        assert_eq!(vm.s_stack().pop(), 7);
        assert_eq!(vm.s_stack().len, 0);
    }

    #[test]
    fn test_char_plus() {
        let vm = &mut VM::new(16, 16);
        vm.char_plus();
        vm.check_stacks();
        assert_eq!(vm.last_error(), Some(STACK_UNDERFLOW));
        vm.reset();
        // 2 char+
        vm.set_source("2 char+");
        vm.evaluate_input();
        assert!(vm.last_error().is_none());
        assert_eq!(vm.s_stack().as_slice(), [3]);
    }

    #[test]
    fn test_cell_plus_and_cells() {
        let vm = &mut VM::new(16, 16);
        vm.cell_plus();
        vm.check_stacks();
        assert_eq!(vm.last_error(), Some(STACK_UNDERFLOW));
        vm.reset();
        vm.cells();
        vm.check_stacks();
        assert_eq!(vm.last_error(), Some(STACK_UNDERFLOW));
        vm.reset();
        vm.set_source("2 cell+  9 cells");
        vm.evaluate_input();
        assert!(vm.last_error().is_none());
        assert_eq!(
            vm.s_stack().as_slice(),
            [
                2 + mem::size_of::<isize>() as isize,
                9 * mem::size_of::<isize>() as isize
            ]
        );
    }

    #[test]
    fn test_tick() {
        let vm = &mut VM::new(16, 16);
        // '
        vm.tick();
        assert_eq!(vm.last_error(), Some(UNEXPECTED_END_OF_FILE));
        vm.reset();
        // ' xdrop
        vm.set_source("' xdrop");
        vm.evaluate_input();
        assert_eq!(vm.last_error(), Some(UNDEFINED_WORD));
        vm.reset();
        vm.clear_stacks();
        // ' drop
        vm.set_source("' drop");
        vm.evaluate_input();
        assert!(vm.last_error().is_none());
        assert_eq!(vm.s_stack().len(), 1);
    }

    #[test]
    fn test_execute() {
        let vm = &mut VM::new(16, 16);
        // execute
        vm.execute();
        vm.check_stacks();
        assert_eq!(vm.last_error(), Some(INVALID_EXECUTION_TOKEN));
        vm.reset();
        vm.clear_stacks();
        // ' drop execute
        vm.set_source("' drop");
        vm.evaluate_input();
        vm.execute();
        vm.check_stacks();
        assert_eq!(vm.last_error(), Some(STACK_UNDERFLOW));
        vm.reset();
        vm.clear_stacks();
        // 1 2  ' swap execute
        vm.set_source("1 2  ' swap execute");
        vm.evaluate_input();
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
        assert_eq!(vm.last_error(), Some(STACK_UNDERFLOW));
        vm.reset();
        // here 2 cells allot here -
        vm.set_source("here 2 cells allot here -");
        vm.evaluate_input();
        assert!(vm.last_error().is_none());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(
            vm.s_stack().pop(),
            -((mem::size_of::<isize>() * 2) as isize)
        );
    }

    #[test]
    fn test_to_r_r_fetch_r_from() {
        let vm = &mut VM::new(16, 16);
        vm.set_source(": t 3 >r 2 r@ + r> + ; t");
        vm.evaluate_input();
        assert!(vm.last_error().is_none());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), 8);
    }

    #[test]
    fn test_two_to_r_two_r_fetch_two_r_from() {
        let vm = &mut VM::new(16, 16);
        vm.set_source(": t 1 2 2>r 2r@ + 2r> - * ; t");
        vm.evaluate_input();
        assert!(vm.last_error().is_none());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), -3);
    }

    #[test]
    fn test_if_then() {
        let vm = &mut VM::new(16, 16);
        // : t5 if ; t5
        vm.set_source(": t5 if ;");
        vm.evaluate_input();
        assert_eq!(vm.last_error(), Some(CONTROL_STRUCTURE_MISMATCH));
        vm.reset();
        vm.clear_stacks();
        // : t4 then ; t4
        vm.set_source(": t4 then ; t4");
        vm.evaluate_input();
        assert_eq!(vm.last_error(), Some(CONTROL_STRUCTURE_MISMATCH));
        vm.reset();
        vm.clear_stacks();
        // : t1 false dup if drop true then ; t1
        vm.set_source(": t1 0 dup if drop -1 then ; t1");
        vm.evaluate_input();
        assert!(vm.last_error().is_none());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), 0);
        vm.reset();
        vm.clear_stacks();
        // : t2 true dup if drop false then ; t1
        vm.set_source(": t1 -1 dup if drop -1 then ; t1");
        vm.evaluate_input();
        assert!(vm.last_error().is_none());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), -1);
    }

    #[test]
    fn test_if_else_then() {
        let vm = &mut VM::new(16, 16);
        // : t3 else then ; t3
        vm.set_source(": t3 else then ;");
        vm.evaluate_input();
        assert_eq!(vm.last_error(), Some(CONTROL_STRUCTURE_MISMATCH));
        vm.reset();
        vm.clear_stacks();
        // : t1 0 if true else false then ; t1
        // let action = vm.code_space().here();
        vm.set_source(": t1 0 if true else false then ; t1");
        vm.evaluate_input();
        // dump(vm, action);
        assert!(vm.last_error().is_none());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), 0);
        // : t2 1 if true else false then ; t2
        vm.set_source(": t2 1 if true else false then ; t2");
        vm.evaluate_input();
        assert!(vm.last_error().is_none());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), -1);
    }

    #[test]
    fn test_begin_again() {
        let vm = &mut VM::new(16, 16);
        // : t3 begin ;
        vm.set_source(": t3 begin ;");
        vm.evaluate_input();
        assert_eq!(vm.last_error(), Some(CONTROL_STRUCTURE_MISMATCH));
        vm.reset();
        vm.clear_stacks();
        // : t2 again ;
        vm.set_source(": t2 again ;");
        vm.evaluate_input();
        assert_eq!(vm.last_error(), Some(CONTROL_STRUCTURE_MISMATCH));
        vm.reset();
        vm.clear_stacks();
        // : t1 0 begin 1+ dup 3 = if exit then again ; t1
        vm.set_source(": t1 0 begin 1+ dup 3 = if exit then again ; t1");
        vm.evaluate_input();
        assert_eq!(vm.last_error(), None);
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), 3);
    }

    #[test]
    fn test_begin_while_repeat() {
        let vm = &mut VM::new(16, 16);
        // : t1 begin ;
        vm.set_source(": t1 begin ;");
        vm.evaluate_input();
        assert_eq!(vm.last_error(), Some(CONTROL_STRUCTURE_MISMATCH));
        vm.reset();
        vm.clear_stacks();
        // : t2 while ;
        vm.set_source(": t2 while ;");
        vm.evaluate_input();
        assert_eq!(vm.last_error(), Some(CONTROL_STRUCTURE_MISMATCH));
        vm.reset();
        vm.clear_stacks();
        // : t3 repeat ;
        vm.set_source(": t3 repeat ;");
        vm.evaluate_input();
        assert_eq!(vm.last_error(), Some(CONTROL_STRUCTURE_MISMATCH));
        vm.reset();
        vm.clear_stacks();
        // : t4 begin while ;
        vm.set_source(": t4 begin while ;");
        vm.evaluate_input();
        assert_eq!(vm.last_error(), Some(CONTROL_STRUCTURE_MISMATCH));
        vm.reset();
        vm.clear_stacks();
        // : t5 begin repeat ;
        vm.set_source(": t5 begin repeat ;");
        vm.evaluate_input();
        assert_eq!(vm.last_error(), Some(CONTROL_STRUCTURE_MISMATCH));
        vm.reset();
        vm.clear_stacks();
        // : t6 while repeat ;
        vm.set_source(": t6 while repeat ;");
        vm.evaluate_input();
        assert_eq!(vm.last_error(), Some(CONTROL_STRUCTURE_MISMATCH));
        vm.reset();
        vm.clear_stacks();
        // : t7 0 begin 1+ dup 3 <> while repeat ; t1
        vm.set_source(": t7 0 begin 1+ dup 3 <> while repeat ; t7");
        vm.evaluate_input();
        assert_eq!(vm.last_error(), None);
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), 3);
    }

    #[test]
    fn test_label_goto_call() {
        let vm = &mut VM::new(16, 16);
        // Go backwards.
        vm.set_source(
            ": test1   0labels  0  [ 10 ] label 1+ dup 3 > if exit then [ 10 ] goto ; test1",
        );
        vm.evaluate_input();
        assert_eq!(vm.s_stack().pop(), 4);
        // Go forwards.
        vm.clear_stacks();
        vm.set_source(": test2   0labels  [ 10 ] goto 1 [ 10 ] label 2 3 ; test2");
        vm.evaluate_input();
        assert_eq!(vm.s_stack().len(), 2);
        // Call backwards
        vm.clear_stacks();
        vm.set_source(": test3   0labels  [ 10 ] goto [ 20 ] label 2 3  exit  [ 10 ] label  [ 20 ] call 4 5 ; test3");
        vm.evaluate_input();
        assert_eq!(vm.s_stack().len(), 4);
        // Call forwards
        vm.clear_stacks();
        vm.set_source(": test4   0labels  [ 10 ] call 1 exit [ 10 ] label 2 3 ; test4");
        vm.evaluate_input();
        assert_eq!(vm.s_stack().len(), 3);
        assert_eq!(vm.s_stack().pop(), 1);
        // Foward reference not resolved.
        vm.clear_stacks();
        vm.set_source(": test5   0labels  [ 10 ] call ; test5");
        vm.evaluate_input();
        assert_eq!(vm.last_error(), Some(CONTROL_STRUCTURE_MISMATCH));
        // 0 goto
        vm.clear_stacks();
        vm.clear_error();
        vm.set_source(": test6   [ 0 ] goto ;");
        vm.evaluate_input();
        assert!(vm.last_error() != None);
        // 0 label
        vm.clear_stacks();
        vm.clear_error();
        vm.set_source(": test7   [ 0 ] label ;");
        vm.evaluate_input();
        assert!(vm.last_error() != None);
        // 0 call
        vm.clear_stacks();
        vm.clear_error();
        vm.set_source(": test8   [ 0 ] call ;");
        vm.evaluate_input();
        assert!(vm.last_error() != None);
    }

    #[test]
    fn test_backslash() {
        let vm = &mut VM::new(16, 16);
        vm.set_source("1 2 3 \\ 5 6 7");
        vm.evaluate_input();
        assert!(vm.last_error().is_none());
        assert_eq!(vm.s_stack().len(), 3);
        assert_eq!(vm.s_stack().pop(), 3);
        assert_eq!(vm.s_stack().pop(), 2);
        assert_eq!(vm.s_stack().pop(), 1);
    }

    #[test]
    fn test_marker_unmark() {
        let vm = &mut VM::new(16, 16);
        let wordlist_len = vm.wordlist().len();
        vm.set_source("here marker empty empty here =");
        vm.evaluate_input();
        assert!(vm.last_error().is_none());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), -1);
        assert_eq!(vm.wordlist().len(), wordlist_len);
    }

    #[test]
    fn test_abort() {
        let vm = &mut VM::new(16, 16);
        vm.set_source("1 2 3 abort 5 6 7");
        vm.evaluate_input();
        assert_eq!(vm.last_error(), Some(ABORT));
        assert_eq!(vm.s_stack().len(), 0);
    }

    #[test]
    fn test_do_loop() {
        let vm = &mut VM::new(16, 16);
        // : t1 do ;
        vm.set_source(": t1 do ;");
        vm.evaluate_input();
        assert_eq!(vm.last_error(), Some(CONTROL_STRUCTURE_MISMATCH));
        vm.reset();
        vm.clear_stacks();
        // : t2 loop ;
        vm.set_source(": t2 loop ;");
        vm.evaluate_input();
        assert_eq!(vm.last_error(), Some(CONTROL_STRUCTURE_MISMATCH));
        vm.reset();
        vm.clear_stacks();
        // : main 1 5 0 do 1+ loop ;  main
        vm.set_source(": main 1 5 0 do 1+ loop ;  main");
        vm.evaluate_input();
        assert!(vm.last_error().is_none());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), 6);
    }

    #[test]
    fn test_do_unloop_exit_loop() {
        let vm = &mut VM::new(16, 16);
        // : t1 unloop ;
        vm.set_source(": t1 unloop ; t1");
        vm.evaluate_input();
        assert_eq!(vm.last_error(), Some(RETURN_STACK_UNDERFLOW));
        vm.reset();
        vm.clear_stacks();
        // : main 1 5 0 do 1+ dup 3 = if unloop exit then loop ;  main
        vm.set_source(": main 1 5 0 do 1+ dup 3 = if unloop exit then loop ;  main");
        vm.evaluate_input();
        assert!(vm.last_error().is_none());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), 3);
    }

    #[test]
    fn test_do_plus_loop() {
        let vm = &mut VM::new(16, 16);
        // : t1 +loop ;
        vm.set_source(": t1 +loop ;");
        vm.evaluate_input();
        assert_eq!(vm.last_error(), Some(CONTROL_STRUCTURE_MISMATCH));
        vm.reset();
        vm.clear_stacks();
        // : t2 5 0 do +loop ;
        vm.set_source(": t2 5 0 do +loop ; t2");
        vm.evaluate_input();
        assert_eq!(vm.last_error(), Some(STACK_UNDERFLOW));
        vm.clear_stacks();
        vm.reset();
        // : t3 1 5 0 do 1+ 2 +loop ;  main
        vm.set_source(": t3 1 5 0 do 1+ 2 +loop ;  t3");
        vm.evaluate_input();
        assert_eq!(vm.last_error(), None);
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), 4);
        // : t4 1 6 0 do 1+ 2 +loop ;  t4
        vm.set_source(": t4 1 6 0 do 1+ 2 +loop ;  t4");
        vm.evaluate_input();
        assert!(vm.last_error().is_none());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), 4);
    }

    #[test]
    fn test_do_leave_loop() {
        let vm = &mut VM::new(16, 16);
        // : t1 leave ;
        vm.set_source(": t1 leave ;  t1");
        vm.evaluate_input();
        assert_eq!(vm.last_error(), Some(CONTROL_STRUCTURE_MISMATCH));
        vm.reset();
        vm.clear_stacks();
        // : main 1 5 0 do 1+ dup 3 = if drop 88 leave then loop 9 ;  main
        vm.set_source(": main 1 5 0 do 1+ dup 3 = if drop 88 leave then loop 9 ;  main");
        vm.evaluate_input();
        assert_eq!(vm.last_error(), None);
        assert_eq!(vm.s_stack().len(), 2);
        assert_eq!(vm.s_stack().pop2(), (88, 9));
    }

    #[test]
    fn test_do_leave_plus_loop() {
        let vm = &mut VM::new(16, 16);
        // : main 1 5 0 do 1+ dup 3 = if drop 88 leave then 2 +loop 9 ;  main
        vm.set_source(": main 1 5 0 do 1+ dup 3 = if drop 88 leave then 2 +loop 9 ;  main");
        vm.evaluate_input();
        assert_eq!(vm.last_error(), None);
        assert_eq!(vm.s_stack().len(), 2);
        assert_eq!(vm.s_stack().pop2(), (88, 9));
    }

    #[test]
    fn test_do_i_loop() {
        let vm = &mut VM::new(16, 16);
        // : main 3 0 do i loop ;  main
        vm.set_source(": main 3 0 do i loop ;  main");
        vm.evaluate_input();
        assert_eq!(vm.last_error(), None);
        assert_eq!(vm.s_stack().len(), 3);
        assert_eq!(vm.s_stack().pop3(), (0, 1, 2));
    }

    #[test]
    fn test_do_i_j_loop() {
        let vm = &mut VM::new(16, 16);
        vm.set_source(": main 6 4 do 3 1 do i j * loop loop ;  main");
        vm.evaluate_input();
        assert_eq!(vm.last_error(), None);
        assert_eq!(vm.s_stack().len(), 4);
        assert_eq!(vm.s_stack().as_slice(), [4, 8, 5, 10]);
    }

    #[test]
    fn test_here_comma_compile_interpret() {
        let vm = &mut VM::new(16, 16);
        vm.comma();
        vm.check_stacks();
        assert_eq!(vm.last_error(), Some(STACK_UNDERFLOW));
        vm.reset();
        // here 1 , 2 , ] lit exit [ here
        let here = vm.data_space().here();
        vm.set_source("here 1 , 2 , ] lit exit [ here");
        vm.evaluate_input();
        assert!(vm.last_error().is_none());
        assert_eq!(vm.s_stack().len(), 2);
        let (n, t) = vm.s_stack().pop2();
        assert!(!vm.s_stack().underflow());
        assert_eq!(t - n, 4 * mem::size_of::<usize>() as isize);
        unsafe {
            assert_eq!(vm.data_space().get_isize(here + 0), 1);
            assert_eq!(vm.data_space().get_isize(here + mem::size_of::<isize>()), 2);
            assert_eq!(
                vm.data_space()
                    .get_isize(here + 2 * mem::size_of::<isize>()),
                vm.references().idx_lit as isize
            );
            assert_eq!(
                vm.data_space()
                    .get_isize(here + 3 * mem::size_of::<isize>()),
                vm.references().idx_exit as isize
            );
        }
    }

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
}
