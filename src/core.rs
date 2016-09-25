extern crate libc;

extern {
    fn memset(s: *mut libc::c_void, c: libc::uint32_t, n: libc::size_t) -> *mut libc::c_void;
}

use std::mem;
use std::ptr::{Unique, self};
use std::fmt;
use std::slice;
use std::ascii::AsciiExt;
use std::result;
use ::jitmem::JitMemory;
use exception::Exception::{
    self,
    Abort,
    UnexpectedEndOfFile,
    UndefinedWord,
    StackOverflow,
    StackUnderflow,
    ReturnStackUnderflow,
    ReturnStackOverflow,
    UnsupportedOperation,
    InterpretingACompileOnlyWord,
    InvalidMemoryAddress,
    Quit,
    Nest,
    Pause,
    Bye,
};

pub const TRUE: isize = -1;
pub const FALSE: isize = 0;

pub type Result = result::Result<(), Exception>;

// Word
pub struct Word<Target> {
    symbol: Symbol,
    is_immediate: bool,
    is_compile_only: bool,
    hidden: bool,
    dfa: usize,
    action: fn(& mut Target) -> Result
}

impl<Target> Word<Target> {
    pub fn new(symbol: Symbol, action: fn(& mut Target) -> Result, dfa: usize) -> Word<Target> {
        Word {
            symbol: symbol,
            is_immediate: false,
            is_compile_only: false,
            hidden: false,
            dfa: dfa,
            action: action
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

    pub fn action(&self) -> (fn(& mut Target) -> Result) {
        self.action
    }
}

pub struct Stack<T> {
    inner: Unique<T>,
    cap: usize,
    len: usize
}

impl<T> Stack<T> {
    pub fn with_capacity(cap: usize) -> Self {
        assert!(cap > 0 && cap <= 2048, "Invalid stack capacity");
        let align = mem::align_of::<isize>();
        let elem_size = mem::size_of::<isize>();
        let size_in_bytes = cap*elem_size;
        unsafe {
            let mut ptr = mem::uninitialized();
            libc::posix_memalign(&mut ptr, align, size_in_bytes);
            if ptr.is_null() {
                panic!("Cannot allocate memory.");
            }
            libc::mprotect(ptr, size_in_bytes, libc::PROT_READ | libc::PROT_WRITE);
            memset(ptr, 0x00, size_in_bytes);
            Stack{ inner: Unique::new(ptr as *mut _), cap: cap, len: 0 }
        }
    }

    pub fn push(&mut self, v: T) -> Option<T> {
        if self.len >= self.cap {
            Some(v)
        } else {
            unsafe {
                ptr::write(self.inner.offset(self.len as isize), v);
            }
            self.len += 1;
            None
        }
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.len < 1 {
            None
        } else {
            self.len -= 1;
            unsafe {
                Some(ptr::read(self.inner.offset(self.len as isize)))
            }
        }
    }

    pub fn push2(&mut self, v1: T, v2: T) -> Option<(T,T)> {
        if self.len + 2 > self.cap {
            Some((v1, v2))
        } else {
            unsafe {
                ptr::write(self.inner.offset(self.len as isize), v1);
                ptr::write(self.inner.offset((self.len+1) as isize), v2);
            }
            self.len += 2;
            None
        }
    }

    pub fn push3(&mut self, v1: T, v2: T, v3: T) -> Option<(T,T, T)> {
        if self.len + 3 > self.cap {
            Some((v1, v2, v3))
        } else {
            unsafe {
                ptr::write(self.inner.offset(self.len as isize), v1);
                ptr::write(self.inner.offset((self.len+1) as isize), v2);
                ptr::write(self.inner.offset((self.len+2) as isize), v3);
            }
            self.len += 3;
            None
        }
    }

    pub fn pop2(&mut self) -> Option<(T,T)> {
        if self.len < 2 {
            None
        } else {
            self.len -= 2;
            unsafe {
                Some((
                    ptr::read(self.inner.offset(self.len as isize)),
                    ptr::read(self.inner.offset((self.len+1) as isize))
                ))
            }
        }
    }

    pub fn pop3(&mut self) -> Option<(T,T,T)> {
        if self.len < 3 {
            None
        } else {
            self.len -= 3;
            unsafe {
                Some((
                    ptr::read(self.inner.offset(self.len as isize)),
                    ptr::read(self.inner.offset((self.len+1) as isize)),
                    ptr::read(self.inner.offset((self.len+2) as isize)),
                ))
            }
        }
    }

    pub fn last(&self) -> Option<T> {
        if self.len == 0 {
            None
        } else {
            unsafe {
                Some(ptr::read(self.inner.offset((self.len-1) as isize)))
            }
        }
    }

    pub fn get(&self, pos: usize) -> Option<T> {
        if pos >= self.len {
            None
        } else {
            unsafe {
                Some(ptr::read(self.inner.offset(pos as isize)))
            }
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
                for i in 0..(self.len()-1) {
                    let v = unsafe { ptr::read(self.inner.offset(i as isize)) };
                    match write!(f, "{} ", v) {
                        Ok(_) => {},
                        Err(e) => { return Err(e); }
                    }
                }
                Ok(())
            },
            Err(e) => Err(e)
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
    pub auto_flush: bool,
    // Last definition, 0 if last define fails.
    pub last_definition: usize,
}

impl State {
    pub fn new() -> State {
      State {
        is_compiling: false,
        instruction_pointer: 0,
        word_pointer: 0,
        source_index: 0,
        auto_flush: true,
        last_definition: 0,
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
    id: usize
}

impl Symbol {
    pub fn id(&self) -> usize { self.id }
}

pub trait Core : Sized {

  // Functions to access VM.

  fn jit_memory(&mut self) -> &mut JitMemory;
  fn jit_memory_const(&self) -> &JitMemory;
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
  fn f_stack(&mut self) -> &mut Stack<f64>;
  fn symbols_mut(&mut self) -> &mut Vec<String>;
  fn symbols(&self) -> &Vec<String>;
  fn wordlist_mut(&mut self) -> &mut Vec<Word<Self>>;
  fn wordlist(&self) -> &Vec<Word<Self>>;
  fn state(&mut self) -> &mut State;
  fn references(&mut self) -> &mut ForwardReferences;
  fn evaluators(&mut self) -> &mut Option<Vec<fn(&mut Self, token: &str) -> Result>>;
  fn set_evaluators(&mut self, Vec<fn(&mut Self, token: &str) -> Result>);

  /// Add core primitives to self.
  fn add_core(&mut self) {
    // Bytecodes
    self.add_primitive("noop", Core::noop); // j1, Ngaro, jx
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

    // Compile-only bytecodes
    self.add_compile_only("exit", Core::exit); // j1, jx, eForth
    self.add_compile_only("halt", Core::halt); // rtForth
    self.add_compile_only("lit", Core::lit); // Ngaro, jx, eForth
    self.add_compile_only("branch", Core::branch); // j1, eForth
    self.add_compile_only("0branch", Core::zero_branch); // j1, eForth
    self.add_compile_only(">r", Core::p_to_r); // j1, Ngaro, jx, eForth
    self.add_compile_only("r>", Core::r_from); // j1, Ngaro, jx, eForth
    self.add_compile_only("r@", Core::r_fetch); // j1, jx, eForth
    self.add_compile_only("2>r", Core::two_to_r); // jx
    self.add_compile_only("2r>", Core::two_r_from); // jx
    self.add_compile_only("2r@", Core::two_r_fetch); // jx
    self.add_compile_only("_do", Core::_do); // jx
    self.add_compile_only("_loop", Core::p_loop); // jx
    self.add_compile_only("_+loop", Core::p_plus_loop); // jx
    self.add_compile_only("unloop", Core::unloop); // jx
    self.add_compile_only("leave", Core::leave); // jx
    self.add_compile_only("i", Core::p_i); // jx
    self.add_compile_only("j", Core::p_j); // jx

    // Candidates for bytecodes
    // Ngaro: LOOP, JUMP, RETURN, IN, OUT, WAIT
    // j1: U<, RET, IO@, IO!
    // eForth: UM+, !IO, ?RX, TX!
    // jx: PICK, U<, UM*, UM/MOD, D+, TX, RX, CATCH, THROW, QUOTE, UP!, UP+, PAUSE,

    // Immediate words
    self.add_immediate("(", Core::imm_paren);
    self.add_immediate("\\", Core::imm_backslash);
    self.add_immediate("[", Core::interpret);
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

    // Compile-only words

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
    self.add_primitive("pause", Core::pause);
    self.add_primitive("/", Core::slash);
    self.add_primitive("mod", Core::p_mod);
    self.add_primitive("abs", Core::abs);
    self.add_primitive("negate", Core::negate);
    self.add_primitive("between", Core::between);
    self.add_primitive("parse-word", Core::parse_word);;
    self.add_primitive("char", Core::char);
    self.add_primitive("parse", Core::parse);
    self.add_primitive("evaluate", Core::evaluate);;
    self.add_primitive(":", Core::colon);
    self.add_primitive("constant", Core::constant);
    self.add_primitive("variable", Core::variable);
    self.add_primitive("create", Core::create);
    self.add_primitive("'", Core::tick);
    self.add_primitive("]", Core::compile);
    self.add_primitive(",", Core::comma);
    self.add_primitive("marker", Core::marker);
    self.add_primitive("quit", Core::quit);
    self.add_primitive("abort", Core::abort);
    self.add_primitive("bye", Core::bye);

    self.references().idx_lit = self.find("lit").expect("lit undefined");
    self.references().idx_exit = self.find("exit").expect("exit undefined");
    self.references().idx_zero_branch = self.find("0branch").expect("0branch undefined");
    self.references().idx_branch = self.find("branch").expect("branch undefined");
    self.references().idx_do = self.find("_do").expect("_do undefined");
    self.references().idx_loop = self.find("_loop").expect("_loop undefined");
    self.references().idx_plus_loop = self.find("_+loop").expect("_+loop undefined");
    let idx_halt = self.find("halt").expect("halt undefined");
    self.jit_memory().put_u32(idx_halt as u32, 0);
    self.extend_evaluator(Core::evaluate_integer);
  }

  /// Add a primitive word to word list.
  fn add_primitive(&mut self, name: &str, action: fn(& mut Self) -> Result) {
      let symbol = self.new_symbol(name);
      let word = Word::new(symbol, action, self.jit_memory().len());
      self.state().last_definition = self.wordlist().len();
      self.wordlist_mut().push(word);
  }

  /// Add an immediate word to word list.
  fn add_immediate(&mut self, name: &str, action: fn(& mut Self) -> Result) {
      self.add_primitive (name, action);
      let def = self.state().last_definition;
      self.wordlist_mut()[def].set_immediate(true);
  }

  /// Add a compile-only word to word list.
  fn add_compile_only(&mut self, name: &str, action: fn(& mut Self) -> Result) {
      self.add_primitive (name, action);
      let def = self.state().last_definition;
      self.wordlist_mut()[def].set_compile_only(true);
  }

  /// Add an immediate and compile-only word to word list.
  fn add_immediate_and_compile_only(&mut self, name: &str, action: fn(& mut Self) -> Result) {
      self.add_primitive (name, action);
      let def = self.state().last_definition;
      let w = &mut self.wordlist_mut()[def];
      w.set_immediate(true);
      w.set_compile_only(true);
  }

  /// Execute word at position `i`.
  fn execute_word(&mut self, i: usize) -> Result {
      self.state().word_pointer = i;
      (self.wordlist()[i].action())(self)
  }

  /// Find the word with name `name`.
  /// If not found returns zero.
  fn find(&mut self, name: &str) -> Option<usize> {
      match self.find_symbol(name) {
          Some(symbol) => {
              for (i, word) in self.wordlist().iter().enumerate() {
                  if !word.is_hidden() && word.symbol() == symbol {
                      return Some(i);
                  }
              }
              None
          },
          None => None
      }
  }

  fn find_symbol(&mut self, s: &str) -> Option<Symbol> {
      for (i, sym) in self.symbols().iter().enumerate().rev() {
          if sym.eq_ignore_ascii_case(s) {
              return Some(Symbol {id: i });
          }
      }
      None
  }

  fn new_symbol(&mut self, s: &str) -> Symbol {
      self.symbols_mut().push(s.to_string());
      Symbol{ id: self.symbols().len() - 1 }
  }

  //------------------
  // Inner interpreter
  //------------------

  /// Evaluate a compiled program following self.state().instruction_pointer.
  /// Any exception other than Nest causes termination of inner loop.
  /// Quit is aspecially used for this purpose.
  /// Never return None and Some(Nest).
  #[no_mangle]
  #[inline(never)]
  fn run(&mut self) -> Result {
      let mut ip = self.state().instruction_pointer;
      while 0 < ip && ip < self.jit_memory().len() {
          let w = self.jit_memory().get_i32(ip) as usize;
          self.state().instruction_pointer += mem::size_of::<i32>();
          if let Err(e) = self.execute_word (w) {
              match e {
                  Nest => {},
                  _ => return Err(e)
              }
          }
          ip = self.state().instruction_pointer;
      }
      if ip == 0 {
          Ok(())
      } else {
          Err(InvalidMemoryAddress)
      }
  }

  //---------
  // Compiler
  //---------

  fn compile_word(&mut self, word_index: usize) {
      self.jit_memory().compile_i32(word_index as i32);
  }

  /// Compile integer `i`.
  fn compile_integer (&mut self, i: isize) {
      let idx = self.references().idx_lit as i32;
      self.jit_memory().compile_i32(idx);
      self.jit_memory().compile_i32(i as i32);
  }

  //-----------
  // Evaluation
  //-----------

  fn interpret(& mut self) -> Result {
      self.state().is_compiling = false;
      Ok(())
  }

  fn compile(& mut self) -> Result {
      self.state().is_compiling = true;
      Ok(())
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
  fn parse_word(&mut self) -> Result {
      let mut last_token = self.last_token().take().unwrap();
      last_token.clear();
      let input_buffer = self.input_buffer().take().unwrap();
      {
          let source = &input_buffer[self.state().source_index..input_buffer.len()];
          let mut cnt = 0;
          for ch in source.chars() {
              cnt = cnt + 1;
              match ch {
                  '\t' | '\n' | '\r' | ' ' => {
                      if !last_token.is_empty() {
                          break;
                      }
                  },
                  _ => last_token.push(ch)
              };
          }
          self.state().source_index = self.state().source_index + cnt;
      }
      self.set_last_token(last_token);
      self.set_input_buffer(input_buffer);
      Ok(())
  }

  /// Run-time: ( "&lt;spaces&gt;name" -- char)
  ///
  /// Skip leading space delimiters. Parse name delimited by a space. Put the value of its first character onto the stack.
  fn char(&mut self) -> Result {
      let result;
      try!(self.parse_word());
      let last_token = self.last_token().take().unwrap();
      match last_token.chars().nth(0) {
          Some(c) =>
              match self.s_stack().push(c as isize) {
                  Some(_) => result = Err(StackOverflow),
                  None => result = Ok(())
              },
          None => result = Err(UnexpectedEndOfFile)
      }
      self.set_last_token(last_token);
      result
  }

  /// Compilation: ( "&lt;spaces&gt;name" -- )
  ///
  /// Skip leading space delimiters. Parse name delimited by a space. Append the run-time semantics given below to the current definition.
  ///
  /// Run-time: ( -- char )
  ///
  /// Place `char`, the value of the first character of name, on the stack.
  fn bracket_char(&mut self) -> Result {
      try!(self.char());
      match self.s_stack().pop() {
          Some(ch) => {
              self.compile_integer(ch);
              Ok(())
          },
          None => Err(StackUnderflow)
      }
  }

  /// Run-time: ( char "ccc&lt;char&gt;" -- )
  ///
  /// Parse ccc delimited by the delimiter char.
  fn parse(&mut self) -> Result {
      let input_buffer = self.input_buffer().take().unwrap();
      match self.s_stack().pop() {
          Some(v) => {
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
              Ok(())
          },
          None => {
            self.set_input_buffer(input_buffer);
            Err(StackUnderflow)
          }
      }
  }

  fn imm_paren(&mut self) -> Result {
      match self.s_stack().push(')' as isize) {
          Some(_) => Err(StackOverflow),
          None => self.parse()
      }
  }

  fn imm_backslash(&mut self) -> Result {
      self.state().source_index = match *self.input_buffer() {
        Some(ref buf) => buf.len(),
        None => 0
      };
      Ok(())
  }

  /// Exception Quit is captured by evaluate. Quit does not be used to leave evaluate.
  /// Never returns Some(Quit).
  fn evaluate(&mut self) -> Result {
      let result;
      let mut last_token;
      loop {
          try!(self.parse_word());
          last_token = self.last_token().take().unwrap();
          if last_token.is_empty() {
              result = Ok(());
              break;
          }
          match self.find(&last_token) {
              Some(found_index) => {
                  let is_immediate_word;
                  let is_compile_only_word;
                  {
                      let word = &self.wordlist()[found_index];
                      is_immediate_word = word.is_immediate();
                      is_compile_only_word = word.is_compile_only();
                  }
                  if self.state().is_compiling && !is_immediate_word {
                      self.compile_word(found_index);
                  } else if !self.state().is_compiling && is_compile_only_word {
                      result = Err(InterpretingACompileOnlyWord);
                      break;
                  } else {
                      self.set_last_token(last_token);
                      match self.execute_word(found_index) {
                          Err(e) => {
                              last_token = self.last_token().take().unwrap();
                              match e {
                                  Nest => {
                                      if let Err(e2) = self.run() {
                                          match e2 {
                                              Quit => {},
                                              _ => {
                                                  result = Err(e2);
                                                  break;
                                              }
                                          }
                                      }
                                  },
                                  Quit => {},
                                  _ => {
                                    result = Err(e);
                                    break;
                                  }
                              }
                          },
                          Ok(()) => {
                            last_token = self.last_token().take().unwrap();
                          }
                      };
                  }
              },
              None => {
                  let mut done = false;
                  let evaluators = self.evaluators().take().unwrap();
                  for h in &evaluators {
                      match h(self, &last_token) {
                          Ok(_) => {
                              done = true;
                              break;
                          },
                          Err(_) => { continue }
                      }
                  }
                  self.set_evaluators(evaluators);
                  if !done {
                      print!("{} ", &last_token);
                      result = Err(UndefinedWord);
                      break;
                  }
              }
          }
          self.set_last_token(last_token);
      }
      self.set_last_token(last_token);
      result
  }

  fn base(&mut self) -> Result {
      let base_addr = self.jit_memory().system_variables().base_addr();
      match self.s_stack().push(base_addr as isize) {
          Some(_) => Err(StackOverflow),
          None => Ok(())
      }
  }

  fn evaluate_integer(&mut self, token: &str) -> Result {
      let base_addr = self.jit_memory().system_variables().base_addr();
      let base = self.jit_memory().get_isize(base_addr);
      match isize::from_str_radix(token, base as u32) {
          Ok(t) => {
              if self.state().is_compiling {
                  self.compile_integer(t);
              } else {
                  self.s_stack().push (t);
              }
              Ok(())
          },
          Err(_) => Err(UnsupportedOperation)
      }
  }

  /// Extend `f` to evaluators.
  /// Will create a vector for evaluators if there was no evaluator.
  fn extend_evaluator(&mut self, f: fn(&mut Self, token: &str) -> Result) {
      let optional_evaluators = self.evaluators().take();
      match optional_evaluators {
          Some(mut evaluators) => {
              evaluators.push(f);
              self.set_evaluators(evaluators);
          },
          None => {
              self.set_evaluators(vec![f]);
          }
      }
  }

  //-----------------------
  // High level definitions
  //-----------------------

  fn nest(&mut self) -> Result {
      if self.r_stack().is_full() {
          Err(ReturnStackOverflow)
      } else {
          unsafe {
              ptr::write(self.r_stack().inner.offset(self.r_stack().len as isize), self.state().instruction_pointer as isize);
          }
          self.r_stack().len += 1;
          let wp = self.state().word_pointer;
          self.state().instruction_pointer = self.wordlist()[wp].dfa();
          Err(Nest)
      }
  }

  fn p_var(&mut self) -> Result {
      let wp = self.state().word_pointer;
      let dfa = self.wordlist()[wp].dfa() as isize;
      match self.s_stack().push(dfa) {
          Some(_) => Err(StackOverflow),
          None => Ok(())
      }
  }

  fn p_const(&mut self) -> Result {
      let wp = self.state().word_pointer;
      let dfa = self.wordlist()[wp].dfa();
      let value = self.jit_memory().get_i32(dfa) as isize;
      match self.s_stack().push(value) {
          Some(_) => Err(StackOverflow),
          None => Ok(())
      }
  }

  fn p_fvar(&mut self) -> Result {
      let wp = self.state().word_pointer;
      let dfa = self.wordlist()[wp].dfa() as isize;
      match self.s_stack().push(dfa) {
          Some(_) => Err(StackOverflow),
          None => Ok(())
      }
  }

  fn define(&mut self, action: fn(& mut Self) -> Result) -> Result {
      try!(self.parse_word());
      let last_token = self.last_token().take().unwrap();
      if let Some(_) = self.find(&last_token) {
          print!("Redefining {}", last_token);
      }
      if last_token.is_empty() {
          self.state().last_definition = 0;
          self.set_last_token(last_token);
          Err(UnexpectedEndOfFile)
      } else {
          let symbol = self.new_symbol(&last_token);
          let word = Word::new(symbol, action, self.jit_memory().len());
          self.state().last_definition = self.wordlist().len();
          self.wordlist_mut().push(word);
          self.set_last_token(last_token);
          Ok(())
      }
  }

  fn colon(&mut self) -> Result {
      match self.define(Core::nest) {
          Err(e) => Err(e),
          Ok(()) => {
              let def = self.state().last_definition;
              self.wordlist_mut()[def].set_hidden(true);
              self.compile()
          }
      }
  }

  fn semicolon(&mut self) -> Result{
      if self.state().last_definition != 0 {
          let idx = self.references().idx_exit as i32;
          self.jit_memory().compile_i32(idx);
          let def = self.state().last_definition;
          self.wordlist_mut()[def].set_hidden(false);
      }
      self.interpret()
  }

  fn create(&mut self) -> Result {
      self.define(Core::p_var)
  }

  fn variable(&mut self) -> Result {
      match self.define(Core::p_var) {
          Err(e) => Err(e),
          Ok(()) => {
              self.jit_memory().compile_i32(0);
              Ok(())
          }
      }
  }

  fn constant(&mut self) -> Result {
      match self.s_stack().pop() {
          Some(v) => {
              match self.define(Core::p_const) {
                  Err(e) => Err(e),
                  Ok(()) => {
                      self.jit_memory().compile_i32(v as i32);
                      Ok(())
                  }
              }
          },
          None => Err(StackUnderflow)
      }
  }

  fn unmark(&mut self) -> Result {
      let wp = self.state().word_pointer;
      let dfa;
      let symbol;
      {
          let w = &self.wordlist()[wp];
          dfa = w.dfa();
          symbol = w.symbol();
      }
      let jlen = self.jit_memory().get_i32(dfa) as usize;
      self.jit_memory().truncate(jlen);
      self.wordlist_mut().truncate(wp+1);
      self.symbols_mut().truncate(symbol.id+1);
      Ok(())
  }

  fn marker(&mut self) -> Result {
      try!(self.define(Core::unmark));
      let jlen = self.jit_memory().len() as i32;
      self.jit_memory().compile_i32(jlen+(mem::size_of::<i32>() as i32));
      Ok(())
  }

  //--------
  // Control
  //--------

  fn branch(&mut self) -> Result {
      let ip = self.state().instruction_pointer;
      self.state().instruction_pointer = self.jit_memory().get_i32(ip) as usize;
      Ok(())
  }

  fn zero_branch(&mut self) -> Result {
      match self.s_stack().pop() {
          Some(v) => {
              if v == 0 {
                  self.branch()
              } else {
                  self.state().instruction_pointer += mem::size_of::<i32>();
                  Ok(())
              }
          },
          None => Err(StackUnderflow)
      }
  }

  /// ( n1|u1 n2|u2 -- ) ( R: -- loop-sys )
  ///
  /// Set up loop control parameters with index `n2`|`u2` and limit `n1`|`u1`. An
  /// ambiguous condition exists if `n1`|`u1` and `n2`|`u2` are not both the same
  /// type.  Anything already on the return stack becomes unavailable until
  /// the loop-control parameters are discarded.
  fn _do(&mut self) -> Result {
      let ip = self.state().instruction_pointer as isize;
      match self.r_stack().push(ip) {
          Some(_) => Err(ReturnStackOverflow),
          None => {
              self.state().instruction_pointer += mem::size_of::<i32>();
              self.two_to_r()
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
  fn p_loop(&mut self) -> Result {
      match self.r_stack().pop2() {
          Some((rn, rt)) => {
              if rt+1 < rn {
                  self.r_stack().push2(rn, rt+1);
                  self.branch()
              } else {
                  match self.r_stack().pop() {
                      Some(_) => {
                          self.state().instruction_pointer += mem::size_of::<i32>();
                          Ok(())
                      },
                      None => Err(ReturnStackUnderflow)
                  }
              }
          },
          None => Err(ReturnStackUnderflow)
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
  fn p_plus_loop(&mut self) -> Result {
      match self.r_stack().pop2() {
          Some((rn, rt)) => {
              match self.s_stack().pop() {
                  Some(t) => {
                      if rt+t < rn {
                          self.r_stack().push2(rn, rt+t);
                          self.branch()
                      } else {
                          match self.r_stack().pop() {
                              Some(_) => {
                                  self.state().instruction_pointer += mem::size_of::<i32>();
                                  Ok(())
                              },
                              None => Err(ReturnStackUnderflow)
                          }
                      }
                  },
                  None => Err(StackUnderflow)
              }
          },
          None => Err(ReturnStackUnderflow)
      }
  }

  /// Run-time: ( -- ) ( R: loop-sys -- )
  ///
  /// Discard the loop-control parameters for the current nesting level. An
  /// `UNLOOP` is required for each nesting level before the definition may be
  /// `EXIT`ed. An ambiguous condition exists if the loop-control parameters
  /// are unavailable.
  fn unloop(&mut self) -> Result {
      match self.r_stack().pop3() {
          Some(_) => Ok(()),
          None => Err(ReturnStackUnderflow)
      }
  }

  fn leave(&mut self) -> Result {
      match self.r_stack().pop3() {
          Some((third, _, _)) => {
              self.state().instruction_pointer = self.jit_memory().get_i32(third as usize) as usize;
              Ok(())
          },
          None => Err(ReturnStackUnderflow)
      }
  }

  fn p_i(&mut self) -> Result {
      match self.r_stack().last() {
          Some(i) => {
              match self.s_stack().push(i) {
                  Some(_) => Err(StackOverflow),
                  None => Ok(())
              }
          },
          None => Err(ReturnStackUnderflow)
      }
  }

  fn p_j(&mut self) -> Result {
      let pos = self.r_stack().len() - 4;
      match self.r_stack().get(pos) {
          Some(j) => {
              match self.s_stack().push(j) {
                  Some(_) => Err(StackOverflow),
                  None => Ok(())
              }
          },
          None => Err(ReturnStackUnderflow)
      }
  }

  fn imm_if(&mut self) -> Result {
      let idx = self.references().idx_zero_branch as i32;
      self.jit_memory().compile_i32(idx);
      self.jit_memory().compile_i32(0);
      self.here()
  }

  fn imm_else(&mut self) -> Result {
      match self.s_stack().pop() {
          Some(if_part) => {
              let idx = self.references().idx_branch as i32;
              self.jit_memory().compile_i32(idx);
              self.jit_memory().compile_i32(0);
              try!(self.here());
              let here = self.jit_memory().len();
              self.jit_memory().put_i32(here as i32, (if_part - mem::size_of::<i32>() as isize) as usize);
              Ok(())
          },
          None => Err(StackUnderflow)
      }
  }

  fn imm_then(&mut self) -> Result {
      match self.s_stack().pop() {
          Some(branch_part) => {
              let here = self.jit_memory().len();
              self.jit_memory().put_i32(here as i32, (branch_part - mem::size_of::<i32>() as isize) as usize);
              Ok(())
          },
          None => Err(StackUnderflow)
      }
  }

  fn imm_begin(&mut self) -> Result {
      self.here()
  }

  fn imm_while(&mut self) -> Result {
      let idx = self.references().idx_zero_branch as i32;
      self.jit_memory().compile_i32(idx);
      self.jit_memory().compile_i32(0);
      self.here()
  }

  fn imm_repeat(&mut self) -> Result {
      match self.s_stack().pop2() {
          Some((begin_part, while_part)) => {
              let idx = self.references().idx_branch as i32;
              self.jit_memory().compile_i32(idx);
              self.jit_memory().compile_i32(begin_part as i32);
              let here = self.jit_memory().len();
              self.jit_memory().put_i32(here as i32, (while_part - mem::size_of::<i32>() as isize) as usize);
              Ok(())
          },
          None => Err(StackUnderflow)
      }
  }

  fn imm_again(&mut self) -> Result {
      match self.s_stack().pop() {
          Some(begin_part) => {
              let idx = self.references().idx_branch as i32;
              self.jit_memory().compile_i32(idx);
              self.jit_memory().compile_i32(begin_part as i32);
              Ok(())
          },
          None => Err(StackUnderflow)
      }
  }

  fn imm_recurse(&mut self) -> Result {
      let last = self.wordlist().len() - 1;
      self.jit_memory().compile_u32(last as u32);
      Ok(())
  }

  /// Execution: ( -- a-ddr )
  ///
  /// Append the run-time semantics of `_do` to the current definition. The semantics are incomplete until resolved by `LOOP` or `+LOOP`.
  fn imm_do(&mut self) -> Result {
      let idx = self.references().idx_do as i32;
      self.jit_memory().compile_i32(idx);
      self.jit_memory().compile_i32(0);
      self.here()
  }

  /// Run-time: ( a-addr -- )
  ///
  /// Append the run-time semantics of `_LOOP` to the current definition.
  /// Resolve the destination of all unresolved occurrences of `LEAVE` between
  /// the location given by do-sys and the next location for a transfer of
  /// control, to execute the words following the `LOOP`.
  fn imm_loop(&mut self) -> Result{
      match self.s_stack().pop() {
          Some(do_part) => {
              let idx = self.references().idx_loop as i32;
              self.jit_memory().compile_i32(idx);
              self.jit_memory().compile_i32(do_part as i32);
              let here = self.jit_memory().len();
              self.jit_memory().put_i32(here as i32, (do_part - mem::size_of::<i32>() as isize) as usize);
              Ok(())
          },
          None => Err(StackUnderflow)
      }
  }

  /// Run-time: ( a-addr -- )
  ///
  /// Append the run-time semantics of `_+LOOP` to the current definition.
  /// Resolve the destination of all unresolved occurrences of `LEAVE` between
  /// the location given by do-sys and the next location for a transfer of
  /// control, to execute the words following `+LOOP`.
  fn imm_plus_loop(&mut self) -> Result {
      match self.s_stack().pop() {
          Some(do_part) => {
              let idx = self.references().idx_plus_loop as i32;
              self.jit_memory().compile_i32(idx);
              self.jit_memory().compile_i32(do_part as i32);
              let here = self.jit_memory().len();
              self.jit_memory().put_i32(here as i32, (do_part - mem::size_of::<i32>() as isize) as usize);
              Ok(())
          },
          None => Err(StackUnderflow)
      }
  }

  //-----------
  // Primitives
  //-----------

  /// Run-time: ( -- )
  ///
  /// No operation
  fn noop(&mut self) -> Result {
      // Do nothing
      Ok(())
  }

  /// Run-time: ( -- true )
  ///
  /// Return a true flag, a single-cell value with all bits set.
  fn p_true(&mut self) -> Result {
      match self.s_stack().push (TRUE) {
          Some(_) => Err(StackOverflow),
          None => Ok(())
      }
  }

  /// Run-time: ( -- false )
  ///
  /// Return a false flag.
  fn p_false(&mut self) -> Result {
      match self.s_stack().push (FALSE) {
          Some(_) => Err(StackOverflow),
          None => Ok(())
      }
  }

  /// Run-time: (c-addr1 -- c-addr2 )
  ///
  ///Add the size in address units of a character to `c-addr1`, giving `c-addr2`.
  fn char_plus(&mut self) -> Result {
      match self.s_stack().pop() {
          Some(v) =>
              match self.s_stack().push(v + mem::size_of::<u8>() as isize) {
                  Some(_) => Err(StackOverflow),
                  None => Ok(())
              },
          None => Err(StackUnderflow)
      }
  }

  /// Run-time: (n1 -- n2 )
  ///
  /// `n2` is the size in address units of `n1` characters.
  fn chars(&mut self) -> Result {
      match self.s_stack().pop() {
          Some(v) =>
              match self.s_stack().push(v*mem::size_of::<u8>() as isize) {
                  Some(_) => Err(StackOverflow),
                  None => Ok(())
              },
          None => Err(StackUnderflow)
      }
  }


  /// Run-time: (a-addr1 -- a-addr2 )
  ///
  /// Add the size in address units of a cell to `a-addr1`, giving `a-addr2`.
  fn cell_plus(&mut self) -> Result {
      match self.s_stack().pop() {
          Some(v) =>
              match self.s_stack().push(v + mem::size_of::<i32>() as isize) {
                  Some(_) => Err(StackOverflow),
                  None => Ok(())
              },
          None => Err(StackUnderflow)
      }
  }

  /// Run-time: (n1 -- n2 )
  ///
  /// `n2` is the size in address units of `n1` cells.
  fn cells(&mut self) -> Result {
      match self.s_stack().pop() {
          Some(v) =>
              match self.s_stack().push(v*mem::size_of::<i32>() as isize) {
                  Some(_) => Err(StackOverflow),
                  None => Ok(())
              },
          None => Err(StackUnderflow)
      }
  }

  fn lit(&mut self) -> Result {
      if self.s_stack().is_full() {
          Err(StackOverflow)
      } else {
          unsafe {
              let ip = self.state().instruction_pointer;
              let v = self.jit_memory().get_i32(ip) as isize;
              ptr::write(self.s_stack().inner.offset((self.s_stack().len) as isize), v);
          }
          self.s_stack().len += 1;
          self.state().instruction_pointer = self.state().instruction_pointer + mem::size_of::<i32>();
          Ok(())
      }
  }

  fn swap(&mut self) -> Result {
      if self.s_stack().len < 2 {
          Err(StackUnderflow)
      } else {
          unsafe {
              let t = ptr::read(self.s_stack().inner.offset((self.s_stack().len-1) as isize));
              ptr::write(self.s_stack().inner.offset((self.s_stack().len-1) as isize), ptr::read(self.s_stack().inner.offset((self.s_stack().len-2) as isize)));
              ptr::write(self.s_stack().inner.offset((self.s_stack().len-2) as isize), t);
          }
          Ok(())
      }
  }

  fn dup(&mut self) -> Result {
      if self.s_stack().len < 1 {
          Err(StackUnderflow)
      } else if self.s_stack().is_full() {
          Err(StackOverflow)
      } else {
          unsafe {
              ptr::write(self.s_stack().inner.offset((self.s_stack().len) as isize), ptr::read(self.s_stack().inner.offset((self.s_stack().len-1) as isize)));
              self.s_stack().len += 1;
          }
          Ok(())
      }
  }

  fn p_drop(&mut self) -> Result {
      if self.s_stack().len < 1 {
          Err(StackUnderflow)
      } else {
          self.s_stack().len -= 1;
          Ok(())
      }
  }

  fn nip(&mut self) -> Result {
      if self.s_stack().len < 2 {
          Err(StackUnderflow)
      } else {
          unsafe {
              self.s_stack().len -= 1;
              ptr::write(self.s_stack().inner.offset((self.s_stack().len-1) as isize), ptr::read(self.s_stack().inner.offset((self.s_stack().len) as isize)));
          }
          Ok(())
      }
  }

  fn over(&mut self) -> Result {
      if self.s_stack().len < 2 {
          Err(StackUnderflow)
      } else if self.s_stack().is_full() {
          Err(StackOverflow)
      } else {
          unsafe {
              ptr::write(self.s_stack().inner.offset((self.s_stack().len) as isize), ptr::read(self.s_stack().inner.offset((self.s_stack().len-2) as isize)));
              self.s_stack().len += 1;
          }
          Ok(())
      }
  }

  fn rot(&mut self) -> Result {
      if self.s_stack().len < 3 {
          Err(StackUnderflow)
      } else {
          unsafe {
              let t = ptr::read(self.s_stack().inner.offset((self.s_stack().len-1) as isize));
              let n = ptr::read(self.s_stack().inner.offset((self.s_stack().len-2) as isize));
              ptr::write(self.s_stack().inner.offset((self.s_stack().len-1) as isize), ptr::read(self.s_stack().inner.offset((self.s_stack().len-3) as isize)));
              ptr::write(self.s_stack().inner.offset((self.s_stack().len-2) as isize), t);
              ptr::write(self.s_stack().inner.offset((self.s_stack().len-3) as isize), n);
          }
          Ok(())
      }
  }

  fn two_drop(&mut self) -> Result {
      if self.s_stack().len < 2 {
          Err(StackUnderflow)
      } else {
          self.s_stack().len -= 2;
          Ok(())
      }
  }

  fn two_dup(&mut self) -> Result {
      if self.s_stack().len < 2 {
          Err(StackUnderflow)
      } else if self.s_stack().len + 2 > self.s_stack().cap {
          Err(StackOverflow)
      } else {
          unsafe {
              self.s_stack().len += 2;
              ptr::write(self.s_stack().inner.offset((self.s_stack().len-1) as isize), ptr::read(self.s_stack().inner.offset((self.s_stack().len-3) as isize)));
              ptr::write(self.s_stack().inner.offset((self.s_stack().len-2) as isize), ptr::read(self.s_stack().inner.offset((self.s_stack().len-4) as isize)));
          }
          Ok(())
      }
  }

  fn two_swap(&mut self) -> Result {
      if self.s_stack().len < 4 {
          Err(StackUnderflow)
      } else {
          unsafe {
              let t = ptr::read(self.s_stack().inner.offset((self.s_stack().len-1) as isize));
              let n = ptr::read(self.s_stack().inner.offset((self.s_stack().len-2) as isize));
              ptr::write(self.s_stack().inner.offset((self.s_stack().len-1) as isize), ptr::read(self.s_stack().inner.offset((self.s_stack().len-3) as isize)));
              ptr::write(self.s_stack().inner.offset((self.s_stack().len-2) as isize), ptr::read(self.s_stack().inner.offset((self.s_stack().len-4) as isize)));
              ptr::write(self.s_stack().inner.offset((self.s_stack().len-3) as isize), t);
              ptr::write(self.s_stack().inner.offset((self.s_stack().len-4) as isize), n);
          }
          Ok(())
      }
  }

  fn two_over(&mut self) -> Result {
      if self.s_stack().len < 4 {
          Err(StackUnderflow)
      } else if self.s_stack().len + 2 > self.s_stack().cap {
          Err(StackOverflow)
      } else {
          unsafe {
              self.s_stack().len += 2;
              ptr::write(self.s_stack().inner.offset((self.s_stack().len-1) as isize), ptr::read(self.s_stack().inner.offset((self.s_stack().len-5) as isize)));
              ptr::write(self.s_stack().inner.offset((self.s_stack().len-2) as isize), ptr::read(self.s_stack().inner.offset((self.s_stack().len-6) as isize)));
          }
          Ok(())
      }
  }

  fn depth(&mut self) -> Result {
      let len = self.s_stack().len;
      match self.s_stack().push(len as isize) {
          Some(_) => Err(StackOverflow),
          None => Ok(())
      }
  }

  fn one_plus(&mut self) -> Result {
      if self.s_stack().len < 1 {
          Err(StackUnderflow)
      } else {
          unsafe {
              ptr::write(self.s_stack().inner.offset((self.s_stack().len-1) as isize), ptr::read(self.s_stack().inner.offset((self.s_stack().len-1) as isize)).wrapping_add(1));
          }
          Ok(())
      }
  }

  fn one_minus(&mut self) -> Result {
      if self.s_stack().len < 1 {
          Err(StackUnderflow)
      } else {
          unsafe {
              ptr::write(self.s_stack().inner.offset((self.s_stack().len-1) as isize), ptr::read(self.s_stack().inner.offset((self.s_stack().len-1) as isize))-1);
          }
          Ok(())
      }
  }

  fn plus(&mut self) -> Result {
      if self.s_stack().len < 2 {
          Err(StackUnderflow)
      } else {
          unsafe {
              self.s_stack().len -= 1;
              ptr::write(self.s_stack().inner.offset((self.s_stack().len-1) as isize),
                  ptr::read(self.s_stack().inner.offset((self.s_stack().len-1) as isize)) + ptr::read(self.s_stack().inner.offset((self.s_stack().len) as isize)));
          }
          Ok(())
      }
  }

  fn minus(&mut self) -> Result {
      if self.s_stack().len < 2 {
          Err(StackUnderflow)
      } else {
          unsafe {
              self.s_stack().len -= 1;
              ptr::write(self.s_stack().inner.offset((self.s_stack().len-1) as isize),
                  ptr::read(self.s_stack().inner.offset((self.s_stack().len-1) as isize)) - ptr::read(self.s_stack().inner.offset((self.s_stack().len) as isize)));
          }
          Ok(())
      }
  }

  fn star(&mut self) -> Result {
      if self.s_stack().len < 2 {
          Err(StackUnderflow)
      } else {
          unsafe {
              self.s_stack().len -= 1;
              ptr::write(self.s_stack().inner.offset((self.s_stack().len-1) as isize),
                  ptr::read(self.s_stack().inner.offset((self.s_stack().len-1) as isize)) * ptr::read(self.s_stack().inner.offset((self.s_stack().len) as isize)));
          }
          Ok(())
      }
  }

  fn slash(&mut self) -> Result {
      if self.s_stack().len < 2 {
          Err(StackUnderflow)
      } else {
          unsafe {
              self.s_stack().len -= 1;
              ptr::write(self.s_stack().inner.offset((self.s_stack().len-1) as isize),
                  ptr::read(self.s_stack().inner.offset((self.s_stack().len-1) as isize)) / ptr::read(self.s_stack().inner.offset((self.s_stack().len) as isize)));
          }
          Ok(())
      }
  }

  fn p_mod(&mut self) -> Result {
      if self.s_stack().len < 2 {
          Err(StackUnderflow)
      } else {
          unsafe {
              self.s_stack().len -= 1;
              ptr::write(self.s_stack().inner.offset((self.s_stack().len-1) as isize),
                  ptr::read(self.s_stack().inner.offset((self.s_stack().len-1) as isize)) % ptr::read(self.s_stack().inner.offset((self.s_stack().len) as isize)));
          }
          Ok(())
      }
  }

  fn slash_mod(&mut self) -> Result {
      if self.s_stack().len < 2 {
          Err(StackUnderflow)
      } else {
          unsafe {
              let t = ptr::read(self.s_stack().inner.offset((self.s_stack().len-1) as isize));
              let n = ptr::read(self.s_stack().inner.offset((self.s_stack().len-2) as isize));
              ptr::write(self.s_stack().inner.offset((self.s_stack().len-2) as isize), n%t);
              ptr::write(self.s_stack().inner.offset((self.s_stack().len-1) as isize), n/t);
          }
          Ok(())
      }
  }

  fn abs(&mut self) -> Result {
      match self.s_stack().pop() {
          Some(t) =>
              match self.s_stack().push(t.abs()) {
                  Some(_) => Err(StackOverflow),
                  None => Ok(())
              },
          None => Err(StackUnderflow)
      }
  }

  fn negate(&mut self) -> Result {
      match self.s_stack().pop() {
          Some(t) =>
              match self.s_stack().push(-t) {
                  Some(_) => Err(StackOverflow),
                  None => Ok(())
              },
          None => Err(StackUnderflow)
      }
  }

  fn zero_less(&mut self) -> Result {
      match self.s_stack().pop() {
          Some(t) =>
              match self.s_stack().push(if t<0 { TRUE } else { FALSE }) {
                  Some(_) => Err(StackOverflow),
                  None => Ok(())
              },
          None => Err(StackUnderflow)
      }
  }

  fn zero_equals(&mut self) -> Result {
      match self.s_stack().pop() {
          Some(t) =>
              match self.s_stack().push(if t==0 { TRUE } else { FALSE }) {
                  Some(_) => Err(StackOverflow),
                  None => Ok(())
              },
          None => Err(StackUnderflow)
      }
  }

  fn zero_greater(&mut self) -> Result {
      match self.s_stack().pop() {
          Some(t) =>
              match self.s_stack().push(if t>0 { TRUE } else { FALSE }) {
                  Some(_) => Err(StackOverflow),
                  None => Ok(())
              },
          None => Err(StackUnderflow)
      }
  }

  fn zero_not_equals(&mut self) -> Result {
      match self.s_stack().pop() {
          Some(t) =>
              match self.s_stack().push(if t==0 { FALSE } else { TRUE }) {
                  Some(_) => Err(StackOverflow),
                  None => Ok(())
              },
          None => Err(StackUnderflow)
      }
  }

  fn equals(&mut self) -> Result {
      match self.s_stack().pop2() {
          Some((n,t)) =>
              match self.s_stack().push(if t==n { TRUE } else { FALSE }) {
                  Some(_) => Err(StackOverflow),
                  None => Ok(())
              },
          None => Err(StackUnderflow)
      }
  }

  fn less_than(&mut self) -> Result {
      match self.s_stack().pop2() {
          Some((n,t)) =>
              match self.s_stack().push(if n<t { TRUE } else { FALSE }) {
                  Some(_) => Err(StackOverflow),
                  None => Ok(())
              },
          None => Err(StackUnderflow)
      }
  }

  fn greater_than(&mut self) -> Result {
      match self.s_stack().pop2() {
          Some((n,t)) =>
              match self.s_stack().push(if n>t { TRUE } else { FALSE }) {
                  Some(_) => Err(StackOverflow),
                  None => Ok(())
              },
          None => Err(StackUnderflow)
      }
  }

  fn not_equals(&mut self) -> Result {
      match self.s_stack().pop2() {
          Some((n,t)) =>
              match self.s_stack().push(if n==t { FALSE } else { TRUE }) {
                  Some(_) => Err(StackOverflow),
                  None => Ok(())
              },
          None => Err(StackUnderflow)
      }
  }

  fn between(&mut self) -> Result {
      match self.s_stack().pop3() {
          Some((x1, x2, x3)) =>
              match self.s_stack().push(if x2<=x1 && x1<=x3 { TRUE } else { FALSE }) {
                  Some(_) => Err(StackOverflow),
                  None => Ok(())
              },
          None => Err(StackUnderflow)
      }
  }

  fn invert(&mut self) -> Result {
      match self.s_stack().pop() {
          Some(t) =>
              match self.s_stack().push(!t) {
                  Some(_) => Err(StackOverflow),
                  None => Ok(())
              },
          None => Err(StackUnderflow)
      }
  }

  fn and(&mut self) -> Result {
      match self.s_stack().pop2() {
          Some((n,t)) =>
              match self.s_stack().push(t & n) {
                  Some(_) => Err(StackOverflow),
                  None => Ok(())
              },
          None => Err(StackUnderflow)
      }
  }

  fn or(&mut self) -> Result {
      match self.s_stack().pop2() {
          Some((n,t)) =>
              match self.s_stack().push(t | n) {
                  Some(_) => Err(StackOverflow),
                  None => Ok(())
              },
          None => Err(StackUnderflow)
      }
  }

  fn xor(&mut self) -> Result {
      match self.s_stack().pop2() {
          Some((n,t)) =>
              match self.s_stack().push(t ^ n) {
                  Some(_) => Err(StackOverflow),
                  None => Ok(())
              },
          None => Err(StackUnderflow)
      }
  }

  /// Run-time: ( x1 u -- x2 )
  ///
  /// Perform a logical left shift of `u` bit-places on `x1`, giving `x2`. Put
  /// zeroes into the least significant bits vacated by the shift. An
  /// ambiguous condition exists if `u` is greater than or equal to the number
  /// of bits in a cell.
  fn lshift(&mut self) -> Result {
      match self.s_stack().pop2() {
          Some((n,t)) =>
              match self.s_stack().push(n << t) {
                  Some(_) => Err(StackOverflow),
                  None => Ok(())
              },
          None => Err(StackUnderflow)
      }
  }

  /// Run-time: ( x1 u -- x2 )
  ///
  /// Perform a logical right shift of `u` bit-places on `x1`, giving `x2`. Put
  /// zeroes into the most significant bits vacated by the shift. An
  /// ambiguous condition exists if `u` is greater than or equal to the number
  /// of bits in a cell.
  fn rshift(&mut self) -> Result {
      match self.s_stack().pop2() {
          Some((n,t)) =>
              match self.s_stack().push((n as usize >> t) as isize) {
                  Some(_) => Err(StackOverflow),
                  None => Ok(())
              },
          None => Err(StackUnderflow)
      }
  }

  /// Run-time: ( x1 u -- x2 )
  ///
  /// Perform a arithmetic right shift of `u` bit-places on `x1`, giving `x2`. Put
  /// zeroes into the most significant bits vacated by the shift. An
  /// ambiguous condition exists if `u` is greater than or equal to the number
  /// of bits in a cell.
  fn arshift(&mut self) -> Result {
      match self.s_stack().pop2() {
          Some((n,t)) =>
              match self.s_stack().push(n >> t) {
                  Some(_) => Err(StackOverflow),
                  None => Ok(())
              },
          None => Err(StackUnderflow)
      }
  }

  /// Interpretation: Interpretation semantics for this word are undefined.
  ///
  /// Execution: ( -- ) ( R: nest-sys -- )
  /// Return control to the calling definition specified by `nest-sys`. Before executing `EXIT` within a
  /// do-loop, a program shall discard the loop-control parameters by executing `UNLOOP`.
  /// TODO: `UNLOOP`
  fn exit(&mut self) -> Result {
      if self.r_stack().len == 0 {
          Err(ReturnStackUnderflow)
      } else {
          self.r_stack().len -= 1;
          unsafe {
              self.state().instruction_pointer = ptr::read(self.r_stack().inner.offset(self.r_stack().len as isize)) as usize;
          }
          Ok(())
      }
  }

  /// Run-time: ( a-addr -- x )
  ///
  /// `x` is the value stored at `a-addr`.
  fn fetch(&mut self) -> Result {
      match self.s_stack().pop() {
          Some(t) => {
            let value = self.jit_memory().get_i32(t as usize) as isize;
            match self.s_stack().push(value) {
                Some(_) => Err(StackOverflow),
                None => Ok(())
            }
          },
          None => Err(StackUnderflow)
      }
  }

  /// Run-time: ( x a-addr -- )
  ///
  /// Store `x` at `a-addr`.
  fn store(&mut self) -> Result {
      match self.s_stack().pop2() {
          Some((n,t)) => {
              self.jit_memory().put_i32(n as i32, t as usize);
              Ok(())
          },
          None => Err(StackUnderflow)
      }
  }

  /// Run-time: ( c-addr -- char )
  ///
  /// Fetch the character stored at `c-addr`. When the cell size is greater than
  /// character size, the unused high-order bits are all zeroes.
  fn c_fetch(&mut self) -> Result {
      match self.s_stack().pop() {
          Some(t) => {
              let value = self.jit_memory().get_u8(t as usize) as isize;
              match self.s_stack().push(value) {
                  Some(_) => Err(StackOverflow),
                  None => Ok(())
              }
          },
          None => Err(StackUnderflow)
      }
  }

  /// Run-time: ( char c-addr -- )
  ///
  /// Store `char` at `c-addr`. When character size is smaller than cell size,
  /// only the number of low-order bits corresponding to character size are
  /// transferred.
  fn c_store(&mut self) -> Result {
      match self.s_stack().pop2() {
          Some((n,t)) => {
              self.jit_memory().put_u8(n as u8, t as usize);
              Ok(())
          },
          None => Err(StackUnderflow)
      }
  }

  /// Run-time: ( "<spaces>name" -- xt )
  ///
  /// Skip leading space delimiters. Parse name delimited by a space. Find
  /// `name` and return `xt`, the execution token for name. An ambiguous
  /// condition exists if name is not found.
  fn tick(&mut self) -> Result {
      let result;
      try!(self.parse_word());
      let last_token = self.last_token().take().unwrap();
      if last_token.is_empty() {
          result = Err(UnexpectedEndOfFile);
      } else {
          match self.find(&last_token) {
              Some(found_index) =>
                  match self.s_stack().push(found_index as isize) {
                      Some(_) => result = Err(StackOverflow),
                      None => result = Ok(())
                  },
              None => result = Err(UndefinedWord)
          }
      }
      self.set_last_token(last_token);
      result
  }

  /// Run-time: ( i*x xt -- j*x )
  ///
  /// Remove `xt` from the stack and perform the semantics identified by it.
  /// Other stack effects are due to the word `EXECUTE`d.
  fn execute(&mut self) -> Result {
      match self.s_stack().pop() {
          Some(t) => {
              self.execute_word(t as usize)
          },
          None => Err(StackUnderflow)
      }
  }

  /// Run-time: ( -- addr )
  ///
  /// `addr` is the data-space pointer.
  fn here(&mut self) -> Result {
      let len = self.jit_memory().len() as isize;
      match self.s_stack().push(len) {
          Some(_) => Err(StackOverflow),
          None => Ok(())
      }
  }

  /// Run-time: ( n -- )
  ///
  /// If `n` is greater than zero, reserve n address units of data space. If `n`
  /// is less than zero, release `|n|` address units of data space. If `n` is
  /// zero, leave the data-space pointer unchanged.
  fn allot(&mut self) -> Result {
      match self.s_stack().pop() {
          Some(v) => {
              self.jit_memory().allot(v);
              Ok(())
          },
          None => Err(StackUnderflow)
      }
  }

  /// Run-time: ( x -- )
  ///
  /// Reserve one cell of data space and store `x` in the cell. If the
  /// data-space pointer is aligned when `,` begins execution, it will remain
  /// aligned when `,` finishes execution. An ambiguous condition exists if the
  /// data-space pointer is not aligned prior to execution of `,`.
  fn comma(&mut self) -> Result {
      match self.s_stack().pop() {
          Some(v) => {
              self.jit_memory().compile_i32(v as i32);
              Ok(())
          },
          None => Err(StackUnderflow)
      }
  }

  fn p_to_r(&mut self) -> Result {
      match self.s_stack().pop() {
          Some(v) => {
              if self.r_stack().is_full() {
                  Err(ReturnStackOverflow)
              } else {
                  unsafe {
                      ptr::write(self.r_stack().inner.offset(self.r_stack().len as isize), v);
                  }
                  self.r_stack().len += 1;
                  Ok(())
              }
          },
          None => Err(StackUnderflow)
      }
  }

  fn r_from(&mut self) -> Result {
      if self.r_stack().len == 0 {
          Err(ReturnStackUnderflow)
      } else {
          self.r_stack().len -= 1;
          unsafe {
              let r0 = self.r_stack().inner.offset(self.r_stack().len as isize);
              self.s_stack().push(ptr::read(r0));
          }
          Ok(())
      }
  }

  fn r_fetch(&mut self) -> Result {
      if self.r_stack().len == 0 {
          Err(ReturnStackUnderflow)
      } else {
          unsafe {
              let r1 = self.r_stack().inner.offset((self.r_stack().len-1) as isize);
              self.s_stack().push(ptr::read(r1));
          }
          Ok(())
      }
  }

  fn two_to_r(&mut self) -> Result {
      match self.s_stack().pop2() {
          Some((n,t)) =>
              if self.r_stack().space_left() < 2 {
                  Err(ReturnStackOverflow)
              } else {
                  unsafe {
                      ptr::write(self.r_stack().inner.offset(self.r_stack().len as isize), n);
                      ptr::write(self.r_stack().inner.offset((self.r_stack().len+1) as isize), t);
                  }
                  self.r_stack().len += 2;
                  Ok(())
              },
          None => Err(StackUnderflow)
      }
  }

  fn two_r_from(&mut self) -> Result {
      if self.r_stack().len < 2 {
          Err(ReturnStackUnderflow)
      } else {
          self.r_stack().len -= 2;
          unsafe {
              let r0 = self.r_stack().inner.offset(self.r_stack().len as isize);
              self.s_stack().push(ptr::read(r0));
              let r1 = self.r_stack().inner.offset((self.r_stack().len+1) as isize);
              self.s_stack().push(ptr::read(r1));
          }
          Ok(())
      }
  }

  fn two_r_fetch(&mut self) -> Result {
      if self.r_stack().len < 2 {
          Err(ReturnStackUnderflow)
      } else {
          unsafe {
              let r2 = self.r_stack().inner.offset((self.r_stack().len-2) as isize);
              self.s_stack().push(ptr::read(r2));
              let r1 = self.r_stack().inner.offset((self.r_stack().len-1) as isize);
              self.s_stack().push(ptr::read(r1));
          }
          Ok(())
      }
  }

  /// Leave VM's inner loop, keep VM's all state.
  /// Call inner to resume inner loop.
  fn pause(&mut self) -> Result {
      Err(Pause)
  }

  //----------------
  // Error handlling
  //----------------

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
      self.state().instruction_pointer = 0;
      self.interpret();
  }

  /// Abort the inner loop with an exception, reset VM and clears stacks.
  fn abort(&mut self) -> Result {
      self.clear_stacks();
      self.reset();
      Err(Abort)
  }

  fn halt(&mut self) -> Result {
      self.state().instruction_pointer = 0;
      Err(Quit)
  }

  /// Quit the inner loop and reset VM, without clearing stacks .
  fn quit(&mut self) -> Result {
      self.reset();
      Err(Quit)
  }

  /// Emit Bye exception.
  fn bye(&mut self) -> Result {
      Err(Bye)
  }

}

#[cfg(test)]

mod tests {
    extern crate test;
    use super::Core;
    use vm::VM;
    use self::test::Bencher;
    use std::mem;
    use exception::Exception::{
        InvalidMemoryAddress,
        Abort,
        Quit,
        Pause,
        Bye
    };

    #[bench]
    fn bench_noop (b: &mut Bencher) {
        let vm = &mut VM::new(16);
        b.iter(|| vm.noop());
    }

    #[test]
    fn test_find() {
        let vm = &mut VM::new(16);
        vm.add_core();
        assert!(vm.find("").is_none());
        assert!(vm.find("word-not-exist").is_none());
        vm.find("noop").expect("noop not found");
    }

    #[bench]
    fn bench_find_word_not_exist(b: &mut Bencher) {
        let vm = &mut VM::new(16);
        vm.add_core();
        b.iter(|| vm.find("unknown"));
    }

    #[bench]
    fn bench_find_word_at_beginning_of_wordlist(b: &mut Bencher) {
        let vm = &mut VM::new(16);
        vm.add_core();
        b.iter(|| vm.find("noop"));
    }

    #[bench]
    fn bench_find_word_at_end_of_wordlist(b: &mut Bencher) {
        let vm = &mut VM::new(16);
        vm.add_core();
        b.iter(|| vm.find("bye"));
    }

    #[test]
    fn test_inner_interpreter_without_nest () {
        let vm = &mut VM::new(16);
        vm.add_core();
        let ip = vm.jit_memory().len();
        vm.compile_integer(3);
        vm.compile_integer(2);
        vm.compile_integer(1);
        vm.state().instruction_pointer = ip;
        match vm.run() {
            Err(e) => {
                match e {
                    InvalidMemoryAddress => assert!(true),
                    _ => assert!(false)
                }
            },
            Ok(()) => assert!(false)
        }
        assert_eq!(3usize, vm.s_stack().len());
    }

    #[bench]
    fn bench_inner_interpreter_without_nest (b: &mut Bencher) {
        let vm = &mut VM::new(16);
        vm.add_core();
        let ip = vm.jit_memory().len();
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
        vm.add_core();
        vm.s_stack().push(1);
        assert!(vm.p_drop().is_ok());
        assert!(vm.s_stack().is_empty());
    }

    #[bench]
    fn bench_drop(b: &mut Bencher) {
        let vm = &mut VM::new(16);
        vm.add_core();
        vm.s_stack().push(1);
        b.iter(|| {
            vm.p_drop();
            vm.s_stack().push(1);
        });
    }

    #[test]
    fn test_nip() {
        let vm = &mut VM::new(16);
        vm.add_core();
        vm.s_stack().push(1);
        vm.s_stack().push(2);
        assert!(vm.nip().is_ok());
        assert!(vm.s_stack().len()==1);
        assert!(vm.s_stack().last() == Some(2));
    }

    #[bench]
    fn bench_nip(b: &mut Bencher) {
        let vm = &mut VM::new(16);
        vm.add_core();
        vm.s_stack().push(1);
        vm.s_stack().push(1);
        b.iter(|| {
            vm.nip();
            vm.s_stack().push(1);
        });
    }

    #[test]
    fn test_swap () {
        let vm = &mut VM::new(16);
        vm.add_core();
        vm.s_stack().push(1);
        vm.s_stack().push(2);
        assert!(vm.swap().is_ok());
        assert_eq!(vm.s_stack().len(), 2);
        assert_eq!(vm.s_stack().pop(), Some(1));
        assert_eq!(vm.s_stack().pop(), Some(2));
    }

    #[bench]
    fn bench_swap (b: &mut Bencher) {
        let vm = &mut VM::new(16);
        vm.add_core();
        vm.s_stack().push(1);
        vm.s_stack().push(2);
        b.iter(|| vm.swap());
    }

    #[test]
    fn test_dup () {
        let vm = &mut VM::new(16);
        vm.add_core();
        vm.s_stack().push(1);
        assert!(vm.dup().is_ok());
        assert_eq!(vm.s_stack().len(), 2);
        assert_eq!(vm.s_stack().pop(), Some(1));
        assert_eq!(vm.s_stack().pop(), Some(1));
    }

    #[bench]
    fn bench_dup (b: &mut Bencher) {
        let vm = &mut VM::new(16);
        vm.add_core();
        vm.s_stack().push(1);
        b.iter(|| {
            vm.dup();
            vm.s_stack().pop();
        });
    }

    #[test]
    fn test_over () {
        let vm = &mut VM::new(16);
        vm.add_core();
        vm.s_stack().push(1);
        vm.s_stack().push(2);
        assert!(vm.over().is_ok());
        assert_eq!(vm.s_stack().len(), 3);
        assert_eq!(vm.s_stack().pop(), Some(1));
        assert_eq!(vm.s_stack().pop(), Some(2));
        assert_eq!(vm.s_stack().pop(), Some(1));
    }

    #[bench]
    fn bench_over (b: &mut Bencher) {
        let vm = &mut VM::new(16);
        vm.add_core();
        vm.s_stack().push(1);
        vm.s_stack().push(2);
        b.iter(|| {
            vm.over();
            vm.s_stack().pop();
        });
    }

    #[test]
    fn test_rot () {
        let vm = &mut VM::new(16);
        vm.add_core();
        vm.s_stack().push(1);
        vm.s_stack().push(2);
        vm.s_stack().push(3);
        assert!(vm.rot().is_ok());
        assert_eq!(vm.s_stack().len(), 3);
        assert_eq!(vm.s_stack().pop(), Some(1));
        assert_eq!(vm.s_stack().pop(), Some(3));
        assert_eq!(vm.s_stack().pop(), Some(2));
    }

    #[bench]
    fn bench_rot (b: &mut Bencher) {
        let vm = &mut VM::new(16);
        vm.add_core();
        vm.s_stack().push(1);
        vm.s_stack().push(2);
        vm.s_stack().push(3);
        b.iter(|| vm.rot());
    }

    #[test]
    fn test_2drop () {
        let vm = &mut VM::new(16);
        vm.add_core();
        vm.s_stack().push(1);
        vm.s_stack().push(2);
        assert!(vm.two_drop().is_ok());
        assert!(vm.s_stack().is_empty());
    }

    #[bench]
    fn bench_2drop (b: &mut Bencher) {
        let vm = &mut VM::new(16);
        vm.add_core();
        b.iter(|| {
            vm.s_stack().push(1);
            vm.s_stack().push(2);
            vm.two_drop();
        });
    }

    #[test]
    fn test_2dup () {
        let vm = &mut VM::new(16);
        vm.add_core();
        vm.s_stack().push(1);
        vm.s_stack().push(2);
        assert!(vm.two_dup().is_ok());
        assert_eq!(vm.s_stack().len(), 4);
        assert_eq!(vm.s_stack().pop(), Some(2));
        assert_eq!(vm.s_stack().pop(), Some(1));
        assert_eq!(vm.s_stack().pop(), Some(2));
        assert_eq!(vm.s_stack().pop(), Some(1));
    }

    #[bench]
    fn bench_2dup (b: &mut Bencher) {
        let vm = &mut VM::new(16);
        vm.add_core();
        vm.s_stack().push(1);
        vm.s_stack().push(2);
        b.iter(|| {
            vm.two_dup();
            vm.two_drop();
        });
    }

    #[test]
    fn test_2swap () {
        let vm = &mut VM::new(16);
        vm.add_core();
        vm.s_stack().push(1);
        vm.s_stack().push(2);
        vm.s_stack().push(3);
        vm.s_stack().push(4);
        assert!(vm.two_swap().is_ok());
        assert_eq!(vm.s_stack().len(), 4);
        assert_eq!(vm.s_stack().pop(), Some(2));
        assert_eq!(vm.s_stack().pop(), Some(1));
        assert_eq!(vm.s_stack().pop(), Some(4));
        assert_eq!(vm.s_stack().pop(), Some(3));
    }

    #[bench]
    fn bench_2swap (b: &mut Bencher) {
        let vm = &mut VM::new(16);
        vm.add_core();
        vm.s_stack().push(1);
        vm.s_stack().push(2);
        vm.s_stack().push(3);
        vm.s_stack().push(4);
        b.iter(|| vm.two_swap());
    }

    #[test]
    fn test_2over () {
        let vm = &mut VM::new(16);
        vm.add_core();
        vm.s_stack().push(1);
        vm.s_stack().push(2);
        vm.s_stack().push(3);
        vm.s_stack().push(4);
        assert!(vm.two_over().is_ok());
        assert_eq!(vm.s_stack().len(), 6);
        assert_eq!(vm.s_stack().as_slice(), [1, 2, 3, 4, 1, 2]);
    }

    #[bench]
    fn bench_2over (b: &mut Bencher) {
        let vm = &mut VM::new(16);
        vm.add_core();
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
        let vm = &mut VM::new(16);
        vm.add_core();
        vm.depth();
        vm.depth();
        vm.depth();
        assert_eq!(vm.s_stack().as_slice(), [0, 1, 2]);
    }

    #[test]
    fn test_one_plus() {
        let vm = &mut VM::new(16);
        vm.add_core();
        vm.s_stack().push(1);
        assert!(vm.one_plus().is_ok());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), Some(2));
    }

    #[bench]
    fn bench_one_plus(b: &mut Bencher) {
        let vm = &mut VM::new(16);
        vm.add_core();
        vm.s_stack().push(0);
        b.iter(|| {
            vm.one_plus();
        });
    }

    #[test]
    fn test_one_minus() {
        let vm = &mut VM::new(16);
        vm.add_core();
        vm.s_stack().push(2);
        assert!(vm.one_minus().is_ok());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), Some(1));
    }

    #[bench]
    fn bench_one_minus(b: &mut Bencher) {
        let vm = &mut VM::new(16);
        vm.add_core();
        vm.s_stack().push(0);
        b.iter(|| {
            vm.one_minus();
        });
    }

    #[test]
    fn test_minus() {
        let vm = &mut VM::new(16);
        vm.add_core();
        vm.s_stack().push(5);
        vm.s_stack().push(7);
        assert!(vm.minus().is_ok());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), Some(-2));
    }

    #[bench]
    fn bench_minus(b: &mut Bencher) {
        let vm = &mut VM::new(16);
        vm.add_core();
        vm.s_stack().push(0);
        b.iter(|| {
            vm.dup();
            vm.minus();
        });
    }

    #[test]
    fn test_plus() {
        let vm = &mut VM::new(16);
        vm.add_core();
        vm.s_stack().push(5);
        vm.s_stack().push(7);
        assert!(vm.plus().is_ok());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), Some(12));
    }

    #[bench]
    fn bench_plus(b: &mut Bencher) {
        let vm = &mut VM::new(16);
        vm.add_core();
        vm.s_stack().push(1);
        b.iter(|| {
            vm.dup();
            vm.plus();
        });
    }

    #[test]
    fn test_star () {
        let vm = &mut VM::new(16);
        vm.add_core();
        vm.s_stack().push(5);
        vm.s_stack().push(7);
        assert!(vm.star().is_ok());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), Some(35));
    }

