use core::VM;
use core::Heap;

use exception::Exception::{
    StackUnderflow,
    StackOverflow,
    FloatingPointStackOverflow,
    FloatingPointStackUnderflow,
};

pub trait Float {
    fn add_float(&mut self);
    fn flit(&mut self);
    fn p_fconst(&mut self);
    fn fvariable(&mut self);
    fn fconstant(&mut self);
    fn ffetch(&mut self);
    fn fstore(&mut self);
    fn fabs(&mut self);
    fn fsin(&mut self);
    fn fcos(&mut self);
    fn ftan(&mut self);
    fn fasin(&mut self);
    fn facos(&mut self);
    fn fatan(&mut self);
    fn fatan2(&mut self);
    fn fsqrt(&mut self);
    fn fswap(&mut self);
    fn fnip(&mut self);
    fn fdup(&mut self);
    fn fdrop(&mut self);
    fn frot(&mut self);
    fn fover(&mut self);
    fn n_to_f(&mut self);
    fn fplus(&mut self);
    fn fminus(&mut self);
    fn fstar(&mut self);
    fn fslash(&mut self);
    fn fproximate(&mut self);
    fn f_zero_less_than(&mut self);
    fn f_zero_equals(&mut self);
    fn f_less_than(&mut self);
}

impl Float for VM {
    fn add_float(&mut self) {
        self.add_primitive("flit", VM::flit);
        self.add_primitive ("fconstant", VM::fconstant);
        self.add_primitive ("fvariable", VM::fvariable);
        self.add_primitive ("f!", VM::fstore);
        self.add_primitive ("f@", VM::ffetch);
        self.add_primitive ("fabs", VM::fabs);
        self.add_primitive ("fsin", VM::fsin);
        self.add_primitive ("fcos", VM::fcos);
        self.add_primitive ("ftan", VM::ftan);
        self.add_primitive ("fasin", VM::fasin);
        self.add_primitive ("facos", VM::facos);
        self.add_primitive ("fatan", VM::fatan);
        self.add_primitive ("fatan2", VM::fatan2);
        self.add_primitive ("fsqrt", VM::fsqrt);
        self.add_primitive ("fdrop", VM::fdrop);
        self.add_primitive ("fdup", VM::fdup);
        self.add_primitive ("fswap", VM::fswap);
        self.add_primitive ("fnip", VM::fnip);
        self.add_primitive ("frot", VM::frot);
        self.add_primitive ("fover", VM::fover);
        self.add_primitive ("n>f", VM::n_to_f);
        self.add_primitive ("f+", VM::fplus);
        self.add_primitive ("f-", VM::fminus);
        self.add_primitive ("f*", VM::fstar);
        self.add_primitive ("f/", VM::fslash);
        self.add_primitive ("f~", VM::fproximate);
        self.add_primitive ("f0<", VM::f_zero_less_than);
        self.add_primitive ("f0=", VM::f_zero_equals);
        self.add_primitive ("f<", VM::f_less_than);
        self.idx_flit = self.find("flit").expect("flit defined");
    }

    fn flit(&mut self) {
        let v = self.f_heap.get_f64(self.s_heap[self.instruction_pointer] as usize);
        match self.f_stack.push (v) {
            Some(_) => self.abort_with_error(FloatingPointStackOverflow),
            None => self.instruction_pointer = self.instruction_pointer + 1
        };
    }

    fn p_fconst(&mut self) {
        let dfa = self.word_list[self.word_pointer()].dfa();
        let v = self.f_heap.get_f64(self.s_heap[dfa] as usize);
        match self.f_stack.push(v) {
            Some(_) => self.abort_with_error(FloatingPointStackOverflow),
            None => {}
        };
    }

    fn fvariable(&mut self) {
        self.define(VM::p_fvar);
        self.s_heap.push(self.f_heap.len() as isize);
        self.f_heap.push_f64(0.0);
    }

    fn fconstant(&mut self) {
        match self.f_stack.pop() {
            Some(v) => {
                self.define(VM::p_fconst);
                self.s_heap.push(self.f_heap.len() as isize);
                self.f_heap.push_f64(v);
            },
            None => self.abort_with_error(FloatingPointStackUnderflow)
        }
    }

// Floating point primitives

    fn ffetch(&mut self) {
        match self.s_stack.pop() {
            Some(t) =>
                match self.f_stack.push(self.f_heap.get_f64(t as usize)) {
                    Some(_) => self.abort_with_error(FloatingPointStackOverflow),
                    None => {}
                },
            None => self.abort_with_error(StackUnderflow)
        };
    }

