use core::{Result, Core, TRUE, FALSE};
use std::str::FromStr;

use std::mem;

use exception::Exception::{StackUnderflow, StackOverflow, FloatingPointStackOverflow,
                           FloatingPointStackUnderflow, UnsupportedOperation};

pub trait Float: Core {
    fn add_float(&mut self) {
        self.add_primitive("flit", Float::flit);
        self.add_primitive("fconstant", Float::fconstant);
        self.add_primitive("fvariable", Float::fvariable);
        self.add_primitive("f!", Float::fstore);
        self.add_primitive("f@", Float::ffetch);
        self.add_primitive("fabs", Float::fabs);
        self.add_primitive("fsin", Float::fsin);
        self.add_primitive("fcos", Float::fcos);
        self.add_primitive("ftan", Float::ftan);
        self.add_primitive("fasin", Float::fasin);
        self.add_primitive("facos", Float::facos);
        self.add_primitive("fatan", Float::fatan);
        self.add_primitive("fatan2", Float::fatan2);
        self.add_primitive("fsqrt", Float::fsqrt);
        self.add_primitive("fdrop", Float::fdrop);
        self.add_primitive("fdup", Float::fdup);
        self.add_primitive("fswap", Float::fswap);
        self.add_primitive("fnip", Float::fnip);
        self.add_primitive("frot", Float::frot);
        self.add_primitive("fover", Float::fover);
        self.add_primitive("n>f", Float::n_to_f);
        self.add_primitive("f+", Float::fplus);
        self.add_primitive("f-", Float::fminus);
        self.add_primitive("f*", Float::fstar);
        self.add_primitive("f/", Float::fslash);
        self.add_primitive("f~", Float::fproximate);
        self.add_primitive("f0<", Float::f_zero_less_than);
        self.add_primitive("f0=", Float::f_zero_equals);
        self.add_primitive("f<", Float::f_less_than);

        self.extend_evaluator(Float::evaluate_float);
        self.references().idx_flit = self.find("flit").expect("flit undefined");
    }

    /// Compile float 'f'.
    fn compile_float(&mut self, f: f64) {
        let idx_flit = self.references().idx_flit;
        self.jit_memory().compile_i32(idx_flit as i32);
        self.jit_memory().compile_f64(f);
    }

    /// Evaluate float.
    fn evaluate_float(&mut self, token: &str) -> Result {
        match FromStr::from_str(token) {
            Ok(t) => {
                if self.references().idx_flit == 0 {
                    print!("{} ", "Floating point");
                    Err(UnsupportedOperation)
                } else {
                    if self.state().is_compiling {
                        self.compile_float(t);
                        Ok(())
                    } else {
                        self.f_stack().push(t).or(Err(FloatingPointStackOverflow))
                    }
                }
            }
            Err(_) => Err(UnsupportedOperation),
        }
    }

    fn flit(&mut self) -> Result {
        let ip = self.state().instruction_pointer as usize;
        let v = self.jit_memory().get_f64(ip);
        match self.f_stack().push(v) {
            Err(_) => Err(FloatingPointStackOverflow),
            Ok(()) => {
                self.state().instruction_pointer = self.state().instruction_pointer +
                                                   mem::size_of::<f64>();
                Ok(())
            }
        }
    }

    fn p_fconst(&mut self) -> Result {
        let wp = self.state().word_pointer();
        let dfa = self.wordlist()[wp].dfa();
        let v = self.jit_memory().get_f64(dfa);
        match self.f_stack().push(v) {
            Err(_) => Err(FloatingPointStackOverflow),
            Ok(()) => Ok(()),
        }
    }

    fn fvariable(&mut self) -> Result {
        try!(self.define(Core::p_fvar));
        self.jit_memory().compile_f64(0.0);
        Ok(())
    }

    fn fconstant(&mut self) -> Result {
        match self.f_stack().pop() {
            Ok(v) => {
                try!(self.define(Float::p_fconst));
                self.jit_memory().compile_f64(v);
                Ok(())
            }
            Err(_) => Err(FloatingPointStackUnderflow),
        }
    }

