use core::{VM, Access, Core, Heap};
use std::str::FromStr;

use std::mem;

use exception::Exception::{
    self,
    StackUnderflow,
    StackOverflow,
    FloatingPointStackOverflow,
    FloatingPointStackUnderflow,
    UnsupportedOperation,
};

pub trait Float {
    fn add_float(&mut self);
    fn compile_float (&mut self, f: f64);
    fn evaluate_float(&mut self, token: &str) -> Result<(), Exception>;
    fn flit(&mut self) -> Option<Exception>;
    fn p_fconst(&mut self) -> Option<Exception>;
    fn fvariable(&mut self) -> Option<Exception>;
    fn fconstant(&mut self) -> Option<Exception>;
    fn ffetch(&mut self) -> Option<Exception>;
    fn fstore(&mut self) -> Option<Exception>;
    fn fabs(&mut self) -> Option<Exception>;
    fn fsin(&mut self) -> Option<Exception>;
    fn fcos(&mut self) -> Option<Exception>;
    fn ftan(&mut self) -> Option<Exception>;
    fn fasin(&mut self) -> Option<Exception>;
    fn facos(&mut self) -> Option<Exception>;
    fn fatan(&mut self) -> Option<Exception>;
    fn fatan2(&mut self) -> Option<Exception>;
    fn fsqrt(&mut self) -> Option<Exception>;
    fn fswap(&mut self) -> Option<Exception>;
    fn fnip(&mut self) -> Option<Exception>;
    fn fdup(&mut self) -> Option<Exception>;
    fn fdrop(&mut self) -> Option<Exception>;
    fn frot(&mut self) -> Option<Exception>;
    fn fover(&mut self) -> Option<Exception>;
    fn n_to_f(&mut self) -> Option<Exception>;
    fn fplus(&mut self) -> Option<Exception>;
    fn fminus(&mut self) -> Option<Exception>;
    fn fstar(&mut self) -> Option<Exception>;
    fn fslash(&mut self) -> Option<Exception>;
    fn fproximate(&mut self) -> Option<Exception>;
    fn f_zero_less_than(&mut self) -> Option<Exception>;
    fn f_zero_equals(&mut self) -> Option<Exception>;
    fn f_less_than(&mut self) -> Option<Exception>;
}

impl Float for VM {
    fn add_float(&mut self) {
        self.add_primitive("flit", Float::flit);
        self.add_primitive ("fconstant", Float::fconstant);
        self.add_primitive ("fvariable", Float::fvariable);
        self.add_primitive ("f!", Float::fstore);
        self.add_primitive ("f@", Float::ffetch);
        self.add_primitive ("fabs", Float::fabs);
        self.add_primitive ("fsin", Float::fsin);
        self.add_primitive ("fcos", Float::fcos);
        self.add_primitive ("ftan", Float::ftan);
        self.add_primitive ("fasin", Float::fasin);
        self.add_primitive ("facos", Float::facos);
        self.add_primitive ("fatan", Float::fatan);
        self.add_primitive ("fatan2", Float::fatan2);
        self.add_primitive ("fsqrt", Float::fsqrt);
        self.add_primitive ("fdrop", Float::fdrop);
        self.add_primitive ("fdup", Float::fdup);
        self.add_primitive ("fswap", Float::fswap);
        self.add_primitive ("fnip", Float::fnip);
        self.add_primitive ("frot", Float::frot);
        self.add_primitive ("fover", Float::fover);
        self.add_primitive ("n>f", Float::n_to_f);
        self.add_primitive ("f+", Float::fplus);
        self.add_primitive ("f-", Float::fminus);
        self.add_primitive ("f*", Float::fstar);
        self.add_primitive ("f/", Float::fslash);
        self.add_primitive ("f~", Float::fproximate);
        self.add_primitive ("f0<", Float::f_zero_less_than);
        self.add_primitive ("f0=", Float::f_zero_equals);
        self.add_primitive ("f<", Float::f_less_than);

        self.extend_evaluator(Float::evaluate_float);
        self.idx_flit = self.find("flit").expect("flit undefined");
    }