    fn fstore(&mut self) {
        match self.s_stack.pop() {
            Some(t) =>
                match self.f_stack.pop() {
                    Some(n) => self.f_heap.put_f64(t as usize, n),
                    None => self.abort_with_error(StackUnderflow)
                },
            None => self.abort_with_error(StackUnderflow)
        };
    }

    fn fabs(&mut self) {
        match self.f_stack.pop() {
            Some(t) =>
                match self.f_stack.push(t.abs()) {
                    Some(_) => self.abort_with_error(FloatingPointStackOverflow),
                    None => {}
                },
            None => self.abort_with_error(FloatingPointStackUnderflow)
        };
    }

    fn fsin(&mut self) {
        match self.f_stack.pop() {
            Some(t) =>
                match self.f_stack.push(t.sin()) {
                    Some(_) => self.abort_with_error(FloatingPointStackOverflow),
                    None => {}
                },
            None => self.abort_with_error(FloatingPointStackUnderflow)
        };
    }

    fn fcos(&mut self) {
        match self.f_stack.pop() {
            Some(t) =>
                match self.f_stack.push(t.cos()) {
                    Some(_) => self.abort_with_error(FloatingPointStackOverflow),
                    None => {}
                },
            None => self.abort_with_error(FloatingPointStackUnderflow)
        };
    }

    fn ftan(&mut self) {
        match self.f_stack.pop() {
            Some(t) =>
                match self.f_stack.push(t.tan()) {
                    Some(_) => self.abort_with_error(FloatingPointStackOverflow),
                    None => {}
                },
            None => self.abort_with_error(FloatingPointStackUnderflow)
        };
    }

    fn fasin(&mut self) {
        match self.f_stack.pop() {
            Some(t) =>
                match self.f_stack.push(t.asin()) {
                    Some(_) => self.abort_with_error(FloatingPointStackOverflow),
                    None => {}
                },
            None => self.abort_with_error(FloatingPointStackUnderflow)
        };
    }

    fn facos(&mut self) {
        match self.f_stack.pop() {
            Some(t) =>
                match self.f_stack.push(t.acos()) {
                    Some(_) => self.abort_with_error(FloatingPointStackOverflow),
                    None => {}
                },
            None => self.abort_with_error(FloatingPointStackUnderflow)
        };
    }

    fn fatan(&mut self) {
        match self.f_stack.pop() {
            Some(t) =>
                match self.f_stack.push(t.atan()) {
                    Some(_) => self.abort_with_error(FloatingPointStackOverflow),
                    None => {}
                },
            None => self.abort_with_error(FloatingPointStackUnderflow)
        };
    }

    fn fatan2(&mut self) {
        match self.f_stack.pop() {
            Some(t) => {
                match self.f_stack.pop() {
                    Some(n) =>
                        match self.f_stack.push(n.atan2(t)) {
                            Some(_) => self.abort_with_error(FloatingPointStackOverflow),
                            None => {}
                        },
                    None => self.abort_with_error(FloatingPointStackUnderflow)
                }
            },
            None => self.abort_with_error(FloatingPointStackUnderflow)
        };
    }

    fn fsqrt(&mut self) {
        match self.f_stack.pop() {
            Some(t) =>
                match self.f_stack.push(t.sqrt()) {
                    Some(_) => self.abort_with_error(FloatingPointStackOverflow),
                    None => {}
                },
            None => self.abort_with_error(FloatingPointStackUnderflow)
        };
    }

    fn fswap(&mut self) {
        match self.f_stack.pop() {
            Some(t) =>
                match self.f_stack.pop() {
                    Some(n) =>
                        match self.f_stack.push2(t, n) {
                            Some(_) => self.abort_with_error(FloatingPointStackOverflow),
                            None => {}
                        },
                    None => self.abort_with_error(FloatingPointStackUnderflow)
                },
            None => self.abort_with_error(FloatingPointStackUnderflow)
        }
    }

    fn fnip(&mut self) {
        match self.f_stack.pop() {
            Some(t) =>
                match self.f_stack.pop() {
                    Some(_) =>
                        match self.f_stack.push(t) {
                            Some(_) => self.abort_with_error(FloatingPointStackOverflow),
                            None => {}
                        },
                    None => self.abort_with_error(FloatingPointStackUnderflow)
                },
            None => self.abort_with_error(FloatingPointStackUnderflow)
        }
    }

