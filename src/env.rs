use core::Core;
use exception::Exception::{
    self,
    StackOverflow,
};

pub trait Environment : Core {
    /// Add environment queries.
    fn add_environment(&mut self) {
        self.add_primitive("max-n", Environment::max_n);
        self.add_primitive("max-u", Environment::max_u);
    }

    /// Run-time: ( -- n )
    ///
    /// Largest usable signed integer
    fn max_n(&mut self) -> Option<Exception> {
        match self.s_stack().push(isize::max_value()) {
            Some(_) => Some(StackOverflow),
            None => None
        }
    }

    /// Run-time: ( -- u )
    ///
    /// Largest usable unsigned integer
    fn max_u(&mut self) -> Option<Exception> {
        match self.s_stack().push(usize::max_value() as isize) {
            Some(_) => Some(StackOverflow),
            None => None
        }
    }

}

#[cfg(test)]
mod tests {
    use vm::VM;
    use core::Core;
    use super::*;

    #[test]
    fn test_max_n() {
        let vm = &mut VM::new(16);
        vm.add_core();
        vm.add_environment();
        vm.set_source("max-n dup 1+ +");
        vm.evaluate();
        match vm.s_stack().pop() {
            Some(t) => assert_eq!(t, -1),
            None => assert!(false)
        }
    }
    #[test]

    fn test_max_u() {
        let vm = &mut VM::new(16);
        vm.add_core();
        vm.add_environment();
        vm.set_source("max-u 1+");
        vm.evaluate();
        match vm.s_stack().pop() {
            Some(t) => assert_eq!(t, 0),
            None => assert!(false)
        }
    }
}
