use errors::{S_STACK_UNDERFLOW};

pub trait IO {
    fn emit(&mut self);
}

impl IO for ::core::VM {
    fn emit(&mut self) {
        match self.s_stack.pop() {
            None => self.abort_with_error(S_STACK_UNDERFLOW),
            Some(ch) => self.output_buffer.push(ch as u8 as char)
        }
    }
}
