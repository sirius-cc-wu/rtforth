use core::{Core, TRUE, FALSE};
use std::str::FromStr;
use exception::Exception::{StackUnderflow, StackOverflow, FloatingPointStackOverflow,
                           FloatingPointStackUnderflow, UnsupportedOperation};

pub trait Float: Core {
    fn add_float(&mut self) {
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
        self.data_space().compile_i32(idx_flit as i32);
        self.data_space().compile_f64(f);
    }

    /// Evaluate float.
    fn evaluate_float(&mut self, token: &str) {
        match FromStr::from_str(token) {
            Ok(t) => {
                if self.references().idx_flit == 0 {
                    print!("{} ", "Floating point");
                    self.set_error(Some(UnsupportedOperation));
                } else {
                    if self.state().is_compiling {
                        self.compile_float(t);
                    } else {
                        if let Err(_) = self.f_stack().push(t) {
                            self.set_error(Some(FloatingPointStackOverflow));
                        }
                    }
                }
            }
            Err(_) => self.set_error(Some(UnsupportedOperation)),
        }
    }

    fn p_fconst(&mut self) {
        let wp = self.state().word_pointer();
        let dfa = self.wordlist()[wp].dfa();
        let v = self.data_space().get_f64(dfa);
        match self.f_stack().push(v) {
            Err(_) => self.set_error(Some(FloatingPointStackOverflow)),
            Ok(()) => {}
        }
    }

    fn fvariable(&mut self) {
        self.define(Core::p_fvar);
        self.data_space().compile_f64(0.0);
    }

    fn fconstant(&mut self) {
        match self.f_stack().pop() {
            Ok(v) => {
                self.define(Float::p_fconst);
                self.data_space().compile_f64(v);
            }
            Err(_) => self.set_error(Some(FloatingPointStackUnderflow)),
        }
    }

    // Floating point primitives

    fn ffetch(&mut self) {
        match self.s_stack().pop() {
            Ok(t) => {
                let value = self.data_space().get_f64(t as usize);
                match self.f_stack().push(value) {
                    Err(_) => self.set_error(Some(FloatingPointStackOverflow)),
                    Ok(()) => {}
                }
            }
            Err(_) => self.set_error(Some(StackUnderflow)),
        }
    }

    fn fstore(&mut self) {
        match self.s_stack().pop() {
            Ok(t) => {
                match self.f_stack().pop() {
                    Ok(n) => {
                        self.data_space().put_f64(n, t as usize);
                    }
                    Err(_) => self.set_error(Some(StackUnderflow)),
                }
            }
            Err(_) => self.set_error(Some(StackUnderflow)),
        }
    }

    fn fabs(&mut self) {
        match self.f_stack().pop() {
            Ok(t) => {
                match self.f_stack().push(t.abs()) {
                    Err(_) => self.set_error(Some(FloatingPointStackOverflow)),
                    Ok(()) => {}
                }
            }
            Err(_) => self.set_error(Some(FloatingPointStackUnderflow)),
        }
    }

    fn fsin(&mut self) {
        match self.f_stack().pop() {
            Ok(t) => {
                match self.f_stack().push(t.sin()) {
                    Err(_) => self.set_error(Some(FloatingPointStackOverflow)),
                    Ok(()) => {}
                }
            }
            Err(_) => self.set_error(Some(FloatingPointStackUnderflow)),
        }
    }

    fn fcos(&mut self) {
        match self.f_stack().pop() {
            Ok(t) => {
                match self.f_stack().push(t.cos()) {
                    Err(_) => self.set_error(Some(FloatingPointStackOverflow)),
                    Ok(()) => {}
                }
            }
            Err(_) => self.set_error(Some(FloatingPointStackUnderflow)),
        }
    }

    fn ftan(&mut self) {
        match self.f_stack().pop() {
            Ok(t) => {
                match self.f_stack().push(t.tan()) {
                    Err(_) => self.set_error(Some(FloatingPointStackOverflow)),
                    Ok(()) => {}
                }
            }
            Err(_) => self.set_error(Some(FloatingPointStackUnderflow)),
        }
    }

