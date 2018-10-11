use output::Output;
use memory::Memory;
use std::fmt::Write;

pub trait Tools: Output {
    /// Add programming-tools primitives.
    fn add_tools(&mut self) {
        self.add_primitive("words", Tools::words);
        self.add_primitive(".s", Tools::dot_s);
        self.add_primitive("(xtime)", Tools::set_execution_times);
        self.add_primitive(".xtime", Tools::dot_xtime);
        self.add_primitive("0xtime", Tools::clear_xtime);
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

    /// Update execution time of word `xt`. `(xtime) ( t0 xt -- )`
    ///
    /// Unit of `t0` is microseconds.
    ///
    /// Example:
    ///
    /// Meaure the execution time of `words`.
    /// ```forth
    /// : xtime ( t0 xt -- )   2>r r@  execute  2r>  (xtime) ;
    /// utime ' words xtime .xtime
    /// ```
    primitive!{fn set_execution_times(&mut self) {
        let (t0, xt) = self.s_stack().pop2();
        let t = (self.system_time_ns()/1_000) as usize - t0 as usize;
        let word = &mut self.wordlist_mut()[xt as usize];
        if word.min_execution_time != 0 {
            word.min_execution_time = word.min_execution_time.min(t);
        } else {
            word.min_execution_time = t;
        }
        word.max_execution_time = word.max_execution_time.max(t);
    }}

    /// Display measured execution time. `.xtime ( -- )`
    primitive!{fn dot_xtime(&mut self) {
        if let Some(mut buf) = self.output_buffer().take() {
            let mut counter = 0;
            for w in (1..self.wordlist().len()).rev() {
                if self.wordlist()[w].min_execution_time > 0 {
                    counter += 1;
                    if 1 != counter {
                        write!(buf, "|").unwrap();
                    }
                    let min_t = self.wordlist()[w].min_execution_time;
                    let max_t = self.wordlist()[w].max_execution_time;
                    let nfa = self.wordlist()[w].nfa();
                    let name = unsafe{ self.data_space().get_str(nfa) };
                    write!(buf, "{}|{},{}", name, min_t, max_t).unwrap();
                }
            }
            self.set_output_buffer(buf);
        }
    }}

    /// Clear measured execution times. `0xtime( -- )`
    primitive!{fn clear_xtime(&mut self) {
        for w in (1..self.wordlist().len()).rev() {
            if self.wordlist()[w].min_execution_time > 0 {
                self.wordlist_mut()[w].min_execution_time = 0;
                self.wordlist_mut()[w].max_execution_time = 0;
            }
        }
    }}

}
