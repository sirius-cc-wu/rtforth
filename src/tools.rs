use core::VM;
use exception::Exception::{
    self
};

pub trait Tools {
    /// Add programming-tools primitives.
    fn add_tools(&mut self);

    /// Run-time: ( -- )
    ///
    /// Display values on the data stack.
    fn dot_s(&mut self) -> Option<Exception>;

    /// Run-time: ( -- )
    ///
    /// List definition names in word list.
    fn words(&mut self) -> Option<Exception>;

}

impl Tools for VM {
    fn add_tools(&mut self) {
        self.add_primitive("words", VM::words);
        self.add_primitive(".s", VM::dot_s);
    }

    fn words(&mut self) -> Option<Exception> {
        println!("");
        let mut w = self.jit_memory.word(self.jit_memory.last());
        loop {
            if !w.hidden {
                print!("{} ", w.name );
            }
            if w.link != 0 {
                w = self.jit_memory.word(w.link);
            } else {
                return None
            }
        }
    }

    fn dot_s(&mut self) -> Option<Exception> {
        println!("TODO: .s");
//        print!("<{}> ", self.s_stack.len());
//        for s in self.s_stack.iter() {
//            print!("{} ", s);
//        }
//        println!("");
        None
    }

}