    fn fasin(&mut self) {
        match self.f_stack().pop() {
            Ok(t) => {
                match self.f_stack().push(t.asin()) {
                    Err(_) => self.set_error(Some(FloatingPointStackOverflow)),
                    Ok(()) => {}
                }
            }
            Err(_) => self.set_error(Some(FloatingPointStackUnderflow)),
        }
    }

    fn facos(&mut self) {
        match self.f_stack().pop() {
            Ok(t) => {
                match self.f_stack().push(t.acos()) {
                    Err(_) => self.set_error(Some(FloatingPointStackOverflow)),
                    Ok(()) => {}
                }
            }
            Err(_) => self.set_error(Some(FloatingPointStackUnderflow)),
        }
    }

    fn fatan(&mut self) {
        match self.f_stack().pop() {
            Ok(t) => {
                match self.f_stack().push(t.atan()) {
                    Err(_) => self.set_error(Some(FloatingPointStackOverflow)),
                    Ok(()) => {}
                }
            }
            Err(_) => self.set_error(Some(FloatingPointStackUnderflow)),
        }
    }

    fn fatan2(&mut self) {
        match self.f_stack().pop() {
            Ok(t) => {
                match self.f_stack().pop() {
                    Ok(n) => {
                        match self.f_stack().push(n.atan2(t)) {
                            Err(_) => self.set_error(Some(FloatingPointStackOverflow)),
                            Ok(()) => {}
                        }
                    }
                    Err(_) => self.set_error(Some(FloatingPointStackUnderflow)),
                }
            }
            Err(_) => self.set_error(Some(FloatingPointStackUnderflow)),
        }
    }

    fn fsqrt(&mut self) {
        match self.f_stack().pop() {
            Ok(t) => {
                match self.f_stack().push(t.sqrt()) {
                    Err(_) => self.set_error(Some(FloatingPointStackOverflow)),
                    Ok(()) => {}
                }
            }
            Err(_) => self.set_error(Some(FloatingPointStackUnderflow)),
        }
    }

    fn fswap(&mut self) {
        match self.f_stack().pop() {
            Ok(t) => {
                match self.f_stack().pop() {
                    Ok(n) => {
                        match self.f_stack().push2(t, n) {
                            Err(_) => self.set_error(Some(FloatingPointStackOverflow)),
                            Ok(()) => {}
                        }
                    }
                    Err(_) => self.set_error(Some(FloatingPointStackUnderflow)),
                }
            }
            Err(_) => self.set_error(Some(FloatingPointStackUnderflow)),
        }
    }

    fn fnip(&mut self) {
        match self.f_stack().pop() {
            Ok(t) => {
                match self.f_stack().pop() {
                    Ok(_) => {
                        match self.f_stack().push(t) {
                            Err(_) => self.set_error(Some(FloatingPointStackOverflow)),
                            Ok(()) => {}
                        }
                    }
                    Err(_) => self.set_error(Some(FloatingPointStackUnderflow)),
                }
            }
            Err(_) => self.set_error(Some(FloatingPointStackUnderflow)),
        }
    }

    fn fdup(&mut self) {
        match self.f_stack().pop() {
            Ok(t) => {
                match self.f_stack().push2(t, t) {
                    Err(_) => self.set_error(Some(FloatingPointStackOverflow)),
                    Ok(()) => {}
                }
            }
            Err(_) => self.set_error(Some(FloatingPointStackUnderflow)),
        }
    }

    fn fdrop(&mut self) {
        match self.f_stack().pop() {
            Ok(_) => {}
            Err(_) => self.set_error(Some(FloatingPointStackUnderflow)),
        }
    }

    fn frot(&mut self) {
        match self.f_stack().pop() {
            Ok(x3) => {
                match self.f_stack().pop() {
                    Ok(x2) => {
                        match self.f_stack().pop() {
                            Ok(x1) => {
                                match self.f_stack().push3(x2, x3, x1) {
                                    Err(_) => self.set_error(Some(FloatingPointStackOverflow)),
                                    Ok(()) => {}
                                }
                            }
                            Err(_) => self.set_error(Some(FloatingPointStackUnderflow)),
                        }
                    }
                    Err(_) => self.set_error(Some(FloatingPointStackUnderflow)),
                }
            }
            Err(_) => self.set_error(Some(FloatingPointStackUnderflow)),
        }
    }