    /// Compile float 'f'.
    fn compile_float (&mut self, f: f64) {
        let idx_flit = self.idx_flit;
        self.jit_memory().compile_i32(idx_flit as i32);
        self.jit_memory().compile_f64(f);
    }

    /// Evaluate float.
    fn evaluate_float(&mut self, token: &str) -> Result<(), Exception> {
        match FromStr::from_str(token) {
            Ok(t) => {
                if self.idx_flit == 0 {
                    print!("{} ", "Floating point");
                    Err(UnsupportedOperation)
                } else {
                    if self.is_compiling {
                        self.compile_float(t);
                    } else {
                        self.f_stack.push (t);
                    }
                    Ok(())
                }
            },
            Err(_) => {
                Err(UnsupportedOperation)
            }
        }
    }

    fn flit(&mut self) -> Option<Exception> {
        let ip = self.instruction_pointer as usize;
        let v = self.jit_memory().get_f64(ip);
        match self.f_stack.push (v) {
            Some(_) => Some(FloatingPointStackOverflow),
            None => {
                self.instruction_pointer = self.instruction_pointer + mem::size_of::<f64>();
                None
            }
        }
    }

    fn p_fconst(&mut self) -> Option<Exception> {
        let wp = self.word_pointer();
        let dfa = self.jit_memory().word(wp).dfa();
        let v = self.jit_memory().get_f64(dfa);
        match self.f_stack.push(v) {
            Some(_) => Some(FloatingPointStackOverflow),
            None => None
        }
    }

    fn fvariable(&mut self) -> Option<Exception> {
        self.define(Core::p_fvar);
        self.jit_memory().compile_f64(0.0);
        None
    }

    fn fconstant(&mut self) -> Option<Exception> {
        match self.f_stack.pop() {
            Some(v) => {
                self.define(Float::p_fconst);
                self.jit_memory().compile_f64(v);
                None
            },
            None => Some(FloatingPointStackUnderflow)
        }
    }

// Floating point primitives

    fn ffetch(&mut self) -> Option<Exception> {
        match self.s_stack.pop() {
            Some(t) => {
                let value = self.jit_memory().get_f64(t as usize);
                match self.f_stack.push(value) {
                    Some(_) => Some(FloatingPointStackOverflow),
                    None => None
                }
            },
            None => Some(StackUnderflow)
        }
    }

    fn fstore(&mut self) -> Option<Exception> {
        match self.s_stack.pop() {
            Some(t) =>
                match self.f_stack.pop() {
                    Some(n) => {
                        self.jit_memory().put_f64(n, t as usize);
                        None
                    },
                    None => Some(StackUnderflow)
                },
            None => Some(StackUnderflow)
        }
    }

    fn fabs(&mut self) -> Option<Exception> {
        match self.f_stack.pop() {
            Some(t) =>
                match self.f_stack.push(t.abs()) {
                    Some(_) => Some(FloatingPointStackOverflow),
                    None => None
                },
            None => Some(FloatingPointStackUnderflow)
        }
    }

    fn fsin(&mut self) -> Option<Exception> {
        match self.f_stack.pop() {
            Some(t) =>
                match self.f_stack.push(t.sin()) {
                    Some(_) => Some(FloatingPointStackOverflow),
                    None => None
                },
            None => Some(FloatingPointStackUnderflow)
        }
    }

    fn fcos(&mut self) -> Option<Exception> {
        match self.f_stack.pop() {
            Some(t) =>
                match self.f_stack.push(t.cos()) {
                    Some(_) => Some(FloatingPointStackOverflow),
                    None => None
                },
            None => Some(FloatingPointStackUnderflow)
        }
    }

    fn ftan(&mut self) -> Option<Exception> {
        match self.f_stack.pop() {
            Some(t) =>
                match self.f_stack.push(t.tan()) {
                    Some(_) => Some(FloatingPointStackOverflow),
                    None => None
                },
            None => Some(FloatingPointStackUnderflow)
        }
    }

