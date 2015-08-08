use core::VM;
use exception::Exception::{StackUnderflow};

/// Types that can output to console.
pub trait Output {
    /// Replace self's output primitives with those in this trait.
    fn patch_output_primitives(&mut self);

    /// Put x into output buffer.
    /// 
    /// Stack effect: ( x -- )
    fn emit(&mut self);

    /// Put the character string specified by c-addr and u into output buffer. 
    ///
    /// Stack effect: ( c-addr u -- )
    fn p_type(&mut self);

    /// Display content of output buffer, empty output buffer.
    ///
    /// Stack effect: ( -- )
    fn flush(&mut self);
}

impl Output for VM {

    fn patch_output_primitives(&mut self) {
        self.patch_primitive ("emit", VM::emit);
        self.patch_primitive ("type", VM::p_type);
        self.patch_primitive ("flush", VM::flush);
    }

    fn emit(&mut self) {
        match self.s_stack.pop() {
            None => self.abort_with_error(StackUnderflow.name()),
            Some(ch) => self.output_buffer.push(ch as u8 as char)
        }
    }

    fn p_type(&mut self) {
        match self.s_stack.pop() {
            None => self.abort_with_error(StackUnderflow.name()),
            Some(icnt) => match self.s_stack.pop() {
                None => self.abort_with_error(StackUnderflow.name()),
                Some(iaddr) => {
                    let cnt = icnt as usize;
                    let addr = iaddr as usize;
                    let s = &self.n_heap[addr..addr+cnt]; 
                    self.output_buffer.push_str(s);
                }
            }
        }
    }

    fn flush(&mut self) {
        println!("{}", self.output_buffer);
        self.output_buffer.clear();
    }

}