    fn fdup(&mut self) {
        match self.f_stack.pop() {
            Some(t) => {
                match self.f_stack.push2(t, t) {
                    Some(_) => self.abort_with_error(FloatingPointStackOverflow),
                    None => {}
                };
            },
            None => self.abort_with_error(FloatingPointStackUnderflow)
        };
    }

    fn fdrop(&mut self) {
        match self.f_stack.pop() {
            Some(_) => { },
            None => self.abort_with_error(FloatingPointStackUnderflow)
        };
    }

    fn frot(&mut self) {
        match self.f_stack.pop() {
            Some(x3) =>
                match self.f_stack.pop() {
                    Some(x2) =>
                        match self.f_stack.pop() {
                            Some(x1) =>
                                match self.f_stack.push3(x2, x3, x1) {
                                    Some(_) => self.abort_with_error(FloatingPointStackOverflow),
                                    None => {}
                                },
                            None => self.abort_with_error(FloatingPointStackUnderflow)
                        },
                    None => self.abort_with_error(FloatingPointStackUnderflow)
                },
            None => self.abort_with_error(FloatingPointStackUnderflow)
        }
    }

    fn fover(&mut self) {
        match self.f_stack.pop() {
            Some(t) =>
                match self.f_stack.pop() {
                    Some(n) =>
                        match self.f_stack.push3(n, t, n) {
                            Some(_) => self.abort_with_error(FloatingPointStackOverflow),
                            None => {}
                        },
                    None => self.abort_with_error(FloatingPointStackUnderflow)
                },
            None => self.abort_with_error(FloatingPointStackUnderflow)
        }
    }

    fn n_to_f(&mut self) {
        match self.s_stack.pop() {
            Some(t) =>
                match self.f_stack.push(t as f64) {
                    Some(_) => self.abort_with_error(FloatingPointStackOverflow),
                    None => {}
                },
            None => self.abort_with_error(FloatingPointStackUnderflow)
        }
    }

    fn fplus(&mut self) {
        match self.f_stack.pop() {
            Some(t) =>
                match self.f_stack.pop() {
                    Some(n) =>
                        match self.f_stack.push(n+t) {
                            Some(_) => self.abort_with_error(FloatingPointStackOverflow),
                            None => {}
                        },
                    None => self.abort_with_error(FloatingPointStackUnderflow)
                },
            None => self.abort_with_error(FloatingPointStackUnderflow)
        }
    }

    fn fminus(&mut self) {
        match self.f_stack.pop() {
            Some(t) =>
                match self.f_stack.pop() {
                    Some(n) =>
                        match self.f_stack.push(n-t) {
                            Some(_) => self.abort_with_error(FloatingPointStackOverflow),
                            None => {}
                        },
                    None => self.abort_with_error(FloatingPointStackUnderflow)
                },
            None => self.abort_with_error(FloatingPointStackUnderflow)
        }
    }

    fn fstar(&mut self) {
        match self.f_stack.pop() {
            Some(t) =>
                match self.f_stack.pop() {
                    Some(n) =>
                        match self.f_stack.push(n*t) {
                            Some(_) => self.abort_with_error(FloatingPointStackOverflow),
                            None => {}
                        },
                    None => self.abort_with_error(FloatingPointStackUnderflow)
                },
            None => self.abort_with_error(FloatingPointStackUnderflow)
        }
    }

    fn fslash(&mut self) {
        match self.f_stack.pop() {
            Some(t) =>
                match self.f_stack.pop() {
                    Some(n) =>
                        match self.f_stack.push(n/t) {
                            Some(_) => self.abort_with_error(FloatingPointStackOverflow),
                            None => {}
                        },
                    None => self.abort_with_error(FloatingPointStackUnderflow)
                },
            None => self.abort_with_error(FloatingPointStackUnderflow)
        }
    }

    fn fproximate(&mut self) {
        match self.f_stack.pop() {
            Some(x3) =>
                match self.f_stack.pop() {
                    Some(x2) =>
                        match self.f_stack.pop() {
                            Some(x1) => {
                                if x3 > 0.0 {
                                    self.s_stack.push(if (x1-x2).abs() < x3 {-1} else {0});
                                } else if x3 == 0.0 {
                                    self.s_stack.push(if x1==x2 {-1} else {0});
                                } else {
                                    self.s_stack.push(if (x1-x2).abs() < (x3.abs()*(x1.abs() + x2.abs())) {-1} else {0});
                                }
                            },
                            None => self.abort_with_error(FloatingPointStackUnderflow)
                        },
                    None => self.abort_with_error(FloatingPointStackUnderflow)
                },
            None => self.abort_with_error(FloatingPointStackUnderflow)
        }
    }