    #[bench]
    fn bench_star(b: &mut Bencher) {
        let vm = &mut VM::new(16);
        vm.add_core();
        vm.s_stack().push(1);
        b.iter(|| {
            vm.dup();
            vm.star();
        });
    }

    #[test]
    fn test_slash () {
        let vm = &mut VM::new(16);
        vm.add_core();
        vm.s_stack().push(30);
        vm.s_stack().push(7);
        assert!(vm.slash().is_ok());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), Some(4));
    }

    #[bench]
    fn bench_slash(b: &mut Bencher) {
        let vm = &mut VM::new(16);
        vm.add_core();
        vm.s_stack().push(1);
        b.iter(|| {
            vm.dup();
            vm.slash();
        });
    }

    #[test]
    fn test_mod () {
        let vm = &mut VM::new(16);
        vm.add_core();
        vm.s_stack().push(30);
        vm.s_stack().push(7);
        assert!(vm.p_mod().is_ok());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), Some(2));
    }

    #[bench]
    fn bench_mod(b: &mut Bencher) {
        let vm = &mut VM::new(16);
        vm.add_core();
        vm.s_stack().push(1);
        vm.s_stack().push(2);
        b.iter(|| {
            vm.p_mod();
            vm.s_stack().push(2);
        });
    }

    #[test]
    fn test_slash_mod () {
        let vm = &mut VM::new(16);
        vm.add_core();
        vm.s_stack().push(30);
        vm.s_stack().push(7);
        assert!(vm.slash_mod().is_ok());
        assert_eq!(vm.s_stack().len(), 2);
        assert_eq!(vm.s_stack().pop(), Some(4));
        assert_eq!(vm.s_stack().pop(), Some(2));
    }

    #[bench]
    fn bench_slash_mod(b: &mut Bencher) {
        let vm = &mut VM::new(16);
        vm.add_core();
        vm.s_stack().push2(1, 2);
        b.iter(|| {
            vm.slash_mod();
            vm.p_drop();
            vm.s_stack().push(2)
        });
    }

    #[test]
    fn test_abs () {
        let vm = &mut VM::new(16);
        vm.add_core();
        vm.s_stack().push(-30);
        assert!(vm.abs().is_ok());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), Some(30));
    }

    #[test]
    fn test_negate () {
        let vm = &mut VM::new(16);
        vm.add_core();
        vm.s_stack().push(30);
        assert!(vm.negate().is_ok());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), Some(-30));
    }

    #[test]
    fn test_zero_less () {
        let vm = &mut VM::new(16);
        vm.add_core();
        vm.s_stack().push(-1);
        assert!(vm.zero_less().is_ok());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), Some(-1));
        vm.s_stack().push(0);
        assert!(vm.zero_less().is_ok());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), Some(0));
    }

    #[test]
    fn test_zero_equals () {
        let vm = &mut VM::new(16);
        vm.add_core();
        vm.s_stack().push(0);
        assert!(vm.zero_equals().is_ok());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), Some(-1));
        vm.s_stack().push(-1);
        assert!(vm.zero_equals().is_ok());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), Some(0));
        vm.s_stack().push(1);
        assert!(vm.zero_equals().is_ok());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), Some(0));
    }

    #[test]
    fn test_zero_greater () {
        let vm = &mut VM::new(16);
        vm.add_core();
        vm.s_stack().push(1);
        assert!(vm.zero_greater().is_ok());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), Some(-1));
        vm.s_stack().push(0);
        assert!(vm.zero_greater().is_ok());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), Some(0));
    }

    #[test]
    fn test_zero_not_equals () {
        let vm = &mut VM::new(16);
        vm.add_core();
        vm.s_stack().push(0);
        assert!(vm.zero_not_equals().is_ok());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), Some(0));
        vm.s_stack().push(-1);
        assert!(vm.zero_not_equals().is_ok());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), Some(-1));
        vm.s_stack().push(1);
        assert!(vm.zero_not_equals().is_ok());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), Some(-1));
    }

    #[test]
    fn test_less_than () {
        let vm = &mut VM::new(16);
        vm.add_core();
        vm.s_stack().push(-1);
        vm.s_stack().push(0);
        assert!(vm.less_than().is_ok());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), Some(-1));
        vm.s_stack().push(0);
        vm.s_stack().push(0);
        assert!(vm.less_than().is_ok());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), Some(0));
    }

    #[test]
    fn test_equals () {
        let vm = &mut VM::new(16);
        vm.add_core();
        vm.s_stack().push(0);
        vm.s_stack().push(0);
        assert!(vm.equals().is_ok());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), Some(-1));
        vm.s_stack().push(-1);
        vm.s_stack().push(0);
        assert!(vm.equals().is_ok());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), Some(0));
        vm.s_stack().push(1);
        vm.s_stack().push(0);
        assert!(vm.equals().is_ok());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), Some(0));
    }

    #[test]
    fn test_greater_than () {
        let vm = &mut VM::new(16);
        vm.add_core();
        vm.s_stack().push(1);
        vm.s_stack().push(0);
        assert!(vm.greater_than().is_ok());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), Some(-1));
        vm.s_stack().push(0);
        vm.s_stack().push(0);
        assert!(vm.greater_than().is_ok());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), Some(0));
    }

    #[test]
    fn test_not_equals () {
        let vm = &mut VM::new(16);
        vm.add_core();
        vm.s_stack().push(0);
        vm.s_stack().push(0);
        assert!(vm.not_equals().is_ok());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), Some(0));
        vm.s_stack().push(-1);
        vm.s_stack().push(0);
        assert!(vm.not_equals().is_ok());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), Some(-1));
        vm.s_stack().push(1);
        vm.s_stack().push(0);
        assert!(vm.not_equals().is_ok());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), Some(-1));
    }

    #[test]
    fn test_between () {
        let vm = &mut VM::new(16);
        vm.add_core();
        vm.s_stack().push(1);
        vm.s_stack().push(1);
        vm.s_stack().push(2);
        assert!(vm.between().is_ok());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), Some(-1));
        vm.s_stack().push(1);
        vm.s_stack().push(0);
        vm.s_stack().push(1);
        assert!(vm.between().is_ok());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), Some(-1));
        vm.s_stack().push(0);
        vm.s_stack().push(1);
        vm.s_stack().push(2);
        assert!(vm.between().is_ok());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), Some(0));
        vm.s_stack().push(3);
        vm.s_stack().push(1);
        vm.s_stack().push(2);
        assert!(vm.between().is_ok());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), Some(0));
    }

    #[test]
    fn test_invert () {
        let vm = &mut VM::new(16);
        vm.add_core();
        vm.s_stack().push(707);
        assert!(vm.invert().is_ok());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), Some(-708));
    }

    #[test]
    fn test_and () {
        let vm = &mut VM::new(16);
        vm.add_core();
        vm.s_stack().push(707);
        vm.s_stack().push(007);
        assert!(vm.and().is_ok());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), Some(3));
    }

    #[test]
    fn test_or () {
        let vm = &mut VM::new(16);
        vm.add_core();
        vm.s_stack().push(707);
        vm.s_stack().push(07);
        assert!(vm.or().is_ok());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), Some(711));
    }

    #[test]
    fn test_xor () {
        let vm = &mut VM::new(16);
        vm.add_core();
        vm.s_stack().push(707);
        vm.s_stack().push(07);
        assert!(vm.xor().is_ok());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), Some(708));
    }

    #[test]
    fn test_lshift () {
        let vm = &mut VM::new(16);
        vm.add_core();
        vm.s_stack().push(1);
        vm.s_stack().push(1);
        assert!(vm.lshift().is_ok());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), Some(2));
        vm.s_stack().push(1);
        vm.s_stack().push(2);
        assert!(vm.lshift().is_ok());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), Some(4));
    }

    #[test]
    fn test_rshift () {
        let vm = &mut VM::new(16);
        vm.add_core();
        vm.s_stack().push(8);
        vm.s_stack().push(1);
        assert!(vm.rshift().is_ok());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), Some(4));
        vm.s_stack().push(-1);
        vm.s_stack().push(1);
        assert!(vm.rshift().is_ok());
        assert_eq!(vm.s_stack().len(), 1);
        assert!(vm.s_stack().pop().unwrap() > 0);
    }

    #[test]
    fn test_arshift () {
        let vm = &mut VM::new(16);
        vm.add_core();
        vm.s_stack().push(8);
        vm.s_stack().push(1);
        assert!(vm.arshift().is_ok());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), Some(4));
        vm.s_stack().push(-8);
        vm.s_stack().push(1);
        assert!(vm.arshift().is_ok());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), Some(-4));
    }

    #[test]
    fn test_parse_word () {
        let vm = &mut VM::new(16);
        vm.add_core();
        vm.set_source("hello world\t\r\n\"");
        assert!(vm.parse_word().is_ok());
        assert_eq!(vm.last_token().clone().unwrap(), "hello");
        assert_eq!(vm.state().source_index, 6);
        assert!(vm.parse_word().is_ok());
        assert_eq!(vm.last_token().clone().unwrap(), "world");
        assert_eq!(vm.state().source_index, 12);
        assert!(vm.parse_word().is_ok());
        assert_eq!(vm.last_token().clone().unwrap(), "\"");
    }

    #[test]
    fn test_evaluate () {
        let vm = &mut VM::new(16);
        vm.add_core();
        vm.set_source("false true dup 1+ 2 -3");
        assert!(vm.evaluate().is_ok());
        assert_eq!(vm.s_stack().len(), 5);
        assert_eq!(vm.s_stack().pop(), Some(-3));
        assert_eq!(vm.s_stack().pop(), Some(2));
        assert_eq!(vm.s_stack().pop(), Some(0));
        assert_eq!(vm.s_stack().pop(), Some(-1));
        assert_eq!(vm.s_stack().pop(), Some(0));
    }

    #[bench]
    fn bench_compile_words_at_beginning_of_wordlist (b: &mut Bencher) {
        let vm = &mut VM::new(16);
        vm.add_core();
        vm.set_source("marker empty");
        assert!(vm.evaluate().is_ok());
        b.iter(|| {
            vm.set_source(": main noop noop noop noop noop noop noop noop ; empty");
            vm.evaluate();
            vm.s_stack().clear();
        });
    }

    #[bench]
    fn bench_compile_words_at_end_of_wordlist(b: &mut Bencher) {
        let vm = &mut VM::new(16);
        vm.add_core();
        vm.set_source("marker empty");
        vm.evaluate();
        b.iter(|| {
            vm.set_source(": main bye bye bye bye bye bye bye bye ; empty");
            vm.evaluate();
            vm.s_stack().clear();
        });
    }

    #[test]
    fn test_colon_and_semi_colon() {
        let vm = &mut VM::new(16);
        vm.add_core();
        vm.set_source(": 2+3 2 3 + ; 2+3");
        assert!(vm.evaluate().is_ok());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), Some(5));
    }

    #[test]
    fn test_constant () {
        let vm = &mut VM::new(16);
        vm.add_core();
        vm.set_source("5 constant x x x");
        assert!(vm.evaluate().is_ok());
        assert_eq!(vm.s_stack().len(), 2);
        assert_eq!(vm.s_stack().pop(), Some(5));
        assert_eq!(vm.s_stack().pop(), Some(5));
    }

    #[test]
    fn test_variable_and_store_fetch () {
        let vm = &mut VM::new(16);
        vm.add_core();
        vm.set_source("variable x  x @  3 x !  x @");
        assert!(vm.evaluate().is_ok());
        assert_eq!(vm.s_stack().len(), 2);
        assert_eq!(vm.s_stack().pop(), Some(3));
        assert_eq!(vm.s_stack().pop(), Some(0));
    }

    #[test]
    fn test_char_plus_and_chars() {
        let vm = &mut VM::new(16);
        vm.add_core();
        vm.set_source("2 char+  9 chars");
        assert!(vm.evaluate().is_ok());
        assert_eq!(vm.s_stack().as_slice(), [3, 9]);
    }

    #[test]
    fn test_cell_plus_and_cells() {
        let vm = &mut VM::new(16);
        vm.add_core();
        vm.set_source("2 cell+  9 cells");
        assert!(vm.evaluate().is_ok());
        assert_eq!(vm.s_stack().as_slice(), [6, 36]);
    }

    #[test]
    fn test_execute () {
        let vm = &mut VM::new(16);
        vm.add_core();
        vm.set_source("1 2  ' swap execute");
        assert!(vm.evaluate().is_ok());
        assert_eq!(vm.s_stack().len(), 2);
        assert_eq!(vm.s_stack().pop(), Some(1));
        assert_eq!(vm.s_stack().pop(), Some(2));
    }

    #[test]
    fn test_here_allot () {
        let vm = &mut VM::new(16);
        vm.add_core();
        vm.set_source("here 2 cells allot here -");
        assert!(vm.evaluate().is_ok());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), Some(-((mem::size_of::<i32>()*2) as isize)));
    }

    #[test]
    fn test_here_comma_compile_interpret () {
        let vm = &mut VM::new(16);
        vm.add_core();
        let here = vm.jit_memory().len();
        vm.set_source("here 1 , 2 , ] lit exit [ here");
        assert!(vm.evaluate().is_ok());
        assert_eq!(vm.s_stack().len(), 2);
        match vm.s_stack().pop2() {
            Some((n, t)) => {
                assert_eq!(t-n, 4*mem::size_of::<u32>() as isize);
            },
            None => { assert!(false); }
        }
        let idx_halt = vm.find("halt").expect("halt undefined");
        assert_eq!(vm.jit_memory().get_i32(0), idx_halt as i32);
        assert_eq!(vm.jit_memory().get_i32(here+0), 1);
        assert_eq!(vm.jit_memory().get_i32(here+4), 2);
        assert_eq!(vm.jit_memory().get_i32(here+8), vm.references().idx_lit as i32);
        assert_eq!(vm.jit_memory().get_i32(here+12), vm.references().idx_exit as i32);
    }

    #[test]
    fn test_to_r_r_fetch_r_from () {
        let vm = &mut VM::new(16);
        vm.add_core();
        vm.set_source(": t 3 >r 2 r@ + r> + ; t");
        assert!(vm.evaluate().is_ok());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), Some(8));
    }

    #[bench]
    fn bench_to_r_r_fetch_r_from (b: &mut Bencher) {
        let vm = &mut VM::new(16);
        vm.add_core();
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
    fn test_two_to_r_two_r_fetch_two_r_from () {
        let vm = &mut VM::new(16);
        vm.add_core();
        vm.set_source(": t 1 2 2>r 2r@ + 2r> - * ; t");
        assert!(vm.evaluate().is_ok());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), Some(-3));
    }

    #[bench]
    fn bench_two_to_r_two_r_fetch_two_r_from (b: &mut Bencher) {
        let vm = &mut VM::new(16);
        vm.add_core();
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
    fn test_if_else_then () {
        let vm = &mut VM::new(16);
        vm.add_core();
        vm.set_source(": t1 0 if true else false then ; t1");
        assert!(vm.evaluate().is_ok());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), Some(0));
        vm.set_source(": t2 1 if true else false then ; t2");
        assert!(vm.evaluate().is_ok());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), Some(-1));
    }

    #[test]
    fn test_begin_again () {
        let vm = &mut VM::new(16);
        vm.add_core();
        vm.set_source(": t1 0 begin 1+ dup 3 = if exit then again ; t1");
        assert!(vm.evaluate().is_ok());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), Some(3));
    }

    #[test]
    fn test_begin_while_repeat () {
        let vm = &mut VM::new(16);
        vm.add_core();
        vm.set_source(": t1 0 begin 1+ dup 3 <> while repeat ; t1");
        assert!(vm.evaluate().is_ok());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), Some(3));
    }

    #[test]
    fn test_backlash () {
        let vm = &mut VM::new(16);
        vm.add_core();
        vm.set_source("1 2 3 \\ 5 6 7");
        assert!(vm.evaluate().is_ok());
        assert_eq!(vm.s_stack().len(), 3);
        assert_eq!(vm.s_stack().pop(), Some(3));
        assert_eq!(vm.s_stack().pop(), Some(2));
        assert_eq!(vm.s_stack().pop(), Some(1));
    }

    #[test]
    fn test_marker_unmark () {
        let vm = &mut VM::new(16);
        vm.add_core();
        vm.set_source("marker empty here empty here =");
        assert!(vm.evaluate().is_ok());
        let symbols_len = vm.symbols().len();
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), Some(-1));
        assert_eq!(vm.symbols().len(), symbols_len);
    }

    #[test]
    fn test_quit() {
        let vm = &mut VM::new(16);
        vm.add_core();
        vm.set_source(": main 1 2 ; main 3 quit 5 6 7");
        match vm.evaluate() {
            Err(_) => assert!(false),
            Ok(()) => assert!(true),
        };
        assert_eq!(vm.s_stack().len(), 3);
        assert_eq!(vm.s_stack().pop(), Some(3));
        assert_eq!(vm.s_stack().pop(), Some(2));
        assert_eq!(vm.s_stack().pop(), Some(1));
        assert_eq!(vm.r_stack().len, 0);
        assert_eq!(vm.input_buffer().clone().unwrap().len(), 0);
        assert_eq!(vm.state().source_index, 0);
        assert_eq!(vm.state().instruction_pointer, 0);
        assert!(!vm.state().is_compiling);
    }

    #[test]
    fn test_abort() {
        let vm = &mut VM::new(16);
        vm.add_core();
        vm.set_source("1 2 3 abort 5 6 7");
        match vm.evaluate() {
            Err(Abort) => assert!(true),
            _ => assert!(false)
        }
        assert_eq!(vm.s_stack().len(), 0);
    }

    #[test]
    fn test_bye() {
        let vm = &mut VM::new(16);
        vm.add_core();
        vm.set_source("1 2 3 bye 5 6 7");
        match vm.evaluate() {
            Err(Bye) => assert!(true),
            _ => assert!(false)
        }
        assert!(vm.state().is_idle());
    }

    #[test]
    fn test_pause() {
        let vm = &mut VM::new(16);
        vm.add_core();
        vm.set_source(": test 1 2 3 pause 5 6 7 ; test");
        match vm.evaluate() {
            Err(Pause) => assert!(true),
            _ => assert!(false)
        }
        assert!(!vm.state().is_idle());
        assert_eq!(vm.s_stack().len(), 3);
        vm.run();
        assert!(vm.state().is_idle());
        assert_eq!(vm.s_stack().len(), 6);
    }

    #[bench]
    fn bench_fib(b: &mut Bencher) {
        let vm = &mut VM::new(16);
        vm.add_core();
        vm.set_source(": fib dup 2 < if drop 1 else dup 1- recurse swap 2 - recurse + then ;");
        assert!(vm.evaluate().is_ok());
        vm.set_source(": main 7 fib drop ;");
        vm.evaluate();
        vm.set_source("' main");
        vm.evaluate();
        b.iter(|| {
            vm.dup();
            vm.execute();
            match vm.run() {
                Err(e) => {
                    match e {
                        Quit => {},
                        _ => {
                            assert!(false);
                        }
                    }
                },
                Ok(()) => assert!(true)
            };
        });
    }

    #[test]
    fn test_do_loop() {
        let vm = &mut VM::new(16);
        vm.add_core();
        vm.set_source(": main 1 5 0 do 1+ loop ;  main");
        assert!(vm.evaluate().is_ok());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), Some(6));
    }

    #[test]
    fn test_do_unloop_exit_loop() {
        let vm = &mut VM::new(16);
        vm.add_core();
        vm.set_source(": main 1 5 0 do 1+ dup 3 = if unloop exit then loop ;  main");
        assert!(vm.evaluate().is_ok());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), Some(3));
    }

    #[test]
    fn test_do_plus_loop() {
        let vm = &mut VM::new(16);
        vm.add_core();
        vm.set_source(": main 1 5 0 do 1+ 2 +loop ;  main");
        assert!(vm.evaluate().is_ok());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), Some(4));
        vm.set_source(": main 1 6 0 do 1+ 2 +loop ;  main");
        assert!(vm.evaluate().is_ok());
        assert_eq!(vm.s_stack().len(), 1);
        assert_eq!(vm.s_stack().pop(), Some(4));
    }

    #[test]
    fn test_do_leave_loop() {
        let vm = &mut VM::new(16);
        vm.add_core();
        vm.set_source(": main 1 5 0 do 1+ dup 3 = if drop 88 leave then loop 9 ;  main");
        assert!(vm.evaluate().is_ok());
        assert_eq!(vm.s_stack().len(), 2);
        assert_eq!(vm.s_stack().pop2(), Some((88, 9)));
    }

    #[test]
    fn test_do_i_loop() {
        let vm = &mut VM::new(16);
        vm.add_core();
        vm.set_source(": main 3 0 do i loop ;  main");
        assert!(vm.evaluate().is_ok());
        assert_eq!(vm.s_stack().len(), 3);
        assert_eq!(vm.s_stack().pop3(), Some((0, 1, 2)));
    }

    #[test]
    fn test_do_i_j_loop() {
        let vm = &mut VM::new(16);
        vm.add_core();
        vm.set_source(": main 6 4 do 3 1 do i j * loop loop ;  main");
        assert!(vm.evaluate().is_ok());
        assert_eq!(vm.s_stack().len(), 4);
        assert_eq!(vm.s_stack().as_slice(), [4, 8, 5, 10]);
    }
}
