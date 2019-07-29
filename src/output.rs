use core::Core;
use memory::Memory;
use exception::Exception::{StackUnderflow, UnsupportedOperation};
use std::fmt::Write;

/// Types that can output to console.
pub trait Output: Core {
    /// Add output primitives.
    fn add_output(&mut self) {
        self.add_primitive("emit", Output::emit);
        self.add_primitive("type", Output::p_type);
        self.add_immediate_and_compile_only("s\"", Output::s_quote);
        self.add_immediate_and_compile_only(".\"", Output::dot_quote);
        self.add_immediate(".(", Output::dot_paren);
        self.add_primitive(".r", Output::dot_r);
        self.add_primitive("f.r", Output::fdot_r);
        self.add_primitive("flush-output", Output::flush_output);
        self.references().idx_s_quote = self.find("_s\"").expect("_s\" undefined");
        self.references().idx_type = self.find("type").expect("type undefined");
    }

    fn push_output(&mut self, text: &str) {
        match self.output_buffer().take() {
            Some(mut buffer) => {
                buffer.push_str(text);
                self.set_output_buffer(buffer);
            }
            None => {}
        }
    }

    /// Run-time: ( x -- )
    ///
    /// Put x into output buffer.
    primitive!{fn emit(&mut self) {
        let ch = self.s_stack().pop();
        match self.output_buffer().take() {
            Some(mut buffer) => {
                buffer.push(ch as u8 as char);
                self.set_output_buffer(buffer);
            }
            None => {}
        }
    }}

    /// Run-time: ( c-addr u -- )
    ///
    /// Put the character string specified by c-addr and u into output buffer.
    primitive!{fn p_type(&mut self) {
        let (addr, len) = self.s_stack().pop2();
        if self.s_stack().underflow() {
            self.abort_with(StackUnderflow);
            return;
        }
        match self.output_buffer().take() {
            Some(mut buffer) => {
                {
                    let s = unsafe{
                        &self.data_space().str_from_raw_parts(
                            addr as usize, len as usize
                        )
                    };
                    buffer.push_str(s);
                }
                self.set_output_buffer(buffer);
            }
            None => {}
        }
    }}

    /// Compilation: ( "ccc<quote>" -- )
    ///
    /// Parse ccc delimited by " (double-quote). Append the run-time semantics given below to the
    /// current definition.
    ///
    /// Run-time: ( -- c-addr u )
    ///
    /// Return c-addr and u describing a string consisting of the characters ccc. A program
    /// shall not alter the returned string.
    primitive!{fn s_quote(&mut self) {
        let input_buffer = self.input_buffer().take().unwrap();
        {
            let source = &input_buffer[self.state().source_index+1..input_buffer.len()];
            let s = match source.find('"') {
                Some(n) => {
                    &input_buffer[
                        self.state().source_index+1..
                        self.state().source_index + 1 + n
                    ]
                }
                None => source,
            };
            let cnt = s.len();
            let idx = self.references().idx_s_quote;
            let compilation_semantics = self.wordlist()[idx].compilation_semantics;
            compilation_semantics(self, idx);
            self.data_space().compile_str(s);
            self.data_space().align();
            // ignore the space following S"
            self.state().source_index = self.state().source_index + 1 + cnt as usize + 1;
        }
        self.set_input_buffer(input_buffer);
    }}

    /// Compilation: ( "ccc<quote>" -- )
    ///
    /// Parse ccc delimited by " (double-quote). Append the run-time semantics given below to the
    /// current definition.
    ///
    /// Run-time: ( -- )
    ///
    /// Display ccc.
    primitive!{fn dot_quote(&mut self) {
        self.s_quote();
        let idx_type = self.references().idx_type;
        self.compile_word(idx_type);
    }}

    /// Execution: ( "ccc&lt;paren&gt;" -- )
    ///
    /// Parse and display ccc delimited by ) (right parenthesis). .( is an immediate word.
    primitive!{fn dot_paren(&mut self) {
        self.s_stack().push(')' as isize);
        self.parse();
        let last_token = self.last_token().take().unwrap();
        if let Some(ref mut buffer) = *self.output_buffer() {
            buffer.extend(last_token.chars());
        }
        self.set_last_token(last_token);
    }}

