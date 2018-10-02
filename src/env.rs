use core::Core;

pub trait Environment: Core {
    /// Add environment queries.
    fn add_environment(&mut self) {
        self.add_primitive("max-n", Environment::max_n);
        self.add_primitive("max-u", Environment::max_u);
    }

    /// Run-time: ( -- n )
    ///
    /// Largest usable signed integer
    primitive!{fn max_n(&mut self) {
        self.s_stack().push(isize::max_value());
    }}

    /// Run-time: ( -- u )
    ///
    /// Largest usable unsigned integer
    primitive!{fn max_u(&mut self) {
        self.s_stack().push(usize::max_value() as isize);
    }}
}

#[cfg(test)]
mod tests {
    use core::Core;
    use vm::VM;

    #[test]
    fn test_max_n() {
        let vm = &mut VM::new(16, 16);
        vm.set_source("max-n dup 1+ +");
        vm.evaluate_input();
        assert_eq!(vm.last_error(), None);
        match vm.s_stack().pop() {
            t => assert_eq!(t, -1),
        }
    }
    #[test]

    fn test_max_u() {
        let vm = &mut VM::new(16, 16);
        vm.set_source("max-u 1+");
        vm.evaluate_input();
        assert_eq!(vm.last_error(), None);
        match vm.s_stack().pop() {
            t => assert_eq!(t, 0),
        }
    }
}
