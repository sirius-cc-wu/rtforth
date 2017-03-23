use core::{Core, Result};
use std::ops::BitAnd;
use std::ops::Shr;
use exception::Exception::StackOverflow;

extern crate time;

pub trait Facility: Core {
    /// Run-time: ( --  )
    ///
    /// Add facility primitives.
    fn add_facility(&mut self) {
        self.add_primitive("ntime", Facility::ntime);
        self.add_primitive("utime", Facility::utime);
    }

    /// Run-time: ( -- ud )
    ///
    /// Current time in nanoseconds since some epoch
    ///
    /// Examples:
    ///
    /// ```
    /// use rtforth::vm::VM;
    /// use rtforth::core::Core;
    /// use rtforth::facility::Facility;
    /// use rtforth::tools::Tools;
    /// let vm = &mut VM::new(16);
    /// vm.add_core();
    /// vm.add_facility();
    /// vm.add_tools();
    /// vm.set_source("ntime .s");
    /// vm.evaluate();
    /// ```
    fn ntime(&mut self) -> Result {
        let t = time::precise_time_ns();
        if t > usize::max_value() as u64 {
            match self.s_stack().push2(t.bitand(usize::max_value() as u64) as isize,
                                       t.shr(usize::max_value().count_ones()) as isize) {
                Err(_) => self.set_error(Some(StackOverflow)),
                Ok(()) => {}
            }
        } else {
            match self.s_stack().push2(t as isize, 0) {
                Err(_) => self.set_error(Some(StackOverflow)),
                Ok(()) => {}
            }
        }
        Ok(())
    }

    /// Run-time: ( -- ud )
    ///
    /// Current time in microseconds since some epoch
    ///
    /// Examples:
    ///
    /// ```
    /// use rtforth::vm::VM;
    /// use rtforth::core::Core;
    /// use rtforth::facility::Facility;
    /// use rtforth::tools::Tools;
    /// let vm = &mut VM::new(16);
    /// vm.add_facility();
    /// vm.add_tools();
    /// vm.set_source("utime .s");
    /// vm.evaluate();
    /// ```
    fn utime(&mut self) -> Result {
        let t = time::precise_time_ns() / 1000;
        if t > usize::max_value() as u64 {
            match self.s_stack().push2(t.bitand(usize::max_value() as u64) as isize,
                                       t.shr(usize::max_value().count_ones()) as isize) {
                Err(_) => self.set_error(Some(StackOverflow)),
                Ok(()) => {}
            }
        } else {
            match self.s_stack().push2(t as isize, 0) {
                Err(_) => self.set_error(Some(StackOverflow)),
                Ok(()) => {}
            }
        }
        Ok(())
    }
}