    // Floating point primitives

    fn ffetch(&mut self) -> Result {
        match self.s_stack().pop() {
            Ok(t) => {
                let value = self.jit_memory().get_f64(t as usize);
                match self.f_stack().push(value) {
                    Err(_) => Err(FloatingPointStackOverflow),
                    Ok(()) => Ok(()),
                }
            }
            Err(_) => Err(StackUnderflow),
        }
    }

    fn fstore(&mut self) -> Result {
        match self.s_stack().pop() {
            Ok(t) => {
                match self.f_stack().pop() {
                    Ok(n) => {
                        self.jit_memory().put_f64(n, t as usize);
                        Ok(())
                    }
                    Err(_) => Err(StackUnderflow),
                }
            }
            Err(_) => Err(StackUnderflow),
        }
    }

    fn fabs(&mut self) -> Result {
        match self.f_stack().pop() {
            Ok(t) => {
                match self.f_stack().push(t.abs()) {
                    Err(_) => Err(FloatingPointStackOverflow),
                    Ok(()) => Ok(()),
                }
            }
            Err(_) => Err(FloatingPointStackUnderflow),
        }
    }

    fn fsin(&mut self) -> Result {
        match self.f_stack().pop() {
            Ok(t) => {
                match self.f_stack().push(t.sin()) {
                    Err(_) => Err(FloatingPointStackOverflow),
                    Ok(()) => Ok(()),
                }
            }
            Err(_) => Err(FloatingPointStackUnderflow),
        }
    }

    fn fcos(&mut self) -> Result {
        match self.f_stack().pop() {
            Ok(t) => {
                match self.f_stack().push(t.cos()) {
                    Err(_) => Err(FloatingPointStackOverflow),
                    Ok(()) => Ok(()),
                }
            }
            Err(_) => Err(FloatingPointStackUnderflow),
        }
    }

    fn ftan(&mut self) -> Result {
        match self.f_stack().pop() {
            Ok(t) => {
                match self.f_stack().push(t.tan()) {
                    Err(_) => Err(FloatingPointStackOverflow),
                    Ok(()) => Ok(()),
                }
            }
            Err(_) => Err(FloatingPointStackUnderflow),
        }
    }

    fn fasin(&mut self) -> Result {
        match self.f_stack().pop() {
            Ok(t) => {
                match self.f_stack().push(t.asin()) {
                    Err(_) => Err(FloatingPointStackOverflow),
                    Ok(()) => Ok(()),
                }
            }
            Err(_) => Err(FloatingPointStackUnderflow),
        }
    }

    fn facos(&mut self) -> Result {
        match self.f_stack().pop() {
            Ok(t) => {
                match self.f_stack().push(t.acos()) {
                    Err(_) => Err(FloatingPointStackOverflow),
                    Ok(()) => Ok(()),
                }
            }
            Err(_) => Err(FloatingPointStackUnderflow),
        }
    }

    fn fatan(&mut self) -> Result {
        match self.f_stack().pop() {
            Ok(t) => {
                match self.f_stack().push(t.atan()) {
                    Err(_) => Err(FloatingPointStackOverflow),
                    Ok(()) => Ok(()),
                }
            }
            Err(_) => Err(FloatingPointStackUnderflow),
        }
    }

    fn fatan2(&mut self) -> Result {
        match self.f_stack().pop() {
            Ok(t) => {
                match self.f_stack().pop() {
                    Ok(n) => {
                        match self.f_stack().push(n.atan2(t)) {
                            Err(_) => Err(FloatingPointStackOverflow),
                            Ok(()) => Ok(()),
                        }
                    }
                    Err(_) => Err(FloatingPointStackUnderflow),
                }
            }
            Err(_) => Err(FloatingPointStackUnderflow),
        }
    }

