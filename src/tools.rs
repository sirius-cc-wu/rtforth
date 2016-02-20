use core::{VM, Core};
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
        self.add_primitive("words", Tools::words);
        self.add_primitive(".s", Tools::dot_s);
    }

    fn words(&mut self) -> Option<Exception> {
        println!("");
        let mut link = self.jit_memory_const().last();
        while !(link == 0) {
            let w = self.jit_memory_const().word(link);
            link = w.link;
            if !w.hidden {
                print!("{} ", self.jit_memory_const().name(w) );
            }
        }
        None
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