    fn f_zero_less_than(&mut self) {
        match self.f_stack.pop() {
            Some(t) =>
                match self.s_stack.push(if t<0.0 {-1} else {0}) {
                    Some(_) => self.abort_with_error(StackOverflow),
                    None => {}
                },
            None => self.abort_with_error(FloatingPointStackUnderflow)
        }
    }

    fn f_zero_equals(&mut self) {
        match self.f_stack.pop() {
            Some(t) =>
                match self.s_stack.push(if t==0.0 {-1} else {0}) {
                    Some(_) => self.abort_with_error(StackOverflow),
                    None => {}
                },
            None => self.abort_with_error(FloatingPointStackUnderflow)
        }
    }

    fn f_less_than(&mut self) {
        match self.f_stack.pop() {
            Some(t) =>
                match self.f_stack.pop() {
                    Some(n) =>
                        match self.s_stack.push(if n<t {-1} else {0}) {
                            Some(_) => self.abort_with_error(StackOverflow),
                            None => {}
                        },
                    None => self.abort_with_error(FloatingPointStackUnderflow)
                },
            None => self.abort_with_error(FloatingPointStackUnderflow)
        }
    }

}

#[cfg(test)]
mod tests {
    use core::VM;
    use super::Float;

    #[test]
    fn test_evaluate_f64 () {
        let vm = &mut VM::new();
        vm.add_float();
        vm.set_source("1.0 2.5");
        vm.evaluate();
        assert_eq!(vm.f_stack.len(), 2);
        assert!(0.99999 < vm.f_stack.as_slice()[0]);
        assert!(vm.f_stack.as_slice()[0] < 1.00001);
        assert!(2.49999 < vm.f_stack.as_slice()[1]);
        assert!(vm.f_stack.as_slice()[1] < 2.50001);
        assert_eq!(vm.error_code, 0);
    }

    #[test]
    fn test_fconstant () {
        let vm = &mut VM::new();
        vm.add_float();
        vm.set_source("1.1 fconstant x x x");
        vm.evaluate();
        assert_eq!(vm.f_stack.as_slice(), [1.1, 1.1]);
        assert_eq!(vm.error_code, 0);
    }

    #[test]
    fn test_fvariable_and_fstore_ffetch () {
        let vm = &mut VM::new();
        vm.add_float();
        vm.set_source("fvariable fx  fx f@  3.3 fx f!  fx f@");
        vm.evaluate();
        assert_eq!(vm.f_stack.as_slice(), [0.0, 3.3]);
        assert_eq!(vm.error_code, 0);
    }

    #[test]
    fn test_fabs () {
        let vm = &mut VM::new();
        vm.add_float();
        vm.set_source("-3.14 fabs");
        vm.evaluate();
        assert_eq!(vm.f_stack.len(), 1);
        assert!(match vm.f_stack.pop() {
            Some(t) => {
                t > 3.13999 && t < 3.14001
            },
            None => false
        });
        assert_eq!(vm.error_code, 0);
    }

    #[test]
    fn test_fsin () {
        let vm = &mut VM::new();
        vm.add_float();
        vm.set_source("3.14 fsin");
        vm.evaluate();
        assert_eq!(vm.f_stack.len(), 1);
        assert!(match vm.f_stack.pop() {
            Some(t) => {
                t > 0.0015925 && t < 0.0015927
            },
            None => false
        });
        assert_eq!(vm.error_code, 0);
    }

    #[test]
    fn test_fcos () {
        let vm = &mut VM::new();
        vm.add_float();
        vm.set_source("3.0 fcos");
        vm.evaluate();
        assert_eq!(vm.f_stack.len(), 1);
        assert!(match vm.f_stack.pop() {
            Some(t) => {
                t > -0.989993 && t < -0.989991
            },
            None => false
        });
        assert_eq!(vm.error_code, 0);
    }

    #[test]
    fn test_ftan () {
        let vm = &mut VM::new();
        vm.add_float();
        vm.set_source("3.0 ftan");
        vm.evaluate();
        assert_eq!(vm.f_stack.len(), 1);
        assert!(match vm.f_stack.pop() {
            Some(t) => {
                t > -0.142547 && t < -0.142545
            },
            None => false
        });
        assert_eq!(vm.error_code, 0);
    }

