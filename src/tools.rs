use core::VM;

pub trait Tools {
    /// Patch self's programming-tools primitives.
    fn patch_tools(&mut self);

    /// Display values on the data stack.
    ///
    /// Stack effect: ( -- )
    fn dot_s(&mut self);

    /// List definition names in word list.
    ///
    /// Stack effect: ( -- )
    fn words(&mut self);
}

impl Tools for ::core::VM {
    fn patch_tools(&mut self) {
        self.patch_primitive("words", VM::words);
        self.patch_primitive(".s", VM::dot_s);
    }

    fn words(&mut self) {
        for w in self.word_list.iter() {
            let s = &self.n_heap[w.nfa..w.nfa+w.name_len];
            println!("{}", s );
        }
    }

    fn dot_s(&mut self) {
        println!("<{}>", self.s_stack.len());
        for s in self.s_stack.iter() {
            println!("{}", s);
        }
    }
}