    fn fasin(&mut self) -> Option<Exception> {
        match self.f_stack.pop() {
            Some(t) =>
                match self.f_stack.push(t.asin()) {
                    Some(_) => Some(FloatingPointStackOverflow),
                    None => None
                },
            None => Some(FloatingPointStackUnderflow)
        }
    }

    fn facos(&mut self) -> Option<Exception> {
        match self.f_stack.pop() {
            Some(t) =>
                match self.f_stack.push(t.acos()) {
                    Some(_) => Some(FloatingPointStackOverflow),
                    None => None
                },
            None => Some(FloatingPointStackUnderflow)
        }
    }

    fn fatan(&mut self) -> Option<Exception> {
        match self.f_stack.pop() {
            Some(t) =>
                match self.f_stack.push(t.atan()) {
                    Some(_) => Some(FloatingPointStackOverflow),
                    None => None
                },
            None => Some(FloatingPointStackUnderflow)
        }
    }

    fn fatan2(&mut self) -> Option<Exception> {
        match self.f_stack.pop() {
            Some(t) => {
                match self.f_stack.pop() {
                    Some(n) =>
                        match self.f_stack.push(n.atan2(t)) {
                            Some(_) => Some(FloatingPointStackOverflow),
                            None => None
                        },
                    None => Some(FloatingPointStackUnderflow)
                }
            },
            None => Some(FloatingPointStackUnderflow)
        }
    }

    fn fsqrt(&mut self) -> Option<Exception> {
        match self.f_stack.pop() {
            Some(t) =>
                match self.f_stack.push(t.sqrt()) {
                    Some(_) => Some(FloatingPointStackOverflow),
                    None => None
                },
            None => Some(FloatingPointStackUnderflow)
        }
    }

    fn fswap(&mut self) -> Option<Exception> {
        match self.f_stack.pop() {
            Some(t) =>
                match self.f_stack.pop() {
                    Some(n) =>
                        match self.f_stack.push2(t, n) {
                            Some(_) => Some(FloatingPointStackOverflow),
                            None => None
                        },
                    None => Some(FloatingPointStackUnderflow)
                },
            None => Some(FloatingPointStackUnderflow)
        }
    }

    fn fnip(&mut self) -> Option<Exception> {
        match self.f_stack.pop() {
            Some(t) =>
                match self.f_stack.pop() {
                    Some(_) =>
                        match self.f_stack.push(t) {
                            Some(_) => Some(FloatingPointStackOverflow),
                            None => None
                        },
                    None => Some(FloatingPointStackUnderflow)
                },
            None => Some(FloatingPointStackUnderflow)
        }
    }

    fn fdup(&mut self) -> Option<Exception> {
        match self.f_stack.pop() {
            Some(t) => {
                match self.f_stack.push2(t, t) {
                    Some(_) => Some(FloatingPointStackOverflow),
                    None => None
                }
            },
            None => Some(FloatingPointStackUnderflow)
        }
    }

    fn fdrop(&mut self) -> Option<Exception> {
        match self.f_stack.pop() {
            Some(_) => None,
            None => Some(FloatingPointStackUnderflow)
        }
    }

    fn frot(&mut self) -> Option<Exception> {
        match self.f_stack.pop() {
            Some(x3) =>
                match self.f_stack.pop() {
                    Some(x2) =>
                        match self.f_stack.pop() {
                            Some(x1) =>
                                match self.f_stack.push3(x2, x3, x1) {
                                    Some(_) => Some(FloatingPointStackOverflow),
                                    None => None
                                },
                            None => Some(FloatingPointStackUnderflow)
                        },
                    None => Some(FloatingPointStackUnderflow)
                },
            None => Some(FloatingPointStackUnderflow)
        }
    }

    fn fover(&mut self) -> Option<Exception> {
        match self.f_stack.pop() {
            Some(t) =>
                match self.f_stack.pop() {
                    Some(n) =>
                        match self.f_stack.push3(n, t, n) {
                            Some(_) => Some(FloatingPointStackOverflow),
                            None => None
                        },
                    None => Some(FloatingPointStackUnderflow)
                },
            None => Some(FloatingPointStackUnderflow)
        }
    }

