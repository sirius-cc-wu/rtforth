use core::VM;
use core::Heap;
use exception::Exception::{
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
    fn emit(&mut self);

    /// Run-time: ( c-addr u -- )
    ///
    /// Put the character string specified by c-addr and u into output buffer. 
    fn p_type(&mut self);

    /// Compilation: ( "ccc<quote>" -- )
    ///
    /// Parse ccc delimited by " (double-quote). Append the run-time semantics given below to the
    /// current definition.
    ///
    /// Run-time: ( -- c-addr u )
    ///
    /// Return c-addr and u describing a string consisting of the characters ccc. A program
    /// shall not alter the returned string. 
    fn s_quote(&mut self);

    /// Compilation: ( "ccc<quote>" -- )
    ///
    /// Parse ccc delimited by " (double-quote). Append the run-time semantics given below to the
    /// current definition.
    ///
    /// Run-time: ( -- )
    ///
    /// Display ccc. 
    fn dot_quote(&mut self);

    /// Execution: ( "ccc&lt;paren&gt;" -- )
    ///
    /// Parse and display ccc delimited by ) (right parenthesis). .( is an immediate word.
    fn dot_paren(&mut self);

    /// Run-time: ( n -- )
    ///
    /// Display n in free field format. 
    fn dot(&mut self);

    /// Run-time: ( -- ) ( F: r -- )
    ///
    /// Display, with a trailing space, the top number on the floating-point stack using fixed-point notation.
    fn fdot(&mut self);

    fn flush(&mut self);
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
        self.idx_type = self.find("type").expect("type defined");
    }

    fn emit(&mut self) {
        match self.s_stack.pop() {
            None => self.abort_with_error(StackUnderflow),
            Some(ch) => self.output_buffer.push(ch as u8 as char)
        }
        if self.auto_flush {
            self.flush();
        }
    }

    fn p_type(&mut self) {
        match self.s_stack.pop() {
            None => self.abort_with_error(StackUnderflow),
            Some(icnt) => match self.s_stack.pop() {
                None => self.abort_with_error(StackUnderflow),
                Some(iaddr) => {
                    let cnt = icnt as usize;
                    let addr = iaddr as usize;
                    let s = &self.n_heap[addr..addr+cnt]; 
                    self.output_buffer.push_str(s);
                }
            }
        }
        if self.auto_flush {
            self.flush();
        }
    }

    fn s_quote(&mut self) {
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
    }

    fn dot_quote(&mut self) {
        self.s_quote();
        let idx_type = self.idx_type;
        self.compile_word(idx_type);
    }

    fn dot_paren(&mut self) {
        self.s_stack.push(')' as isize);
        self.parse();
        self.output_buffer.extend(self.last_token.chars());
    }

    fn dot(&mut self) {
        match self.s_stack.pop() {
            Some(n) => print!("{} ", n),
            None => self.abort_with_error(StackUnderflow)
        }
    }

    fn fdot(&mut self) {
        match self.f_stack.pop() {
            Some(r) => print!("{} ", r),
            None => self.abort_with_error(FloatingPointStackUnderflow)
        }
    }

    fn flush(&mut self) {
        print!("{}", self.output_buffer);
        self.output_buffer.clear();
    }
}

#[cfg(test)]
mod tests {
    use core::VM;
    use super::*;

    #[test]
    fn test_s_quote_and_type () {
        let vm = &mut VM::new();
        vm.auto_flush = false;
        vm.add_output();
        vm.set_source(": hi   s\" Hi, how are you\" type ; hi");
        vm.evaluate();
        assert_eq!(vm.f_stack.as_slice(), []);
        assert_eq!(vm.output_buffer, "Hi, how are you");
        assert_eq!(vm.error_code, 0);
    }

    #[test]
    fn test_emit_and_flush () {
        let vm = &mut VM::new();
        vm.auto_flush = false;
        vm.add_output();
        vm.set_source("42 emit 43 emit");
        vm.evaluate();
        assert_eq!(vm.s_stack.as_slice(), []);
        assert_eq!(vm.output_buffer, "*+");
        assert_eq!(vm.error_code, 0);
        vm.flush();
        assert_eq!(vm.s_stack.as_slice(), []);
        assert_eq!(vm.output_buffer, "");
        assert_eq!(vm.error_code, 0);
    }

}