    fn fsqrt(&mut self) -> Result {
        match self.f_stack().pop() {
            Ok(t) => {
                match self.f_stack().push(t.sqrt()) {
                    Err(_) => Err(FloatingPointStackOverflow),
                    Ok(()) => Ok(()),
                }
            }
            Err(_) => Err(FloatingPointStackUnderflow),
        }
    }

    fn fswap(&mut self) -> Result {
        match self.f_stack().pop() {
            Ok(t) => {
                match self.f_stack().pop() {
                    Ok(n) => {
                        match self.f_stack().push2(t, n) {
                            Err(_) => Err(FloatingPointStackOverflow),
                            Ok(()) => Ok(()),
                        }
                    }
                    Err(_) => Err(FloatingPointStackUnderflow),
                }
            }
            Err(_) => Err(FloatingPointStackUnderflow),
        }
    }

    fn fnip(&mut self) -> Result {
        match self.f_stack().pop() {
            Ok(t) => {
                match self.f_stack().pop() {
                    Ok(_) => {
                        match self.f_stack().push(t) {
                            Err(_) => Err(FloatingPointStackOverflow),
                            Ok(()) => Ok(()),
                        }
                    }
                    Err(_) => Err(FloatingPointStackUnderflow),
                }
            }
            Err(_) => Err(FloatingPointStackUnderflow),
        }
    }

    fn fdup(&mut self) -> Result {
        match self.f_stack().pop() {
            Ok(t) => {
                match self.f_stack().push2(t, t) {
                    Err(_) => Err(FloatingPointStackOverflow),
                    Ok(()) => Ok(()),
                }
            }
            Err(_) => Err(FloatingPointStackUnderflow),
        }
    }

    fn fdrop(&mut self) -> Result {
        match self.f_stack().pop() {
            Ok(_) => Ok(()),
            Err(_) => Err(FloatingPointStackUnderflow),
        }
    }

    fn frot(&mut self) -> Result {
        match self.f_stack().pop() {
            Ok(x3) => {
                match self.f_stack().pop() {
                    Ok(x2) => {
                        match self.f_stack().pop() {
                            Ok(x1) => {
                                match self.f_stack().push3(x2, x3, x1) {
                                    Err(_) => Err(FloatingPointStackOverflow),
                                    Ok(()) => Ok(()),
                                }
                            }
                            Err(_) => Err(FloatingPointStackUnderflow),
                        }
                    }
                    Err(_) => Err(FloatingPointStackUnderflow),
                }
            }
            Err(_) => Err(FloatingPointStackUnderflow),
        }
    }

    fn fover(&mut self) -> Result {
        match self.f_stack().pop() {
            Ok(t) => {
                match self.f_stack().pop() {
                    Ok(n) => {
                        match self.f_stack().push3(n, t, n) {
                            Err(_) => Err(FloatingPointStackOverflow),
                            Ok(()) => Ok(()),
                        }
                    }
                    Err(_) => Err(FloatingPointStackUnderflow),
                }
            }
            Err(_) => Err(FloatingPointStackUnderflow),
        }
    }

    fn n_to_f(&mut self) -> Result {
        match self.s_stack().pop() {
            Ok(t) => {
                match self.f_stack().push(t as f64) {
                    Err(_) => Err(FloatingPointStackOverflow),
                    Ok(()) => Ok(()),
                }
            }
            Err(_) => Err(FloatingPointStackUnderflow),
        }
    }

    fn fplus(&mut self) -> Result {
        match self.f_stack().pop() {
            Ok(t) => {
                match self.f_stack().pop() {
                    Ok(n) => {
                        match self.f_stack().push(n + t) {
                            Err(_) => Err(FloatingPointStackOverflow),
                            Ok(()) => Ok(()),
                        }
                    }
                    Err(_) => Err(FloatingPointStackUnderflow),
                }
            }
            Err(_) => Err(FloatingPointStackUnderflow),
        }
    }

