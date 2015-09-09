use core::VM;
use core::Heap;
use exception::Exception::{
    self,
    StackUnderflow,
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
        self.add_primitive ("emit", VM::emit);
        self.add_primitive ("type", VM::p_type);
        self.add_immediate ("s\"", VM::s_quote);
        self.add_immediate (".\"", VM::dot_quote);
        self.add_immediate (".(", VM::dot_paren);
        self.add_primitive (".", VM::dot);
        self.add_primitive ("f.", VM::fdot);
        self.idx_type = self.find("type").expect("type undefined");
    }

    fn emit(&mut self) -> Option<Exception> {
        match self.s_stack.pop() {
            None => Some(StackUnderflow),
            Some(ch) => {
                self.output_buffer.push(ch as u8 as char);
                if self.auto_flush {
                    self.flush();
                }
                None
            }
        }
    }

    fn p_type(&mut self) -> Option<Exception> {
        match self.s_stack.pop() {
            None => Some(StackUnderflow),
            Some(icnt) => match self.s_stack.pop() {
                None => Some(StackUnderflow),
                Some(iaddr) => {
                    let cnt = icnt as usize;
                    let addr = iaddr as usize;
                    {
                        let s = &self.n_heap[addr..addr+cnt]; 
                        self.output_buffer.push_str(s);
                    }
                    if self.auto_flush {
                        self.flush();
                    }
                    None
                }
            }
        }
    }

    fn s_quote(&mut self) -> Option<Exception> {
        // ignore the space following S"
        let source = &self.input_buffer[self.source_index..self.input_buffer.len()];
        let naddr = self.n_heap.len();
        let mut cnt = 0;
        for ch in source.chars() {
            if ch == '"' {
                break;
            } else {
                self.n_heap.push(ch);
            }
            cnt = cnt + 1;
        }
        self.s_heap.push_i32(self.idx_lit as i32);
        self.s_heap.push_i32(naddr as i32);
        self.s_heap.push_i32(self.idx_lit as i32);
        self.s_heap.push_i32(cnt);
        self.source_index = self.source_index + 1 + cnt as usize + 1;
        None
    }

    fn dot_quote(&mut self) -> Option<Exception> {
        self.s_quote();
        let idx_type = self.idx_type;
        self.compile_word(idx_type);
        None
    }

    fn dot_paren(&mut self) -> Option<Exception> {
        self.s_stack.push(')' as isize);
        self.parse();
        self.output_buffer.extend(self.last_token.chars());
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
        print!("{}", self.output_buffer);
        self.output_buffer.clear();
        None
    }
}

#[cfg(test)]
mod tests {
    use core::VM;
    use super::*;

    #[test]
    fn test_s_quote_and_type () {
        let vm = &mut VM::new(1024);
        vm.auto_flush = false;
        vm.add_output();
        vm.set_source(": hi   s\" Hi, how are you\" type ; hi");
        vm.evaluate();
        assert_eq!(vm.f_stack.as_slice(), []);
        assert_eq!(vm.output_buffer, "Hi, how are you");
        assert!(!vm.has_error());
    }

    #[test]
    fn test_emit_and_flush () {
        let vm = &mut VM::new(1024);
        vm.auto_flush = false;
        vm.add_output();
        vm.set_source("42 emit 43 emit");
        vm.evaluate();
        assert_eq!(vm.s_stack.as_slice(), []);
        assert_eq!(vm.output_buffer, "*+");
        assert!(!vm.has_error());
        vm.flush();
        assert_eq!(vm.s_stack.as_slice(), []);
        assert_eq!(vm.output_buffer, "");
        assert!(!vm.has_error());
    }

}
