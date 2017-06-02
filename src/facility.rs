use core::Core;
use std::ops::BitAnd;
use std::ops::Shr;
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
    /// vm.set_source("ntime .s");
    /// vm.evaluate();
    /// ```
    extern "fastcall" fn ntime(&mut self) {
        let t = time::precise_time_ns();
        if t > usize::max_value() as u64 {
            self.s_stack()
                .push2(t.bitand(usize::max_value() as u64) as isize,
                       t.shr(usize::max_value().count_ones()) as isize);
        } else {
            self.s_stack().push2(t as isize, 0);
        }
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
    /// vm.set_source("utime .s");
    /// vm.evaluate();
    /// ```
    extern "fastcall" fn utime(&mut self) {
        let t = time::precise_time_ns() / 1000;
        if t > usize::max_value() as u64 {
            self.s_stack()
                .push2(t.bitand(usize::max_value() as u64) as isize,
                       t.shr(usize::max_value().count_ones()) as isize);
        } else {
            self.s_stack().push2(t as isize, 0);
        }
    }
}