    fn fminus(&mut self) -> Result {
        match self.f_stack().pop() {
            Ok(t) => {
                match self.f_stack().pop() {
                    Ok(n) => {
                        match self.f_stack().push(n - t) {
                            Err(_) => Err(FloatingPointStackOverflow),
                            Ok(()) => Ok(()),
                        }
                    }
                    Err(_) => Err(FloatingPointStackUnderflow),
                }
            }
            Err(_) => Err(FloatingPointStackUnderflow),
        }
    }

    fn fstar(&mut self) -> Result {
        match self.f_stack().pop() {
            Ok(t) => {
                match self.f_stack().pop() {
                    Ok(n) => {
                        match self.f_stack().push(n * t) {
                            Err(_) => Err(FloatingPointStackOverflow),
                            Ok(()) => Ok(()),
                        }
                    }
                    Err(_) => Err(FloatingPointStackUnderflow),
                }
            }
            Err(_) => Err(FloatingPointStackUnderflow),
        }
    }

    fn fslash(&mut self) -> Result {
        match self.f_stack().pop() {
            Ok(t) => {
                match self.f_stack().pop() {
                    Ok(n) => {
                        match self.f_stack().push(n / t) {
                            Err(_) => Err(FloatingPointStackOverflow),
                            Ok(()) => Ok(()),
                        }
                    }
                    Err(_) => Err(FloatingPointStackUnderflow),
                }
            }
            Err(_) => Err(FloatingPointStackUnderflow),
        }
    }

    fn fproximate(&mut self) -> Result {
        match self.f_stack().pop3() {
            Ok((x1, x2, x3)) => {
                if x3 > 0.0 {
                    self.s_stack().push(if (x1 - x2).abs() < x3 { TRUE } else { FALSE })
                } else if x3 == 0.0 {
                    self.s_stack().push(if x1 == x2 { TRUE } else { FALSE })
                } else {
                    self.s_stack().push(if (x1 - x2).abs() < (x3.abs() * (x1.abs() + x2.abs())) {
                        TRUE
                    } else {
                        FALSE
                    })
                }
            }
            Err(_) => Err(FloatingPointStackUnderflow),
        }
    }

    fn f_zero_less_than(&mut self) -> Result {
        match self.f_stack().pop() {
            Ok(t) => {
                match self.s_stack().push(if t < 0.0 { TRUE } else { FALSE }) {
                    Err(_) => Err(StackOverflow),
                    Ok(()) => Ok(()),
                }
            }
            Err(_) => Err(FloatingPointStackUnderflow),
        }
    }

    fn f_zero_equals(&mut self) -> Result {
        match self.f_stack().pop() {
            Ok(t) => {
                match self.s_stack().push(if t == 0.0 { TRUE } else { FALSE }) {
                    Err(_) => Err(StackOverflow),
                    Ok(()) => Ok(()),
                }
            }
            Err(_) => Err(FloatingPointStackUnderflow),
        }
    }

    fn f_less_than(&mut self) -> Result {
        match self.f_stack().pop() {
            Ok(t) => {
                match self.f_stack().pop() {
                    Ok(n) => {
                        match self.s_stack().push(if n < t { TRUE } else { FALSE }) {
                            Err(_) => Err(StackOverflow),
                            Ok(()) => Ok(()),
                        }
                    }
                    Err(_) => Err(FloatingPointStackUnderflow),
                }
            }
            Err(_) => Err(FloatingPointStackUnderflow),
        }
    }
}

#[cfg(test)]
mod tests {
    use vm::VM;
    use core::Core;
    use super::Float;

    #[test]
    fn test_evaluate_f64() {
        let vm = &mut VM::new(16);
        vm.add_core();
        vm.add_float();
        vm.set_source("1.0 2.5");
        assert!(vm.evaluate().is_ok());
        assert_eq!(vm.f_stack().len(), 2);
        assert!(0.99999 < vm.f_stack().as_slice()[0]);
        assert!(vm.f_stack().as_slice()[0] < 1.00001);
        assert!(2.49999 < vm.f_stack().as_slice()[1]);
        assert!(vm.f_stack().as_slice()[1] < 2.50001);
    }

