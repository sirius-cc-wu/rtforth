use output::Output;
use memory::Memory;
use std::fmt::Write;

pub trait Tools: Output {
    /// Add programming-tools primitives.
    fn add_tools(&mut self) {
        self.add_primitive("words", Tools::words);
        self.add_primitive(".s", Tools::dot_s);
    }

    /// Run-time: ( -- )
    ///
    /// Display values on the data stack.
    primitive!{fn dot_s(&mut self) {
        match self.output_buffer().take() {
            Some(mut buf) => {
                if self.s_stack().len() > 0 {
                    write!(buf, "{:?}", self.s_stack()).expect("write data stack");
                }
                if self.f_stack().len() > 0 {
                    if self.s_stack().len() > 0 {
                        write!(buf, " ").unwrap();
                    }
                    write!(buf, "F: {:?}", self.f_stack()).expect("write floating stack");
                }
                self.set_output_buffer(buf);
            }
            None => {}
        }
        //        write!(buf, "<{}> ", self.s_stack().len()).unwrap();
    }}

    /// Run-time: ( -- )
    ///
    /// List definition names in word list.
    primitive!{fn words(&mut self) {
        if let Some(mut buf) = self.output_buffer().take() {
            writeln!(buf, "").unwrap();
            for w in (1..self.wordlist().len()).rev() {
                if !self.wordlist()[w].is_hidden() {
                    let nfa = self.wordlist()[w].nfa();
                    let name = unsafe{ self.data_space().get_str(nfa) };
                    write!(buf, "{} ", name).unwrap();
                }
            }
            self.set_output_buffer(buf);
        }
    }}
}
