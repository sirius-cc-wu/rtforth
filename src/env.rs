use core::VM;
use exception::Exception::{
    StackOverflow,
};


pub trait Environment {
    /// Add environment queries.
    fn add_environment(&mut self);

    /// Run-time: ( -- n )
    ///
    /// Largest usable signed integer
    fn max_n(&mut self);

    /// Run-time: ( -- u )
    ///
    /// Largest usable unsigned integer
    fn max_u(&mut self);
}

impl Environment for VM {
    fn add_environment(&mut self) {
        self.add_primitive("max-n", VM::max_n);
        self.add_primitive("max-u", VM::max_u);
    }

    fn max_n(&mut self) {
        match self.s_stack.push(isize::max_value()) {
            Some(_) => self.abort_with_error(StackOverflow),
            None => {}
        };
    }

    fn max_u(&mut self) {
        match self.s_stack.push(usize::max_value() as isize) {
            Some(_) => self.abort_with_error(StackOverflow),
            None => {}
        }
    }

}

#[cfg(test)]
mod tests {
    use core::VM;
    use super::*;

    #[test]
    fn test_max_n() {
        let mut vm = VM::new();
        vm.add_environment();
        vm.set_source("max-n dup 1+ +");
        vm.evaluate();
        match vm.s_stack.pop() {
            Some(t) => assert_eq!(t, -1),
            None => assert!(false)
        }
    }
    #[test]

    fn test_max_u() {
        let mut vm = VM::new();
        vm.add_environment();
        vm.set_source("max-u 1+");
        vm.evaluate();
        match vm.s_stack.pop() {
            Some(t) => assert_eq!(t, 0),
            None => assert!(false)
        }
    }
}