    #[test]
    fn test_fconstant() {
        let vm = &mut VM::new(16);
        vm.add_core();
        vm.add_float();
        vm.set_source("1.1 fconstant x x x");
        assert!(vm.evaluate().is_ok());
        assert_eq!(vm.f_stack().as_slice(), [1.1, 1.1]);
    }

    #[test]
    fn test_fvariable_and_fstore_ffetch() {
        let vm = &mut VM::new(16);
        vm.add_core();
        vm.add_float();
        vm.set_source("fvariable fx  fx f@  3.3 fx f!  fx f@");
        assert!(vm.evaluate().is_ok());
        assert_eq!(vm.f_stack().as_slice(), [0.0, 3.3]);
    }

    #[test]
    fn test_fabs() {
        let vm = &mut VM::new(16);
        vm.add_core();
        vm.add_float();
        vm.set_source("-3.14 fabs");
        assert!(vm.evaluate().is_ok());
        assert_eq!(vm.f_stack().len(), 1);
        assert!(match vm.f_stack().pop() {
            Ok(t) => t > 3.13999 && t < 3.14001,
            Err(_) => false,
        });
    }

    #[test]
    fn test_fsin() {
        let vm = &mut VM::new(16);
        vm.add_core();
        vm.add_float();
        vm.set_source("3.14 fsin");
        assert!(vm.evaluate().is_ok());
        assert_eq!(vm.f_stack().len(), 1);
        assert!(match vm.f_stack().pop() {
            Ok(t) => t > 0.0015925 && t < 0.0015927,
            Err(_) => false,
        });
    }

    #[test]
    fn test_fcos() {
        let vm = &mut VM::new(16);
        vm.add_core();
        vm.add_float();
        vm.set_source("3.0 fcos");
        assert!(vm.evaluate().is_ok());
        assert_eq!(vm.f_stack().len(), 1);
        assert!(match vm.f_stack().pop() {
            Ok(t) => t > -0.989993 && t < -0.989991,
            Err(_) => false,
        });
    }

    #[test]
    fn test_ftan() {
        let vm = &mut VM::new(16);
        vm.add_core();
        vm.add_float();
        vm.set_source("3.0 ftan");
        assert!(vm.evaluate().is_ok());
        assert_eq!(vm.f_stack().len(), 1);
        assert!(match vm.f_stack().pop() {
            Ok(t) => t > -0.142547 && t < -0.142545,
            Err(_) => false,
        });
    }

    #[test]
    fn test_fasin() {
        let vm = &mut VM::new(16);
        vm.add_core();
        vm.add_float();
        vm.set_source("0.3 fasin");
        assert!(vm.evaluate().is_ok());
        assert_eq!(vm.f_stack().len(), 1);
        assert!(match vm.f_stack().pop() {
            Ok(t) => t > 0.304691 && t < 0.304693,
            Err(_) => false,
        });
    }

    #[test]
    fn test_facos() {
        let vm = &mut VM::new(16);
        vm.add_core();
        vm.add_float();
        vm.set_source("0.3 facos");
        assert!(vm.evaluate().is_ok());
        assert_eq!(vm.f_stack().len(), 1);
        assert!(match vm.f_stack().pop() {
            Ok(t) => t > 1.266102 && t < 1.266104,
            Err(_) => false,
        });
    }

    #[test]
    fn test_fatan() {
        let vm = &mut VM::new(16);
        vm.add_core();
        vm.add_float();
        vm.set_source("0.3 fatan");
        assert!(vm.evaluate().is_ok());
        assert_eq!(vm.f_stack().len(), 1);
        assert!(match vm.f_stack().pop() {
            Ok(t) => t > 0.291455 && t < 0.291457,
            Err(_) => false,
        });
    }

    #[test]
    fn test_fatan2() {
        let vm = &mut VM::new(16);
        vm.add_core();
        vm.add_float();
        vm.set_source("3.0 4.0 fatan2");
        assert!(vm.evaluate().is_ok());
        assert_eq!(vm.f_stack().len(), 1);
        assert!(match vm.f_stack().pop() {
            Ok(t) => t > 0.643500 && t < 0.643502,
            Err(_) => false,
        });
    }