    fn fover(&mut self) {
        match self.f_stack().pop() {
            Ok(t) => {
                match self.f_stack().pop() {
                    Ok(n) => {
                        match self.f_stack().push3(n, t, n) {
                            Err(_) => self.set_error(Some(FloatingPointStackOverflow)),
                            Ok(()) => {}
                        }
                    }
                    Err(_) => self.set_error(Some(FloatingPointStackUnderflow)),
                }
            }
            Err(_) => self.set_error(Some(FloatingPointStackUnderflow)),
        }
    }

    fn n_to_f(&mut self) {
        match self.s_stack().pop() {
            Ok(t) => {
                match self.f_stack().push(t as f64) {
                    Err(_) => self.set_error(Some(FloatingPointStackOverflow)),
                    Ok(()) => {}
                }
            }
            Err(_) => self.set_error(Some(FloatingPointStackUnderflow)),
        }
    }

    fn fplus(&mut self) {
        match self.f_stack().pop() {
            Ok(t) => {
                match self.f_stack().pop() {
                    Ok(n) => {
                        match self.f_stack().push(n + t) {
                            Err(_) => self.set_error(Some(FloatingPointStackOverflow)),
                            Ok(()) => {}
                        }
                    }
                    Err(_) => self.set_error(Some(FloatingPointStackUnderflow)),
                }
            }
            Err(_) => self.set_error(Some(FloatingPointStackUnderflow)),
        }
    }

    fn fminus(&mut self) {
        match self.f_stack().pop() {
            Ok(t) => {
                match self.f_stack().pop() {
                    Ok(n) => {
                        match self.f_stack().push(n - t) {
                            Err(_) => self.set_error(Some(FloatingPointStackOverflow)),
                            Ok(()) => {}
                        }
                    }
                    Err(_) => self.set_error(Some(FloatingPointStackUnderflow)),
                }
            }
            Err(_) => self.set_error(Some(FloatingPointStackUnderflow)),
        }
    }

    fn fstar(&mut self) {
        match self.f_stack().pop() {
            Ok(t) => {
                match self.f_stack().pop() {
                    Ok(n) => {
                        match self.f_stack().push(n * t) {
                            Err(_) => self.set_error(Some(FloatingPointStackOverflow)),
                            Ok(()) => {}
                        }
                    }
                    Err(_) => self.set_error(Some(FloatingPointStackUnderflow)),
                }
            }
            Err(_) => self.set_error(Some(FloatingPointStackUnderflow)),
        }
    }

    fn fslash(&mut self) {
        match self.f_stack().pop() {
            Ok(t) => {
                match self.f_stack().pop() {
                    Ok(n) => {
                        match self.f_stack().push(n / t) {
                            Err(_) => self.set_error(Some(FloatingPointStackOverflow)),
                            Ok(()) => {}
                        }
                    }
                    Err(_) => self.set_error(Some(FloatingPointStackUnderflow)),
                }
            }
            Err(_) => self.set_error(Some(FloatingPointStackUnderflow)),
        }
    }

    fn fproximate(&mut self) {
        match self.f_stack().pop3() {
            Ok((x1, x2, x3)) => {
                if x3 > 0.0 {
                    if let Err(e) = self.s_stack()
                        .push(if (x1 - x2).abs() < x3 { TRUE } else { FALSE }) {
                        self.set_error(Some(e));
                    }
                } else if x3 == 0.0 {
                    if let Err(e) = self.s_stack().push(if x1 == x2 { TRUE } else { FALSE }) {
                        self.set_error(Some(e));
                    }
                } else {
                    if let Err(e) = self.s_stack()
                        .push(if (x1 - x2).abs() < (x3.abs() * (x1.abs() + x2.abs())) {
                            TRUE
                        } else {
                            FALSE
                        }) {
                        self.set_error(Some(e));
                    }
                }
            }
            Err(_) => self.set_error(Some(FloatingPointStackUnderflow)),
        }
    }

    fn f_zero_less_than(&mut self) {
        match self.f_stack().pop() {
            Ok(t) => {
                match self.s_stack().push(if t < 0.0 { TRUE } else { FALSE }) {
                    Err(_) => self.set_error(Some(StackOverflow)),
                    Ok(()) => {}
                }
            }
            Err(_) => self.set_error(Some(FloatingPointStackUnderflow)),
        }
    }