    /// Run-time: ( n1 n2 -- )
    ///
    /// Display `n1` right aligned in a field `n2` characters wide.
    primitive!{fn dot_r(&mut self) {
        let base_addr = self.data_space().system_variables().base_addr();
        let base = unsafe{ self.data_space().get_isize(base_addr) };
        let mut valid_base = true;
        let (n1, n2) = self.s_stack().pop2();
        if let Some(mut buf) = self.output_buffer().take() {
            self.hold_buffer().clear();
            match base {
                2 => {
                    write!(self.hold_buffer(), "{:b}", n1).unwrap();
                }
                8 => {
                    write!(self.hold_buffer(), "{:o}", n1).unwrap();
                }
                10 => {
                    write!(self.hold_buffer(), "{}", n1).unwrap();
                }
                16 => {
                    write!(self.hold_buffer(), "{:X}", n1).unwrap();
                }
                _ => {
                    valid_base = false;
                }
            }
            if valid_base {
                for _ in 0..(n2 - self.hold_buffer().len() as isize) {
                    buf.push(' ');
                }
                buf.push_str(self.hold_buffer());
            }
            self.set_output_buffer(buf);
        }
        if !valid_base {
            self.abort_with(UnsupportedOperation);
        }
    }}

    /// Run-time: ( n1 n2 -- ) ( F: r -- )
    ///
    /// Display, without a trailing space, the top number on the floating-point
    /// stack in a field `n1` characters wide, with `n2` digits after the decimal point.
    ///
    /// If n2 greater than 17, only 17 digits after the decimal point is printed.
    primitive!{fn fdot_r(&mut self) {
        let r = self.f_stack().pop();
        let (n1, n2) = self.s_stack().pop2();
        if let Some(mut buf) = self.output_buffer().take() {
            self.hold_buffer().clear();
            match n2 {
                0 => write!(self.hold_buffer(), "{:.0}", r).unwrap(),
                1 => write!(self.hold_buffer(), "{:.1}", r).unwrap(),
                2 => write!(self.hold_buffer(), "{:.2}", r).unwrap(),
                3 => write!(self.hold_buffer(), "{:.3}", r).unwrap(),
                4 => write!(self.hold_buffer(), "{:.4}", r).unwrap(),
                5 => write!(self.hold_buffer(), "{:.5}", r).unwrap(),
                6 => write!(self.hold_buffer(), "{:.6}", r).unwrap(),
                7 => write!(self.hold_buffer(), "{:.7}", r).unwrap(),
                8 => write!(self.hold_buffer(), "{:.8}", r).unwrap(),
                9 => write!(self.hold_buffer(), "{:.9}", r).unwrap(),
                10 => write!(self.hold_buffer(), "{:.10}", r).unwrap(),
                11 => write!(self.hold_buffer(), "{:.11}", r).unwrap(),
                12 => write!(self.hold_buffer(), "{:.12}", r).unwrap(),
                13 => write!(self.hold_buffer(), "{:.13}", r).unwrap(),
                14 => write!(self.hold_buffer(), "{:.14}", r).unwrap(),
                15 => write!(self.hold_buffer(), "{:.15}", r).unwrap(),
                16 => write!(self.hold_buffer(), "{:.16}", r).unwrap(),
                _ => write!(self.hold_buffer(), "{:.17}", r).unwrap(),
            }
            for _ in 0..(n1 - self.hold_buffer().len() as isize) {
                buf.push(' ');
            }
            buf.push_str(self.hold_buffer());
            self.set_output_buffer(buf);
        }
    }}

    primitive!{fn flush_output(&mut self) {
        match self.output_buffer().as_mut() {
            Some(buf) => {
                if buf.len() > 0 {
                    println!("{}", buf);
                    buf.clear();
                }
            }
            None => {}
        }
    }}

}

#[cfg(test)]
mod tests {
    use core::Core;
    use mock_vm::VM;

    #[test]
    fn test_s_quote_and_type() {
        let vm = &mut VM::new(16, 16);
        vm.set_source(": hi   s\" Hi, how are you\" type ; hi");
        vm.evaluate_input();
        assert_eq!(vm.last_error(), None);
        assert_eq!(vm.f_stack().as_slice(), []);
        assert_eq!(vm.output_buffer().clone().unwrap(), "Hi, how are you");
    }

    #[test]
    fn test_emit() {
        let vm = &mut VM::new(16, 16);
        vm.set_source("42 emit 43 emit");
        vm.evaluate_input();
        assert_eq!(vm.last_error(), None);
        assert_eq!(vm.s_stack().as_slice(), []);
        assert_eq!(vm.output_buffer().clone().unwrap(), "*+");
    }
}