    #[test]
    fn test_fsqrt() {
        let vm = &mut VM::new(16);
        vm.add_core();
        vm.add_float();
        vm.set_source("0.3 fsqrt");
        assert!(vm.evaluate().is_ok());
        assert_eq!(vm.f_stack().len(), 1);
        assert!(match vm.f_stack().pop() {
            Ok(t) => t > 0.547721 && t < 0.547723,
            Err(_) => false,
        });
    }

    #[test]
    fn test_fdrop() {
        let vm = &mut VM::new(16);
        vm.add_core();
        vm.add_float();
        match vm.f_stack().push(1.0) {
            Err(_) => assert!(true, "Floating point stack overflow"),
            Ok(()) => {}
        }
        assert!(vm.fdrop().is_ok());
        assert_eq!(vm.f_stack().as_slice(), []);
    }

    #[test]
    fn test_fnip() {
        let vm = &mut VM::new(16);
        vm.add_core();
        vm.add_float();
        match vm.f_stack().push2(1.0, 2.0) {
            Err(_) => assert!(true, "Floating point stack overflow"),
            Ok(()) => {}
        };
        assert!(vm.fnip().is_ok());
        assert_eq!(vm.f_stack().as_slice(), [2.0]);
    }

    #[test]
    fn test_fswap() {
        let vm = &mut VM::new(16);
        vm.add_core();
        vm.add_float();
        match vm.f_stack().push2(1.0, 2.0) {
            Err(_) => assert!(true, "Floating point stack overflow"),
            Ok(()) => {}
        };
        assert!(vm.fswap().is_ok());
        assert_eq!(vm.f_stack().as_slice(), [2.0, 1.0]);
    }

    #[test]
    fn test_fdup() {
        let vm = &mut VM::new(16);
        vm.add_core();
        vm.add_float();
        match vm.f_stack().push(1.0) {
            Err(_) => assert!(true, "Floating point stack overflow"),
            Ok(()) => {}
        };
        assert!(vm.fdup().is_ok());
        assert_eq!(vm.f_stack().as_slice(), [1.0, 1.0]);
    }

    #[test]
    fn test_fover() {
        let vm = &mut VM::new(16);
        vm.add_core();
        vm.add_float();
        match vm.f_stack().push2(1.0, 2.0) {
            Err(_) => assert!(true, "Floating point stack overflow"),
            Ok(()) => {}
        };
        assert!(vm.fover().is_ok());
        assert_eq!(vm.f_stack().as_slice(), [1.0, 2.0, 1.0]);
    }

    #[test]
    fn test_frot() {
        let vm = &mut VM::new(16);
        vm.add_core();
        vm.add_float();
        match vm.f_stack().push3(1.0, 2.0, 3.0) {
            Err(_) => assert!(true, "Floating point stack overflow"),
            Ok(()) => {}
        };
        assert!(vm.frot().is_ok());
        assert_eq!(vm.f_stack().as_slice(), [2.0, 3.0, 1.0]);
    }

    #[test]
    fn test_fplus_fminus_fstar_fslash() {
        let vm = &mut VM::new(16);
        vm.add_core();
        vm.add_float();
        vm.set_source("9.0 10.0 f+ 11.0 f- 12.0 f* 13.0 f/");
        assert!(vm.evaluate().is_ok());
        assert_eq!(vm.f_stack().len(), 1);
        assert!(match vm.f_stack().pop() {
            Ok(t) => t > 7.384614 && t < 7.384616,
            Err(_) => false,
        });
    }

