use std::mem;
use std::fmt::Write;
use core::Core;
use core::Heap;
use exception::Exception::{
    self,
    StackUnderflow,
    StackOverflow,
    FloatingPointStackUnderflow
};

/// Types that can output to console.
pub trait Output : Core {
    /// Add output primitives.
    fn add_output(&mut self) {
        self.add_primitive("emit", Output::emit);
        self.add_primitive("type", Output::p_type);
        self.add_primitive("_s\"", Output::p_s_quote);
        self.add_immediate("s\"", Output::s_quote);
        self.add_immediate(".\"", Output::dot_quote);
        self.add_immediate(".(", Output::dot_paren);
        self.add_primitive(".", Output::dot);
        self.add_primitive("f.", Output::fdot);
        self.references().idx_s_quote = self.find("_s\"").expect("_s\" undefined");
        self.references().idx_type = self.find("type").expect("type undefined");
    }

    /// Run-time: ( x -- )
    ///
    /// Put x into output buffer.
    fn emit(&mut self) -> Option<Exception> {
        match self.s_stack().pop() {
            None => Some(StackUnderflow),
            Some(ch) => {
                if let Some(ref mut buffer) = *self.output_buffer() {
                    buffer.push(ch as u8 as char)
                }
                if self.state().auto_flush {
                    self.flush();
                }
                None
            }
        }
    }

    /// Run-time: ( c-addr u -- )
    ///
    /// Put the character string specified by c-addr and u into output buffer.
    fn p_type(&mut self) -> Option<Exception> {
        match self.s_stack().pop2() {
            None => Some(StackUnderflow),
            Some((addr, len)) => {
                {
                    let mut output_buffer = self.output_buffer().take().unwrap();
                    {
                        let s = &self.jit_memory().get_str(addr as usize, len as usize);
                        output_buffer.push_str(s);
                    }
                    self.set_output_buffer(output_buffer);
                }
                if self.state().auto_flush {
                    self.flush();
                }
                None
            }
        }
    }

    /// Runtime of S"
    fn p_s_quote(&mut self) -> Option<Exception> {
        let ip = self.state().instruction_pointer;
        let cnt = self.jit_memory().get_i32(ip);
        let addr = self.state().instruction_pointer + mem::size_of::<i32>();
        match self.s_stack().push2(addr as isize, cnt as isize) {
            Some(_) => { Some(StackOverflow) },
            None => {
                self.state().instruction_pointer = self.state().instruction_pointer + mem::size_of::<i32>() + cnt as usize;
                None
            }
        }
    }

    /// Compilation: ( "ccc<quote>" -- )
    ///
    /// Parse ccc delimited by " (double-quote). Append the run-time semantics given below to the
    /// current definition.
    ///
    /// Run-time: ( -- c-addr u )
    ///
    /// Return c-addr and u describing a string consisting of the characters ccc. A program
    /// shall not alter the returned string.
    fn s_quote(&mut self) -> Option<Exception> {
        let input_buffer = self.input_buffer().take().unwrap();
        {
            let source = &input_buffer[self.state().source_index..input_buffer.len()];
            let (s, cnt)= match source.find('"') {
                Some(n) => (&input_buffer[self.state().source_index..self.state().source_index + n], n),
                None => (source, source.len())
            };
            let idx = self.references().idx_s_quote as i32;
            self.jit_memory().compile_i32(idx);
            self.jit_memory().compile_i32(cnt as i32);
            self.jit_memory().compile_str(s);
            // ignore the space following S"
            self.state().source_index = self.state().source_index + 1 + cnt as usize + 1;
        }
        self.set_input_buffer(input_buffer);
        None
    }

    /// Compilation: ( "ccc<quote>" -- )
    ///
    /// Parse ccc delimited by " (double-quote). Append the run-time semantics given below to the
    /// current definition.
    ///
    /// Run-time: ( -- )
    ///
    /// Display ccc.
    fn dot_quote(&mut self) -> Option<Exception> {
        self.s_quote();
        let idx_type = self.references().idx_type;
        self.compile_word(idx_type);
        None
    }

    /// Execution: ( "ccc&lt;paren&gt;" -- )
    ///
    /// Parse and display ccc delimited by ) (right parenthesis). .( is an immediate word.
    fn dot_paren(&mut self) -> Option<Exception> {
        let last_token = self.last_token().take().unwrap();
        self.s_stack().push(')' as isize);
        self.parse();
        if let Some(ref mut buffer) = *self.output_buffer() {
            buffer.extend(last_token.chars());
        }
        self.set_last_token(last_token);
        None
    }

    /// Run-time: ( n -- )
    ///
    /// Display n in free field format.
    fn dot(&mut self) -> Option<Exception> {
        match self.s_stack().pop() {
            Some(n) => {
                if let Some(mut buf) = self.output_buffer().take() {
                  write!(buf, "{} ", n);
                  self.set_output_buffer(buf);
                  if self.state().auto_flush { self.flush(); }
                }
                None
            },
            None => Some(StackUnderflow)
        }
    }

    /// Run-time: ( -- ) ( F: r -- )
    ///
    /// Display, with a trailing space, the top number on the floating-point stack using fixed-point notation.
    fn fdot(&mut self) -> Option<Exception> {
        match self.f_stack().pop() {
            Some(r) => {
              if let Some(mut buf) = self.output_buffer().take() {
                write!(buf, "{} ", r);
                self.set_output_buffer(buf);
                if self.state().auto_flush { self.flush(); }
              }
              None
            },
            None => Some(FloatingPointStackUnderflow)
        }
    }

    fn flush(&mut self) -> Option<Exception> {
      if let Some(ref mut buffer) = *self.output_buffer() {
        print!("{}", buffer);
        buffer.clear();
      }
      None
    }
}

#[cfg(test)]
mod tests {
    use vm::VM;
    use core::Core;
    use super::*;

    #[test]
    fn test_s_quote_and_type () {
        let vm = &mut VM::new(16);
        vm.add_core();
        vm.state().auto_flush = false;
        vm.add_output();
        vm.set_source(": hi   s\" Hi, how are you\" type ; hi");
        assert!(vm.evaluate().is_none());
        assert_eq!(vm.f_stack().as_slice(), []);
        assert_eq!(vm.output_buffer().clone().unwrap(), "Hi, how are you");
    }

    #[test]
    fn test_emit_and_flush () {
        let vm = &mut VM::new(16);
        vm.add_core();
        vm.state().auto_flush = false;
        vm.add_output();
        vm.set_source("42 emit 43 emit");
        assert!(vm.evaluate().is_none());
        assert_eq!(vm.s_stack().as_slice(), []);
        assert_eq!(vm.output_buffer().clone().unwrap(), "*+");
        assert!(vm.flush().is_none());
        assert_eq!(vm.s_stack().as_slice(), []);
        assert_eq!(vm.output_buffer().clone().unwrap(), "");
    }

}