    #[test]
    fn test_fasin () {
        let vm = &mut VM::new();
        vm.add_float();
        vm.set_source("0.3 fasin");
        vm.evaluate();
        assert_eq!(vm.f_stack.len(), 1);
        assert!(match vm.f_stack.pop() {
            Some(t) => {
                t > 0.304691 && t < 0.304693
            },
            None => false
        });
        assert_eq!(vm.error_code, 0);
    }

    #[test]
    fn test_facos () {
        let vm = &mut VM::new();
        vm.add_float();
        vm.set_source("0.3 facos");
        vm.evaluate();
        assert_eq!(vm.f_stack.len(), 1);
        assert!(match vm.f_stack.pop() {
            Some(t) => {
                t > 1.266102 && t < 1.266104
            },
            None => false
        });
        assert_eq!(vm.error_code, 0);
    }

    #[test]
    fn test_fatan () {
        let vm = &mut VM::new();
        vm.add_float();
        vm.set_source("0.3 fatan");
        vm.evaluate();
        assert_eq!(vm.f_stack.len(), 1);
        assert!(match vm.f_stack.pop() {
            Some(t) => {
                t > 0.291455 && t < 0.291457
            },
            None => false
        });
        assert_eq!(vm.error_code, 0);
    }

    #[test]
    fn test_fatan2 () {
        let vm = &mut VM::new();
        vm.add_float();
        vm.set_source("3.0 4.0 fatan2");
        vm.evaluate();
        assert_eq!(vm.f_stack.len(), 1);
        assert!(match vm.f_stack.pop() {
            Some(t) => {
                t > 0.643500  && t < 0.643502
            },
            None => false
        });
        assert_eq!(vm.error_code, 0);
    }

    #[test]
    fn test_fsqrt () {
        let vm = &mut VM::new();
        vm.add_float();
        vm.set_source("0.3 fsqrt");
        vm.evaluate();
        assert_eq!(vm.f_stack.len(), 1);
        assert!(match vm.f_stack.pop() {
            Some(t) => {
                t > 0.547721 && t < 0.547723 
            },
            None => false
        });
        assert_eq!(vm.error_code, 0);
    }

    #[test]
    fn test_fdrop() {
        let vm = &mut VM::new();
        vm.add_float();
        match vm.f_stack.push(1.0) {
            Some(_) => assert!(true, "Floating point stack overflow"),
            None => {}
        }
        vm.fdrop();
        assert_eq!(vm.f_stack.as_slice(), []);
        assert_eq!(vm.error_code, 0);
    }

    #[test]
    fn test_fnip() {
        let vm = &mut VM::new();
        vm.add_float();
        match vm.f_stack.push2(1.0, 2.0) {
            Some(_) => assert!(true, "Floating point stack overflow"),
            None => {}
        };
        vm.fnip();
        assert_eq!(vm.f_stack.as_slice(), [2.0]);
        assert_eq!(vm.error_code, 0);
    }

    #[test]
    fn test_fswap () {
        let vm = &mut VM::new();
        vm.add_float();
        match vm.f_stack.push2(1.0, 2.0) {
            Some(_) => assert!(true, "Floating point stack overflow"),
            None => {}
        };
        vm.fswap();
        assert_eq!(vm.f_stack.as_slice(), [2.0,1.0]);
        assert_eq!(vm.error_code, 0);
    }

    #[test]
    fn test_fdup () {
        let vm = &mut VM::new();
        vm.add_float();
        match vm.f_stack.push(1.0) {
            Some(_) => assert!(true, "Floating point stack overflow"),
            None => {}
        };
        vm.fdup();
        assert_eq!(vm.f_stack.as_slice(), [1.0, 1.0]);
        assert_eq!(vm.error_code, 0);
    }

    #[test]
    fn test_fover () {
        let vm = &mut VM::new();
        vm.add_float();
        match vm.f_stack.push2(1.0, 2.0) {
            Some(_) => assert!(true, "Floating point stack overflow"),
            None => {}
        };
        vm.fover();
        assert_eq!(vm.f_stack.as_slice(), [1.0,2.0,1.0]);
        assert_eq!(vm.error_code, 0);
    }

    #[test]
    fn test_frot () {
        let vm = &mut VM::new();
        vm.add_float();
        match vm.f_stack.push3(1.0, 2.0, 3.0) {
            Some(_) => assert!(true, "Floating point stack overflow"),
            None => {}
        };
        vm.frot();
        assert_eq!(vm.f_stack.as_slice(), [2.0, 3.0, 1.0]);
        assert_eq!(vm.error_code, 0);
    }