    #[test]
    fn test_f_zero_less_than() {
        let vm = &mut VM::new(16);
        vm.add_core();
        vm.add_float();
        vm.set_source("0.0 f0<   0.1 f0<   -0.1 f0<");
        assert!(vm.evaluate().is_ok());
        assert_eq!(vm.s_stack().len(), 3);
        assert_eq!(vm.s_stack().pop(), Ok(-1));
        assert_eq!(vm.s_stack().pop(), Ok(0));
        assert_eq!(vm.s_stack().pop(), Ok(0));
        assert_eq!(vm.f_stack().as_slice(), []);
    }

    #[test]
    fn test_f_zero_equals() {
        let vm = &mut VM::new(16);
        vm.add_core();
        vm.add_float();
        vm.set_source("0.0 f0=   0.1 f0=   -0.1 f0=");
        assert!(vm.evaluate().is_ok());
        assert_eq!(vm.s_stack().len(), 3);
        assert_eq!(vm.s_stack().pop(), Ok(0));
        assert_eq!(vm.s_stack().pop(), Ok(0));
        assert_eq!(vm.s_stack().pop(), Ok(-1));
        assert_eq!(vm.f_stack().as_slice(), []);
    }

    #[test]
    fn test_f_less_than() {
        let vm = &mut VM::new(16);
        vm.add_core();
        vm.add_float();
        vm.set_source("0.0 0.0 f<   0.1 0.0 f<   -0.1 0.0 f<");
        assert!(vm.evaluate().is_ok());
        assert_eq!(vm.s_stack().len(), 3);
        assert_eq!(vm.s_stack().pop(), Ok(-1));
        assert_eq!(vm.s_stack().pop(), Ok(0));
        assert_eq!(vm.s_stack().pop(), Ok(0));
        assert_eq!(vm.f_stack().as_slice(), []);
    }

    #[test]
    fn test_fproximate() {
        let vm = &mut VM::new(16);
        vm.add_core();
        vm.add_float();
        vm.set_source("0.1 0.1 0.0 f~   0.1 0.10000000001 0.0 f~");
        assert!(vm.evaluate().is_ok());
        assert_eq!(vm.s_stack().len(), 2);
        assert_eq!(vm.s_stack().pop(), Ok(0));
        assert_eq!(vm.s_stack().pop(), Ok(-1));
        assert_eq!(vm.f_stack().as_slice(), []);
        vm.s_stack().clear();
        vm.set_source("0.1 0.1 0.001 f~   0.1 0.109 0.01 f~   0.1 0.111  0.01 f~");
        assert!(vm.evaluate().is_ok());
        assert_eq!(vm.s_stack().len(), 3);
        assert_eq!(vm.s_stack().pop(), Ok(0));
        assert_eq!(vm.s_stack().pop(), Ok(-1));
        assert_eq!(vm.s_stack().pop(), Ok(-1));
        assert_eq!(vm.f_stack().as_slice(), []);
        vm.s_stack().clear();
        vm.set_source("0.1 0.1 -0.001 f~   0.1 0.109 -0.1 f~   0.1 0.109  -0.01 f~");
        assert!(vm.evaluate().is_ok());
        assert_eq!(vm.s_stack().len(), 3);
        assert_eq!(vm.s_stack().pop(), Ok(0));
        assert_eq!(vm.s_stack().pop(), Ok(-1));
        assert_eq!(vm.s_stack().pop(), Ok(-1));
        assert_eq!(vm.f_stack().as_slice(), []);
        vm.s_stack().clear();
    }

    #[test]
    fn test_n_to_f() {
        let vm = &mut VM::new(16);
        vm.add_core();
        vm.add_float();
        vm.set_source("0 n>f -1 n>f 1 n>f");
        assert!(vm.evaluate().is_ok());
        assert_eq!(vm.f_stack().as_slice(), [0.0, -1.0, 1.0]);
    }

    #[test]
    fn test_flit_and_compile_float() {
        let vm = &mut VM::new(16);
        vm.add_core();
        vm.add_float();
        vm.set_source(": test 1.0 2.0 ; test");
        assert!(vm.evaluate().is_ok());
        assert_eq!(vm.f_stack().as_slice(), [1.0, 2.0]);
    }
}