    fn n_to_f(&mut self) -> Option<Exception> {
        match self.s_stack.pop() {
            Some(t) =>
                match self.f_stack.push(t as f64) {
                    Some(_) => Some(FloatingPointStackOverflow),
                    None => None
                },
            None => Some(FloatingPointStackUnderflow)
        }
    }

    fn fplus(&mut self) -> Option<Exception> {
        match self.f_stack.pop() {
            Some(t) =>
                match self.f_stack.pop() {
                    Some(n) =>
                        match self.f_stack.push(n+t) {
                            Some(_) => Some(FloatingPointStackOverflow),
                            None => None
                        },
                    None => Some(FloatingPointStackUnderflow)
                },
            None => Some(FloatingPointStackUnderflow)
        }
    }

    fn fminus(&mut self) -> Option<Exception> {
        match self.f_stack.pop() {
            Some(t) =>
                match self.f_stack.pop() {
                    Some(n) =>
                        match self.f_stack.push(n-t) {
                            Some(_) => Some(FloatingPointStackOverflow),
                            None => None
                        },
                    None => Some(FloatingPointStackUnderflow)
                },
            None => Some(FloatingPointStackUnderflow)
        }
    }

    fn fstar(&mut self) -> Option<Exception> {
        match self.f_stack.pop() {
            Some(t) =>
                match self.f_stack.pop() {
                    Some(n) =>
                        match self.f_stack.push(n*t) {
                            Some(_) => Some(FloatingPointStackOverflow),
                            None => None
                        },
                    None => Some(FloatingPointStackUnderflow)
                },
            None => Some(FloatingPointStackUnderflow)
        }
    }

    fn fslash(&mut self) -> Option<Exception> {
        match self.f_stack.pop() {
            Some(t) =>
                match self.f_stack.pop() {
                    Some(n) =>
                        match self.f_stack.push(n/t) {
                            Some(_) => Some(FloatingPointStackOverflow),
                            None => None
                        },
                    None => Some(FloatingPointStackUnderflow)
                },
            None => Some(FloatingPointStackUnderflow)
        }
    }

    fn fproximate(&mut self) -> Option<Exception> {
        match self.f_stack.pop3() {
            Some((x1, x2, x3)) => {
                if x3 > 0.0 {
                    self.s_stack.push(if (x1-x2).abs() < x3 {-1} else {0});
                } else if x3 == 0.0 {
                    self.s_stack.push(if x1==x2 {-1} else {0});
                } else {
                    self.s_stack.push(if (x1-x2).abs() < (x3.abs()*(x1.abs() + x2.abs())) {-1} else {0});
                }
                None
            },
            None => Some(FloatingPointStackUnderflow)
        }
    }

    fn f_zero_less_than(&mut self) -> Option<Exception> {
        match self.f_stack.pop() {
            Some(t) =>
                match self.s_stack.push(if t<0.0 {-1} else {0}) {
                    Some(_) => Some(StackOverflow),
                    None => None
                },
            None => Some(FloatingPointStackUnderflow)
        }
    }

    fn f_zero_equals(&mut self) -> Option<Exception> {
        match self.f_stack.pop() {
            Some(t) =>
                match self.s_stack.push(if t==0.0 {-1} else {0}) {
                    Some(_) => Some(StackOverflow),
                    None => None
                },
            None => Some(FloatingPointStackUnderflow)
        }
    }

    fn f_less_than(&mut self) -> Option<Exception> {
        match self.f_stack.pop() {
            Some(t) =>
                match self.f_stack.pop() {
                    Some(n) =>
                        match self.s_stack.push(if n<t {-1} else {0}) {
                            Some(_) => Some(StackOverflow),
                            None => None
                        },
                    None => Some(FloatingPointStackUnderflow)
                },
            None => Some(FloatingPointStackUnderflow)
        }
    }

}

#[cfg(test)]
mod tests {
    use core::{VM, Core};
    use super::Float;

