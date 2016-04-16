use std::fmt::Write;
use output::Output;
use exception::Exception::{
    self
};

pub trait Tools : Output {
    /// Add programming-tools primitives.
    fn add_tools(&mut self) {
        self.add_primitive("words", Tools::words);
        self.add_primitive(".s", Tools::dot_s);
    }

    /// Run-time: ( -- )
    ///
    /// Display values on the data stack.
    fn dot_s(&mut self) -> Option<Exception> {
        let mut buf = self.output_buffer().take().unwrap();
        write!(buf, "TODO: .s");
//        write!(buf, "<{}> ", self.s_stack().len());
//        for s in self.s_stack().iter() {
//            write!(buf, "{} ", s);
//        }
        self.set_output_buffer(buf);
        if self.state().auto_flush {
          self.flush();
        }
        None
    }

    /// Run-time: ( -- )
    ///
    /// List definition names in word list.
    fn words(&mut self) -> Option<Exception> {
        let mut buf = self.output_buffer().take().unwrap();
        writeln!(buf, "");
        let mut link = self.jit_memory_const().last();
        while !(link == 0) {
            let w = self.jit_memory_const().word(link);
            link = w.link();
            if !w.is_hidden() {
                write!(buf, "{} ", self.jit_memory_const().name(w) );
            }
        }
        self.set_output_buffer(buf);
        if self.state().auto_flush {
          self.flush();
        }
        None
    }
}