    fn f_zero_equals(&mut self) {
        match self.f_stack().pop() {
            Ok(t) => {
                match self.s_stack().push(if t == 0.0 { TRUE } else { FALSE }) {
                    Err(_) => self.set_error(Some(StackOverflow)),
                    Ok(()) => {}
                }
            }
            Err(_) => self.set_error(Some(FloatingPointStackUnderflow)),
        }
    }

    fn f_less_than(&mut self) {
        match self.f_stack().pop() {
            Ok(t) => {
                match self.f_stack().pop() {
                    Ok(n) => {
                        match self.s_stack().push(if n < t { TRUE } else { FALSE }) {
                            Err(_) => self.set_error(Some(StackOverflow)),
                            Ok(()) => {}
                        }
                    }
                    Err(_) => self.set_error(Some(FloatingPointStackUnderflow)),
                }
            }
            Err(_) => self.set_error(Some(FloatingPointStackUnderflow)),
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
        vm.set_source("1.0 2.5");
        vm.evaluate();
        assert_eq!(vm.last_error(), None);
        assert_eq!(vm.f_stack().len(), 2);
        assert!(0.99999 < vm.f_stack().as_slice()[0]);
        assert!(vm.f_stack().as_slice()[0] < 1.00001);
        assert!(2.49999 < vm.f_stack().as_slice()[1]);
        assert!(vm.f_stack().as_slice()[1] < 2.50001);
    }

    #[test]
    fn test_fconstant() {
        let vm = &mut VM::new(16);
        vm.set_source("1.1 fconstant x x x");
        vm.evaluate();
        assert_eq!(vm.last_error(), None);
        assert_eq!(vm.f_stack().as_slice(), [1.1, 1.1]);
    }

    #[test]
    fn test_fvariable_and_fstore_ffetch() {
        let vm = &mut VM::new(16);
        vm.set_source("fvariable fx  fx f@  3.3 fx f!  fx f@");
        vm.evaluate();
        assert_eq!(vm.last_error(), None);
        assert_eq!(vm.f_stack().as_slice(), [0.0, 3.3]);
    }

    #[test]
    fn test_fabs() {
        let vm = &mut VM::new(16);
        vm.set_source("-3.14 fabs");
        vm.evaluate();
        assert_eq!(vm.last_error(), None);
        assert_eq!(vm.f_stack().len(), 1);
        assert!(match vm.f_stack().pop() {
            Ok(t) => t > 3.13999 && t < 3.14001,
            Err(_) => false,
        });
    }

    #[test]
    fn test_fsin() {
        let vm = &mut VM::new(16);
        vm.set_source("3.14 fsin");
        vm.evaluate();
        assert_eq!(vm.last_error(), None);
        assert_eq!(vm.f_stack().len(), 1);
        assert!(match vm.f_stack().pop() {
            Ok(t) => t > 0.0015925 && t < 0.0015927,
            Err(_) => false,
        });
    }

    #[test]
    fn test_fcos() {
        let vm = &mut VM::new(16);
        vm.set_source("3.0 fcos");
        vm.evaluate();
        assert_eq!(vm.last_error(), None);
        assert_eq!(vm.f_stack().len(), 1);
        assert!(match vm.f_stack().pop() {
            Ok(t) => t > -0.989993 && t < -0.989991,
            Err(_) => false,
        });
    }

    #[test]
    fn test_ftan() {
        let vm = &mut VM::new(16);
        vm.set_source("3.0 ftan");
        vm.evaluate();
        assert_eq!(vm.last_error(), None);
        assert_eq!(vm.f_stack().len(), 1);
        assert!(match vm.f_stack().pop() {
            Ok(t) => t > -0.142547 && t < -0.142545,
            Err(_) => false,
        });
    }

    #[test]
    fn test_fasin() {
        let vm = &mut VM::new(16);
        vm.set_source("0.3 fasin");
        vm.evaluate();
        assert_eq!(vm.last_error(), None);
        assert_eq!(vm.f_stack().len(), 1);
        assert!(match vm.f_stack().pop() {
            Ok(t) => t > 0.304691 && t < 0.304693,
            Err(_) => false,
        });
    }

    #[test]
    fn test_facos() {
        let vm = &mut VM::new(16);
        vm.set_source("0.3 facos");
        vm.evaluate();
        assert_eq!(vm.last_error(), None);
        assert_eq!(vm.f_stack().len(), 1);
        assert!(match vm.f_stack().pop() {
            Ok(t) => t > 1.266102 && t < 1.266104,
            Err(_) => false,
        });
    }

    #[test]
    fn test_fatan() {
        let vm = &mut VM::new(16);
        vm.set_source("0.3 fatan");
        vm.evaluate();
        assert_eq!(vm.last_error(), None);
        assert_eq!(vm.f_stack().len(), 1);
        assert!(match vm.f_stack().pop() {
            Ok(t) => t > 0.291455 && t < 0.291457,
            Err(_) => false,
        });
    }

    #[test]
    fn test_fatan2() {
        let vm = &mut VM::new(16);
        vm.set_source("3.0 4.0 fatan2");
        vm.evaluate();
        assert_eq!(vm.last_error(), None);
        assert_eq!(vm.f_stack().len(), 1);
        assert!(match vm.f_stack().pop() {
            Ok(t) => t > 0.643500 && t < 0.643502,
            Err(_) => false,
        });
    }

    #[test]
    fn test_fsqrt() {
        let vm = &mut VM::new(16);
        vm.set_source("0.3 fsqrt");
        vm.evaluate();
        assert_eq!(vm.last_error(), None);
        assert_eq!(vm.f_stack().len(), 1);
        assert!(match vm.f_stack().pop() {
            Ok(t) => t > 0.547721 && t < 0.547723,
            Err(_) => false,
        });
    }

    #[test]
    fn test_fdrop() {
        let vm = &mut VM::new(16);
        match vm.f_stack().push(1.0) {
            Err(_) => assert!(true, "Floating point stack overflow"),
            Ok(()) => {}
        }
        vm.fdrop();
        assert_eq!(vm.last_error(), None);
        assert_eq!(vm.f_stack().as_slice(), []);
    }

    #[test]
    fn test_fnip() {
        let vm = &mut VM::new(16);
        vm.f_stack().push2(1.0, 2.0).unwrap();
        match vm.last_error() {
            Some(_) => assert!(true, "Floating point stack overflow"),
            None => {}
        };
        vm.fnip();
        assert_eq!(vm.last_error(), None);
        assert_eq!(vm.f_stack().as_slice(), [2.0]);
    }

    #[test]
    fn test_fswap() {
        let vm = &mut VM::new(16);
        vm.f_stack().push2(1.0, 2.0).unwrap();
        match vm.last_error() {
            Some(_) => assert!(true, "Floating point stack overflow"),
            None => {}
        };
        vm.fswap();
        assert_eq!(vm.last_error(), None);
        assert_eq!(vm.f_stack().as_slice(), [2.0, 1.0]);
    }

    #[test]
    fn test_fdup() {
        let vm = &mut VM::new(16);
        match vm.f_stack().push(1.0) {
            Err(_) => assert!(true, "Floating point stack overflow"),
            Ok(()) => {}
        };
        vm.fdup();
        assert_eq!(vm.last_error(), None);
        assert_eq!(vm.f_stack().as_slice(), [1.0, 1.0]);
    }

    #[test]
    fn test_fover() {
        let vm = &mut VM::new(16);
        match vm.f_stack().push2(1.0, 2.0) {
            Err(_) => assert!(true, "Floating point stack overflow"),
            Ok(()) => {}
        };
        vm.fover();
        assert_eq!(vm.last_error(), None);
        assert_eq!(vm.f_stack().as_slice(), [1.0, 2.0, 1.0]);
    }

    #[test]
    fn test_frot() {
        let vm = &mut VM::new(16);
        match vm.f_stack().push3(1.0, 2.0, 3.0) {
            Err(_) => assert!(true, "Floating point stack overflow"),
            Ok(()) => {}
        };
        vm.frot();
        assert_eq!(vm.last_error(), None);
        assert_eq!(vm.f_stack().as_slice(), [2.0, 3.0, 1.0]);
    }

    #[test]
    fn test_fplus_fminus_fstar_fslash() {
        let vm = &mut VM::new(16);
        vm.set_source("9.0 10.0 f+ 11.0 f- 12.0 f* 13.0 f/");
        vm.evaluate();
        assert_eq!(vm.last_error(), None);
        assert_eq!(vm.f_stack().len(), 1);
        assert!(match vm.f_stack().pop() {
            Ok(t) => t > 7.384614 && t < 7.384616,
            Err(_) => false,
        });
    }

    #[test]
    fn test_f_zero_less_than() {
        let vm = &mut VM::new(16);
        vm.set_source("0.0 f0<   0.1 f0<   -0.1 f0<");
        vm.evaluate();
        assert_eq!(vm.last_error(), None);
        assert_eq!(vm.s_stack().len(), 3);
        assert_eq!(vm.s_stack().pop(), Ok(-1));
        assert_eq!(vm.s_stack().pop(), Ok(0));
        assert_eq!(vm.s_stack().pop(), Ok(0));
        assert_eq!(vm.f_stack().as_slice(), []);
    }

    #[test]
    fn test_f_zero_equals() {
        let vm = &mut VM::new(16);
        vm.set_source("0.0 f0=   0.1 f0=   -0.1 f0=");
        vm.evaluate();
        assert_eq!(vm.last_error(), None);
        assert_eq!(vm.s_stack().len(), 3);
        assert_eq!(vm.s_stack().pop(), Ok(0));
        assert_eq!(vm.s_stack().pop(), Ok(0));
        assert_eq!(vm.s_stack().pop(), Ok(-1));
        assert_eq!(vm.f_stack().as_slice(), []);
    }

    #[test]
    fn test_f_less_than() {
        let vm = &mut VM::new(16);
        vm.set_source("0.0 0.0 f<   0.1 0.0 f<   -0.1 0.0 f<");
        vm.evaluate();
        assert_eq!(vm.last_error(), None);
        assert_eq!(vm.s_stack().len(), 3);
        assert_eq!(vm.s_stack().pop(), Ok(-1));
        assert_eq!(vm.s_stack().pop(), Ok(0));
        assert_eq!(vm.s_stack().pop(), Ok(0));
        assert_eq!(vm.f_stack().as_slice(), []);
    }

    #[test]
    fn test_fproximate() {
        let vm = &mut VM::new(16);
        vm.set_source("0.1 0.1 0.0 f~   0.1 0.10000000001 0.0 f~");
        vm.evaluate();
        assert_eq!(vm.last_error(), None);
        assert_eq!(vm.s_stack().len(), 2);
        assert_eq!(vm.s_stack().pop(), Ok(0));
        assert_eq!(vm.s_stack().pop(), Ok(-1));
        assert_eq!(vm.f_stack().as_slice(), []);
        vm.s_stack().clear();
        vm.set_source("0.1 0.1 0.001 f~   0.1 0.109 0.01 f~   0.1 0.111  0.01 f~");
        vm.evaluate();
        assert_eq!(vm.last_error(), None);
        assert_eq!(vm.s_stack().len(), 3);
        assert_eq!(vm.s_stack().pop(), Ok(0));
        assert_eq!(vm.s_stack().pop(), Ok(-1));
        assert_eq!(vm.s_stack().pop(), Ok(-1));
        assert_eq!(vm.f_stack().as_slice(), []);
        vm.s_stack().clear();
        vm.set_source("0.1 0.1 -0.001 f~   0.1 0.109 -0.1 f~   0.1 0.109  -0.01 f~");
        vm.evaluate();
        assert_eq!(vm.last_error(), None);
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
        vm.set_source("0 n>f -1 n>f 1 n>f");
        vm.evaluate();
        assert_eq!(vm.last_error(), None);
        assert_eq!(vm.f_stack().as_slice(), [0.0, -1.0, 1.0]);
    }

    #[test]
    fn test_flit_and_compile_float() {
        let vm = &mut VM::new(16);
        vm.set_source(": test 1.0 2.0 ; test");
        vm.evaluate();
        assert_eq!(vm.last_error(), None);
        assert_eq!(vm.f_stack().as_slice(), [1.0, 2.0]);
    }
}