    #[test]
    fn test_evaluate_f64 () {
        let vm = &mut VM::new(16);
        vm.add_core();
        vm.add_float();
        vm.set_source("1.0 2.5");
        assert!(vm.evaluate().is_none());
        assert_eq!(vm.f_stack.len(), 2);
        assert!(0.99999 < vm.f_stack.as_slice()[0]);
        assert!(vm.f_stack.as_slice()[0] < 1.00001);
        assert!(2.49999 < vm.f_stack.as_slice()[1]);
        assert!(vm.f_stack.as_slice()[1] < 2.50001);
    }

    #[test]
    fn test_fconstant () {
        let vm = &mut VM::new(16);
        vm.add_core();
        vm.add_float();
        vm.set_source("1.1 fconstant x x x");
        assert!(vm.evaluate().is_none());
        assert_eq!(vm.f_stack.as_slice(), [1.1, 1.1]);
    }

    #[test]
    fn test_fvariable_and_fstore_ffetch () {
        let vm = &mut VM::new(16);
        vm.add_core();
        vm.add_float();
        vm.set_source("fvariable fx  fx f@  3.3 fx f!  fx f@");
        assert!(vm.evaluate().is_none());
        assert_eq!(vm.f_stack.as_slice(), [0.0, 3.3]);
    }

    #[test]
    fn test_fabs () {
        let vm = &mut VM::new(16);
        vm.add_core();
        vm.add_float();
        vm.set_source("-3.14 fabs");
        assert!(vm.evaluate().is_none());
        assert_eq!(vm.f_stack.len(), 1);
        assert!(match vm.f_stack.pop() {
            Some(t) => {
                t > 3.13999 && t < 3.14001
            },
            None => false
        });
    }

    #[test]
    fn test_fsin () {
        let vm = &mut VM::new(16);
        vm.add_core();
        vm.add_float();
        vm.set_source("3.14 fsin");
        assert!(vm.evaluate().is_none());
        assert_eq!(vm.f_stack.len(), 1);
        assert!(match vm.f_stack.pop() {
            Some(t) => {
                t > 0.0015925 && t < 0.0015927
            },
            None => false
        });
    }

    #[test]
    fn test_fcos () {
        let vm = &mut VM::new(16);
        vm.add_core();
        vm.add_float();
        vm.set_source("3.0 fcos");
        assert!(vm.evaluate().is_none());
        assert_eq!(vm.f_stack.len(), 1);
        assert!(match vm.f_stack.pop() {
            Some(t) => {
                t > -0.989993 && t < -0.989991
            },
            None => false
        });
    }

    #[test]
    fn test_ftan () {
        let vm = &mut VM::new(16);
        vm.add_core();
        vm.add_float();
        vm.set_source("3.0 ftan");
        assert!(vm.evaluate().is_none());
        assert_eq!(vm.f_stack.len(), 1);
        assert!(match vm.f_stack.pop() {
            Some(t) => {
                t > -0.142547 && t < -0.142545
            },
            None => false
        });
    }

    #[test]
    fn test_fasin () {
        let vm = &mut VM::new(16);
        vm.add_core();
        vm.add_float();
        vm.set_source("0.3 fasin");
        assert!(vm.evaluate().is_none());
        assert_eq!(vm.f_stack.len(), 1);
        assert!(match vm.f_stack.pop() {
            Some(t) => {
                t > 0.304691 && t < 0.304693
            },
            None => false
        });
    }

    #[test]
    fn test_facos () {
        let vm = &mut VM::new(16);
        vm.add_core();
        vm.add_float();
        vm.set_source("0.3 facos");
        assert!(vm.evaluate().is_none());
        assert_eq!(vm.f_stack.len(), 1);
        assert!(match vm.f_stack.pop() {
            Some(t) => {
                t > 1.266102 && t < 1.266104
            },
            None => false
        });
    }

    #[test]
    fn test_fatan () {
        let vm = &mut VM::new(16);
        vm.add_core();
        vm.add_float();
        vm.set_source("0.3 fatan");
        assert!(vm.evaluate().is_none());
        assert_eq!(vm.f_stack.len(), 1);
        assert!(match vm.f_stack.pop() {
            Some(t) => {
                t > 0.291455 && t < 0.291457
            },
            None => false
        });
    }

