use core::VM;
use std::ops::BitAnd;
use std::ops::Shr;

extern crate time;

pub trait Facility {
    /// Run-time: ( --  )
    ///
    /// Add facility primitives.
    fn add_facility(&mut self);

    /// Run-time: ( -- ud )
    ///
    /// Current time in nanoseconds since some epoch
    ///
    /// Examples:
    ///
    /// ```
    /// use rtforth::core::VM;
    /// use rtforth::facility::Facility;
    /// use rtforth::tools::Tools;
    /// let mut vm = VM::new();
    /// vm.add_facility();
    /// vm.add_tools();
    /// vm.set_source("ntime .s");
    /// vm.evaluate();
    /// ```
    fn ntime(&mut self);

    /// Run-time: ( -- ud )
    ///
    /// Current time in microseconds since some epoch
    ///
    /// Examples:
    ///
    /// ```
    /// use rtforth::core::VM;
    /// use rtforth::facility::Facility;
    /// use rtforth::tools::Tools;
    /// let mut vm = VM::new();
    /// vm.add_facility();
    /// vm.add_tools();
    /// vm.set_source("utime .s");
    /// vm.evaluate();
    /// ```
    fn utime(&mut self);
}

impl Facility for VM {
    fn add_facility(&mut self) {
        self.add_primitive("ntime", VM::ntime);
        self.add_primitive("utime", VM::utime);
    }

    fn ntime(&mut self) {
        let t = time::precise_time_ns();
        if t > usize::max_value() as u64 {
            self.s_stack.push(t.bitand(usize::max_value() as u64) as isize);
            self.s_stack.push(t.shr(usize::max_value().count_ones()) as isize);
        } else {
            self.s_stack.push(t as isize);
            self.s_stack.push(0);
        }
    }

    fn utime(&mut self) {
        let t = time::precise_time_ns()/1000;
        if t > usize::max_value() as u64 {
            self.s_stack.push(t.bitand(usize::max_value() as u64) as isize);
            self.s_stack.push(t.shr(usize::max_value().count_ones()) as isize);
        } else {
            self.s_stack.push(t as isize);
            self.s_stack.push(0);
        }
    }
}

