use core::VM;
use std::ops::BitAnd;
use std::ops::Shr;

extern crate time;

pub trait Facility {
    /// Stack effect: ( --  )
    ///
    /// Add facility primitives.
    fn add_facility(&mut self);

    /// Stack effect: ( -- ud )
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
}

impl Facility for VM {
    fn add_facility(&mut self) {
        self.add_primitive("ntime", VM::ntime);
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
}
