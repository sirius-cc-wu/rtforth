use core::Core;
use exception::Exception::{
    self
};

pub trait Tools : Core {
    /// Add programming-tools primitives.
    fn add_tools(&mut self) {
        self.add_primitive("words", Tools::words);
        self.add_primitive(".s", Tools::dot_s);
    }

    /// Run-time: ( -- )
    ///
    /// Display values on the data stack.
    fn dot_s(&mut self) -> Option<Exception> {
        println!("TODO: .s");
//        print!("<{}> ", self.s_stack.len());
//        for s in self.s_stack.iter() {
//            print!("{} ", s);
//        }
//        println!("");
        None
    }

    /// Run-time: ( -- )
    ///
    /// List definition names in word list.
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
}
