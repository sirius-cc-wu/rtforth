use std::fmt::Write;
use core::Core;
use exception::Exception::{StackUnderflow, FloatingPointStackUnderflow, UnsupportedOperation};

/// Types that can output to console.
pub trait Output: Core {
    /// Add output primitives.
    fn add_output(&mut self) {
        self.add_primitive("emit", Output::emit);
        self.add_primitive("type", Output::p_type);
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
    fn emit(&mut self) {
        match self.s_stack().pop() {
            Err(_) => self.set_error(Some(StackUnderflow)),
            Ok(ch) => {
                match self.output_buffer().take() {
                    Some(mut buffer) => {
                        buffer.push(ch as u8 as char);
                        self.set_output_buffer(buffer);
                    }
                    None => {}
                }
            }
        }
    }

    /// Run-time: ( c-addr u -- )
    ///
    /// Put the character string specified by c-addr and u into output buffer.
    fn p_type(&mut self) {
        match self.s_stack().pop2() {
            Err(_) => self.set_error(Some(StackUnderflow)),
            Ok((addr, len)) => {
                match self.output_buffer().take() {
                    Some(mut buffer) => {
                        {
                            let s = &self.data_space().get_str(addr as usize, len as usize);
                            buffer.push_str(s);
                        }
                        self.set_output_buffer(buffer);
                    }
                    None => {}
                }
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
    fn s_quote(&mut self) {
        let input_buffer = self.input_buffer().take().unwrap();
        {
            let source = &input_buffer[self.state().source_index..input_buffer.len()];
            let (s, cnt) = match source.find('"') {
                Some(n) => {
                    (&input_buffer[self.state().source_index..self.state().source_index + n], n)
                }
                None => (source, source.len()),
            };
            let idx = self.references().idx_s_quote as i32;
            self.data_space().compile_i32(idx);
            self.data_space().compile_i32(cnt as i32);
            self.data_space().compile_str(s);
            // ignore the space following S"
            self.state().source_index = self.state().source_index + 1 + cnt as usize + 1;
        }
        self.set_input_buffer(input_buffer);
    }

    /// Compilation: ( "ccc<quote>" -- )
    ///
    /// Parse ccc delimited by " (double-quote). Append the run-time semantics given below to the
    /// current definition.
    ///
    /// Run-time: ( -- )
    ///
    /// Display ccc.
    fn dot_quote(&mut self) {
        self.s_quote();
        let idx_type = self.references().idx_type;
        self.compile_word(idx_type);
    }

    /// Execution: ( "ccc&lt;paren&gt;" -- )
    ///
    /// Parse and display ccc delimited by ) (right parenthesis). .( is an immediate word.
    fn dot_paren(&mut self) {
        self.s_stack().push(')' as isize);
        self.parse();
        let last_token = self.last_token().take().unwrap();
        if let Some(ref mut buffer) = *self.output_buffer() {
            buffer.extend(last_token.chars());
        }
        self.set_last_token(last_token);
    }

    /// Run-time: ( n -- )
    ///
    /// Display n in free field format.
    fn dot(&mut self) {
        let base_addr = self.data_space().system_variables().base_addr();
        let base = self.data_space().get_isize(base_addr);
        let mut invalid_base = false;
        match self.s_stack().pop() {
            Ok(n) => {
                if let Some(mut buf) = self.output_buffer().take() {
                    match base {
                        2 => {
                            write!(buf, "{:b}", n).unwrap();
                        }
                        8 => {
                            write!(buf, "{:o}", n).unwrap();
                        }
                        10 => {
                            write!(buf, "{} ", n).unwrap();
                        }
                        16 => {
                            write!(buf, "{:X}", n).unwrap();
                        }
                        _ => {
                            invalid_base = true;
                        }
                    }
                    self.set_output_buffer(buf);
                }
                if invalid_base {
                    self.set_error(Some(UnsupportedOperation));
                }
            }
            Err(_) => self.set_error(Some(StackUnderflow)),
        }
    }

    /// Run-time: ( -- ) ( F: r -- )
    ///
    /// Display, with a trailing space, the top number on the floating-point
    /// stack using fixed-point notation.
    fn fdot(&mut self) {
        match self.f_stack().pop() {
            Ok(r) => {
                if let Some(mut buf) = self.output_buffer().take() {
                    write!(buf, "{} ", r).unwrap();
                    self.set_output_buffer(buf);
                }
            }
            Err(_) => self.set_error(Some(FloatingPointStackUnderflow)),
        }
    }
}

#[cfg(test)]
mod tests {
    use vm::VM;
    use core::Core;

    #[test]
    fn test_s_quote_and_type() {
        let vm = &mut VM::new(16);
        vm.set_source(": hi   s\" Hi, how are you\" type ; hi");
        vm.evaluate();
        assert_eq!(vm.last_error(), None);
        assert_eq!(vm.f_stack().as_slice(), []);
        assert_eq!(vm.output_buffer().clone().unwrap(), "Hi, how are you");
    }

    #[test]
    fn test_emit() {
        let vm = &mut VM::new(16);
        vm.set_source("42 emit 43 emit");
        vm.evaluate();
        assert_eq!(vm.last_error(), None);
        assert_eq!(vm.s_stack().as_slice(), []);
        assert_eq!(vm.output_buffer().clone().unwrap(), "*+");
    }
}
