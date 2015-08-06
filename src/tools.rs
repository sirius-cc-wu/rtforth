pub trait Tools {
    fn words(&mut self);
    fn dot_s(&mut self);
}

impl Tools for ::core::VM {
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