    #[test]
    fn test_fatan2 () {
        let vm = &mut VM::new(16);
        vm.add_core();
        vm.add_float();
        vm.set_source("3.0 4.0 fatan2");
        assert!(vm.evaluate().is_none());
        assert_eq!(vm.f_stack.len(), 1);
        assert!(match vm.f_stack.pop() {
            Some(t) => {
                t > 0.643500  && t < 0.643502
            },
            None => false
        });
    }

    #[test]
    fn test_fsqrt () {
        let vm = &mut VM::new(16);
        vm.add_core();
        vm.add_float();
        vm.set_source("0.3 fsqrt");
        assert!(vm.evaluate().is_none());
        assert_eq!(vm.f_stack.len(), 1);
        assert!(match vm.f_stack.pop() {
            Some(t) => {
                t > 0.547721 && t < 0.547723
            },
            None => false
        });
    }

    #[test]
    fn test_fdrop() {
        let vm = &mut VM::new(16);
        vm.add_core();
        vm.add_float();
        match vm.f_stack.push(1.0) {
            Some(_) => assert!(true, "Floating point stack overflow"),
            None => {}
        }
        assert!(vm.fdrop().is_none());
        assert_eq!(vm.f_stack.as_slice(), []);
    }

    #[test]
    fn test_fnip() {
        let vm = &mut VM::new(16);
        vm.add_core();
        vm.add_float();
        match vm.f_stack.push2(1.0, 2.0) {
            Some(_) => assert!(true, "Floating point stack overflow"),
            None => {}
        };
        assert!(vm.fnip().is_none());
        assert_eq!(vm.f_stack.as_slice(), [2.0]);
    }

    #[test]
    fn test_fswap () {
        let vm = &mut VM::new(16);
        vm.add_core();
        vm.add_float();
        match vm.f_stack.push2(1.0, 2.0) {
            Some(_) => assert!(true, "Floating point stack overflow"),
            None => {}
        };
        assert!(vm.fswap().is_none());
        assert_eq!(vm.f_stack.as_slice(), [2.0,1.0]);
    }

    #[test]
    fn test_fdup () {
        let vm = &mut VM::new(16);
        vm.add_core();
        vm.add_float();
        match vm.f_stack.push(1.0) {
            Some(_) => assert!(true, "Floating point stack overflow"),
            None => {}
        };
        assert!(vm.fdup().is_none());
        assert_eq!(vm.f_stack.as_slice(), [1.0, 1.0]);
    }

    #[test]
    fn test_fover () {
        let vm = &mut VM::new(16);
        vm.add_core();
        vm.add_float();
        match vm.f_stack.push2(1.0, 2.0) {
            Some(_) => assert!(true, "Floating point stack overflow"),
            None => {}
        };
        assert!(vm.fover().is_none());
        assert_eq!(vm.f_stack.as_slice(), [1.0,2.0,1.0]);
    }

    #[test]
    fn test_frot () {
        let vm = &mut VM::new(16);
        vm.add_core();
        vm.add_float();
        match vm.f_stack.push3(1.0, 2.0, 3.0) {
            Some(_) => assert!(true, "Floating point stack overflow"),
            None => {}
        };
        assert!(vm.frot().is_none());
        assert_eq!(vm.f_stack.as_slice(), [2.0, 3.0, 1.0]);
    }

    #[test]
    fn test_fplus_fminus_fstar_fslash () {
        let vm = &mut VM::new(16);
        vm.add_core();
        vm.add_float();
        vm.set_source("9.0 10.0 f+ 11.0 f- 12.0 f* 13.0 f/");
        assert!(vm.evaluate().is_none());
        assert_eq!(vm.f_stack.len(), 1);
        assert!(match vm.f_stack.pop() {
            Some(t) => {
                t > 7.384614 && t < 7.384616
            },
            None => false
        });
    }

