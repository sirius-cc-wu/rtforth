use std::mem;
use core::{VM, Access, Core};
use core::Heap;
use exception::Exception::{
    self,
    StackUnderflow,
    StackOverflow,
    FloatingPointStackUnderflow
};

/// Types that can output to console.
pub trait Output {
    /// Add output primitives.
    fn add_output(&mut self);

    /// Run-time: ( x -- )
    ///
    /// Put x into output buffer.
    fn emit(&mut self) -> Option<Exception>;

    /// Run-time: ( c-addr u -- )
    ///
    /// Put the character string specified by c-addr and u into output buffer.
    fn p_type(&mut self) -> Option<Exception>;

    /// Runtime of S"
    fn p_s_quote(&mut self) -> Option<Exception>;

    /// Compilation: ( "ccc<quote>" -- )
    ///
    /// Parse ccc delimited by " (double-quote). Append the run-time semantics given below to the
    /// current definition.
    ///
    /// Run-time: ( -- c-addr u )
    ///
    /// Return c-addr and u describing a string consisting of the characters ccc. A program
    /// shall not alter the returned string.
    fn s_quote(&mut self) -> Option<Exception>;

    /// Compilation: ( "ccc<quote>" -- )
    ///
    /// Parse ccc delimited by " (double-quote). Append the run-time semantics given below to the
    /// current definition.
    ///
    /// Run-time: ( -- )
    ///
    /// Display ccc.
    fn dot_quote(&mut self) -> Option<Exception>;

    /// Execution: ( "ccc&lt;paren&gt;" -- )
    ///
    /// Parse and display ccc delimited by ) (right parenthesis). .( is an immediate word.
    fn dot_paren(&mut self) -> Option<Exception>;

    /// Run-time: ( n -- )
    ///
    /// Display n in free field format.
    fn dot(&mut self) -> Option<Exception>;

    /// Run-time: ( -- ) ( F: r -- )
    ///
    /// Display, with a trailing space, the top number on the floating-point stack using fixed-point notation.
    fn fdot(&mut self) -> Option<Exception>;

    fn flush(&mut self) -> Option<Exception>;
}

impl Output for VM {

    fn add_output(&mut self) {
        self.add_primitive("emit", VM::emit);
        self.add_primitive("type", VM::p_type);
        self.add_primitive("_s\"", VM::p_s_quote);
        self.add_immediate("s\"", VM::s_quote);
        self.add_immediate(".\"", VM::dot_quote);
        self.add_immediate(".(", VM::dot_paren);
        self.add_primitive(".", VM::dot);
        self.add_primitive("f.", VM::fdot);
        self.idx_s_quote = self.find("_s\"").expect("_s\" undefined");
        self.idx_type = self.find("type").expect("type undefined");
    }

    fn emit(&mut self) -> Option<Exception> {
        match self.s_stack.pop() {
            None => Some(StackUnderflow),
            Some(ch) => {
                match self.output_buffer {
                  Some(ref mut buffer) => buffer.push(ch as u8 as char),
                  None => {}
                }
                if self.auto_flush {
                    self.flush();
                }
                None
            }
        }
    }

    fn p_type(&mut self) -> Option<Exception> {
        match self.s_stack.pop2() {
            None => Some(StackUnderflow),
            Some((addr, len)) => {
                {
                    let mut output_buffer = self.output_buffer.take().unwrap();
                    {
                        let s = &self.jit_memory().get_str(addr as usize, len as usize);
                        output_buffer.push_str(s);
                    }
                    self.output_buffer = Some(output_buffer);
                }
                if self.auto_flush {
                    self.flush();
                }
                None
            }
        }
    }

    fn p_s_quote(&mut self) -> Option<Exception> {
        let ip = self.instruction_pointer;
        let cnt = self.jit_memory().get_i32(ip);
        let addr = self.instruction_pointer + mem::size_of::<i32>();
        match self.s_stack.push2(addr as isize, cnt as isize) {
            Some(_) => { Some(StackOverflow) },
            None => {
                self.instruction_pointer = self.instruction_pointer + mem::size_of::<i32>() + cnt as usize;
                None
            }
        }
    }

    fn s_quote(&mut self) -> Option<Exception> {
        let input_buffer = self.input_buffer.take().unwrap();
        {
            let source = &input_buffer[self.source_index..input_buffer.len()];
            let (s, cnt)= match source.find('"') {
                Some(n) => (&input_buffer[self.source_index..self.source_index + n], n),
                None => (source, source.len())
            };
            let idx = self.idx_s_quote as i32;
            self.jit_memory().compile_i32(idx);
            self.jit_memory().compile_i32(cnt as i32);
            self.jit_memory().compile_str(s);
            // ignore the space following S"
            self.source_index = self.source_index + 1 + cnt as usize + 1;
        }
        self.input_buffer = Some(input_buffer);
        None
    }

    fn dot_quote(&mut self) -> Option<Exception> {
        self.s_quote();
        let idx_type = self.idx_type;
        self.compile_word(idx_type);
        None
    }

    fn dot_paren(&mut self) -> Option<Exception> {
        let last_token = self.last_token.take().unwrap();
        self.s_stack.push(')' as isize);
        self.parse();
        match self.output_buffer {
          Some(ref mut buffer) => {
            buffer.extend(last_token.chars());
          }
          None => {}
        }
        self.last_token = Some(last_token);
        None
    }

    fn dot(&mut self) -> Option<Exception> {
        match self.s_stack.pop() {
            Some(n) => {
                print!("{} ", n);
                None
            },
            None => Some(StackUnderflow)
        }
    }

    fn fdot(&mut self) -> Option<Exception> {
        match self.f_stack.pop() {
            Some(r) => {
                print!("{} ", r);
                None
            },
            None => Some(FloatingPointStackUnderflow)
        }
    }

    fn flush(&mut self) -> Option<Exception> {
      match self.output_buffer {
        Some(ref mut buffer) => {
          print!("{}", buffer);
          buffer.clear();
        },
        None => {}
      }
      None
    }
}

#[cfg(test)]
mod tests {
    use core::{VM, Core};
    use super::*;

    #[test]
    fn test_s_quote_and_type () {
        let vm = &mut VM::new(16);
        vm.add_core();
        vm.auto_flush = false;
        vm.add_output();
        vm.set_source(": hi   s\" Hi, how are you\" type ; hi");
        assert!(vm.evaluate().is_none());
        assert_eq!(vm.f_stack.as_slice(), []);
        assert_eq!(vm.output_buffer.clone().unwrap(), "Hi, how are you");
    }

    #[test]
    fn test_emit_and_flush () {
        let vm = &mut VM::new(16);
        vm.add_core();
        vm.auto_flush = false;
        vm.add_output();
        vm.set_source("42 emit 43 emit");
        assert!(vm.evaluate().is_none());
        assert_eq!(vm.s_stack.as_slice(), []);
        assert_eq!(vm.output_buffer.clone().unwrap(), "*+");
        assert!(vm.flush().is_none());
        assert_eq!(vm.s_stack.as_slice(), []);
        assert_eq!(vm.output_buffer.clone().unwrap(), "");
    }

}
