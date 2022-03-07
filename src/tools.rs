use memory::Memory;
use output::Output;
use std::fmt::Write;

pub trait Tools: Output {
    /// Add programming-tools primitives.
    fn add_tools(&mut self) {
        self.add_primitive("words", Tools::words);
        self.add_primitive(".word", Tools::dot_word);
        self.add_primitive(".backtrace", Tools::dot_backtrace);
        self.add_primitive(".s", Tools::dot_s);
        self.add_primitive(".memory", Tools::dot_memory);
        self.add_primitive("(xtime)", Tools::set_execution_times);
        self.add_primitive(".xtime", Tools::dot_xtime);
        self.add_primitive("0xtime", Tools::clear_xtime);
        self.add_primitive(".input", Tools::dot_input);
        self.add_primitive("flush-to-err", Tools::flush_to_err);
    }

    /// Run-time: ( -- )
    ///
    /// Display values on the data stack.
    primitive! {fn dot_s(&mut self) {
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
    primitive! {fn words(&mut self) {
        if let Some(mut buf) = self.output_buffer().take() {
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

    /// Run-time: ( xt -- )
    ///
    /// Print name of execution token xt.
    primitive! {fn dot_word(&mut self) {
        if let Some(mut buf) = self.output_buffer().take() {
            let xt = self.s_stack().pop() as usize;
            if xt < self.wordlist().len() {
                let nfa = self.wordlist()[xt].nfa();
                let name = unsafe{ self.data_space().get_str(nfa) };
                write!(buf, "{} ", name).unwrap();
            } else {
                write!(buf, "unknown ").unwrap();
            }
            self.set_output_buffer(buf);
        }
    }}

    /// Run-time: ( -- )
    ///
    /// Print backtrace.
    primitive! {fn dot_backtrace(&mut self) {
        if let Some(mut buf) = self.output_buffer().take() {
            write!(buf, "backtrace: ").unwrap();
            for i in 0..self.r_stack().len() {
                let ip = self.r_stack()[i] as usize;
                match self.wordlist().find_xt(ip) {
                    Some(xt) => {
                        let nfa = self.wordlist()[xt].nfa();
                        let name = unsafe{ self.data_space().get_str(nfa) };
                        write!(buf, "{} ", name).unwrap();
                    }
                    None => {
                        write!(buf, "unknown({:0X}) ", ip).unwrap();
                    }
                }
            }

            let xt = self.state().aborted_word_pointer;
            if xt != 0 {
                let nfa = self.wordlist()[xt].nfa();
                let name = unsafe{ self.data_space().get_str(nfa) };
                write!(buf, "{} ", name).unwrap();
            }

            let ip = self.state().instruction_pointer;
            match self.wordlist().find_xt(ip) {
                Some(xt) => {
                    let nfa = self.wordlist()[xt].nfa();
                    let name = unsafe{ self.data_space().get_str(nfa) };
                    write!(buf, "{} ", name).unwrap();
                }
                None => {
                    write!(buf, "unknown({:0X}) ", ip).unwrap();
                }
            }

            self.set_output_buffer(buf);
        }
    }}

    /// `.MEMROY ( -- )`
    ///
    /// Print the memory usage information.
    primitive! {fn dot_memory(&mut self) {
        let ds_start = self.data_space().start();
        let ds_limit = self.data_space().limit();
        let ds_here = self.data_space().here();
        let ds_cap = ds_limit - ds_start;
        let ds_used = ds_here - ds_start;
        match self.output_buffer().as_mut() {
            Some(buf) => {
                writeln!(buf, "data space capacity: {}, used: {}, start: 0x{:X}, limit: 0x{:X}, here: 0x{:X}",
                    ds_cap, ds_used, ds_start, ds_limit, ds_here).expect("write data space");
            }
            None => {}
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
    primitive! {fn set_execution_times(&mut self) {
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
    primitive! {fn dot_xtime(&mut self) {
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
    primitive! {fn clear_xtime(&mut self) {
        for w in (1..self.wordlist().len()).rev() {
            if self.wordlist()[w].min_execution_time > 0 {
                self.wordlist_mut()[w].min_execution_time = 0;
                self.wordlist_mut()[w].max_execution_time = 0;
            }
        }
    }}

    /// Print content of the input buffer. `.input ( -- )`
    primitive! {fn dot_input(&mut self) {
        match self.input_buffer().take() {
            Some(input) => {
                match self.output_buffer().as_mut() {
                    Some(out) => {
                        out.push_str(&input);
                    }
                    None => {}
                }
                self.set_input_buffer(input);
            }
            None => {}
        }
    }}

    /// Flush output buffer to standard error output. `flush-to-err ( -- )`
    primitive! {fn flush_to_err(&mut self) {
        match self.output_buffer().as_mut() {
            Some(out) => {
                eprintln!("{}", out);
                out.clear();
            }
            None => {}
        }
    }}
}