    #[test]
    fn test_f_zero_less_than () {
        let vm = &mut VM::new(16);
        vm.add_core();
        vm.add_float();
        vm.set_source("0.0 f0<   0.1 f0<   -0.1 f0<");
        assert!(vm.evaluate().is_none());
        assert_eq!(vm.s_stack.len(), 3);
        assert_eq!(vm.s_stack.pop(), Some(-1));
        assert_eq!(vm.s_stack.pop(), Some(0));
        assert_eq!(vm.s_stack.pop(), Some(0));
        assert_eq!(vm.f_stack.as_slice(), []);
    }

    #[test]
    fn test_f_zero_equals () {
        let vm = &mut VM::new(16);
        vm.add_core();
        vm.add_float();
        vm.set_source("0.0 f0=   0.1 f0=   -0.1 f0=");
        assert!(vm.evaluate().is_none());
        assert_eq!(vm.s_stack.len(), 3);
        assert_eq!(vm.s_stack.pop(), Some(0));
        assert_eq!(vm.s_stack.pop(), Some(0));
        assert_eq!(vm.s_stack.pop(), Some(-1));
        assert_eq!(vm.f_stack.as_slice(), []);
    }

    #[test]
    fn test_f_less_than () {
        let vm = &mut VM::new(16);
        vm.add_core();
        vm.add_float();
        vm.set_source("0.0 0.0 f<   0.1 0.0 f<   -0.1 0.0 f<");
        assert!(vm.evaluate().is_none());
        assert_eq!(vm.s_stack.len(), 3);
        assert_eq!(vm.s_stack.pop(), Some(-1));
        assert_eq!(vm.s_stack.pop(), Some(0));
        assert_eq!(vm.s_stack.pop(), Some(0));
        assert_eq!(vm.f_stack.as_slice(), []);
    }

    #[test]
    fn test_fproximate () {
        let vm = &mut VM::new(16);
        vm.add_core();
        vm.add_float();
        vm.set_source("0.1 0.1 0.0 f~   0.1 0.10000000001 0.0 f~");
        assert!(vm.evaluate().is_none());
        assert_eq!(vm.s_stack.len(), 2);
        assert_eq!(vm.s_stack.pop(), Some(0));
        assert_eq!(vm.s_stack.pop(), Some(-1));
        assert_eq!(vm.f_stack.as_slice(), []);
        vm.s_stack.clear();
        vm.set_source("0.1 0.1 0.001 f~   0.1 0.109 0.01 f~   0.1 0.111  0.01 f~");
        assert!(vm.evaluate().is_none());
        assert_eq!(vm.s_stack.len(), 3);
        assert_eq!(vm.s_stack.pop(), Some(0));
        assert_eq!(vm.s_stack.pop(), Some(-1));
        assert_eq!(vm.s_stack.pop(), Some(-1));
        assert_eq!(vm.f_stack.as_slice(), []);
        vm.s_stack.clear();
        vm.set_source("0.1 0.1 -0.001 f~   0.1 0.109 -0.1 f~   0.1 0.109  -0.01 f~");
        assert!(vm.evaluate().is_none());
        assert_eq!(vm.s_stack.len(), 3);
        assert_eq!(vm.s_stack.pop(), Some(0));
        assert_eq!(vm.s_stack.pop(), Some(-1));
        assert_eq!(vm.s_stack.pop(), Some(-1));
        assert_eq!(vm.f_stack.as_slice(), []);
        vm.s_stack.clear();
    }

    #[test]
    fn test_n_to_f () {
        let vm = &mut VM::new(16);
        vm.add_core();
        vm.add_float();
        vm.set_source("0 n>f -1 n>f 1 n>f");
        assert!(vm.evaluate().is_none());
        assert_eq!(vm.f_stack.as_slice(), [0.0, -1.0, 1.0]);
    }

    #[test]
    fn test_flit_and_compile_float () {
        let vm = &mut VM::new(16);
        vm.add_core();
        vm.add_float();
        vm.set_source(": test 1.0 2.0 ; test");
        assert!(vm.evaluate().is_none());
        assert_eq!(vm.f_stack.as_slice(), [1.0, 2.0]);
    }
}