    #[test]
    fn test_fplus_fminus_fstar_fslash () {
        let vm = &mut VM::new();
        vm.add_float();
        vm.set_source("9.0 10.0 f+ 11.0 f- 12.0 f* 13.0 f/");
        vm.evaluate();
        assert_eq!(vm.f_stack.len(), 1);
        assert!(match vm.f_stack.pop() {
            Some(t) => {
                t > 7.384614 && t < 7.384616
            },
            None => false
        });
        assert_eq!(vm.error_code, 0);
    }

    #[test]
    fn test_f_zero_less_than () {
        let vm = &mut VM::new();
        vm.add_float();
        vm.set_source("0.0 f0<   0.1 f0<   -0.1 f0<");
        vm.evaluate();
        assert_eq!(vm.s_stack.len(), 3);
        assert_eq!(vm.s_stack.pop(), Some(-1));
        assert_eq!(vm.s_stack.pop(), Some(0));
        assert_eq!(vm.s_stack.pop(), Some(0));
        assert_eq!(vm.f_stack.as_slice(), []);
        assert_eq!(vm.error_code, 0);
    }

    #[test]
    fn test_f_zero_equals () {
        let vm = &mut VM::new();
        vm.add_float();
        vm.set_source("0.0 f0=   0.1 f0=   -0.1 f0=");
        vm.evaluate();
        assert_eq!(vm.s_stack.len(), 3);
        assert_eq!(vm.s_stack.pop(), Some(0));
        assert_eq!(vm.s_stack.pop(), Some(0));
        assert_eq!(vm.s_stack.pop(), Some(-1));
        assert_eq!(vm.f_stack.as_slice(), []);
        assert_eq!(vm.error_code, 0);
    }

    #[test]
    fn test_f_less_than () {
        let vm = &mut VM::new();
        vm.add_float();
        vm.set_source("0.0 0.0 f<   0.1 0.0 f<   -0.1 0.0 f<");
        vm.evaluate();
        assert_eq!(vm.s_stack.len(), 3);
        assert_eq!(vm.s_stack.pop(), Some(-1));
        assert_eq!(vm.s_stack.pop(), Some(0));
        assert_eq!(vm.s_stack.pop(), Some(0));
        assert_eq!(vm.f_stack.as_slice(), []);
        assert_eq!(vm.error_code, 0);
    }

    #[test]
    fn test_fproximate () {
        let vm = &mut VM::new();
        vm.add_float();
        vm.set_source("0.1 0.1 0.0 f~   0.1 0.10000000001 0.0 f~");
        vm.evaluate();
        assert_eq!(vm.s_stack.len(), 2);
        assert_eq!(vm.s_stack.pop(), Some(0));
        assert_eq!(vm.s_stack.pop(), Some(-1));
        assert_eq!(vm.f_stack.as_slice(), []);
        assert_eq!(vm.error_code, 0);
        vm.s_stack.clear();
        vm.set_source("0.1 0.1 0.001 f~   0.1 0.109 0.01 f~   0.1 0.111  0.01 f~");
        vm.evaluate();
        assert_eq!(vm.s_stack.len(), 3);
        assert_eq!(vm.s_stack.pop(), Some(0));
        assert_eq!(vm.s_stack.pop(), Some(-1));
        assert_eq!(vm.s_stack.pop(), Some(-1));
        assert_eq!(vm.f_stack.as_slice(), []);
        assert_eq!(vm.error_code, 0);
        vm.s_stack.clear();
        vm.set_source("0.1 0.1 -0.001 f~   0.1 0.109 -0.1 f~   0.1 0.109  -0.01 f~");
        vm.evaluate();
        assert_eq!(vm.s_stack.len(), 3);
        assert_eq!(vm.s_stack.pop(), Some(0));
        assert_eq!(vm.s_stack.pop(), Some(-1));
        assert_eq!(vm.s_stack.pop(), Some(-1));
        assert_eq!(vm.f_stack.as_slice(), []);
        assert_eq!(vm.error_code, 0);
        vm.s_stack.clear();
    }

    #[test]
    fn test_n_to_f () {
        let vm = &mut VM::new();
        vm.add_float();
        vm.set_source("0 n>f -1 n>f 1 n>f");
        vm.evaluate();
        assert_eq!(vm.f_stack.as_slice(), [0.0, -1.0, 1.0]);
        assert_eq!(vm.error_code, 0);
    }

}
