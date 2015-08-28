use core::VM;

use exception::Exception::{
    StackUnderflow,
    FloatingPointStackUnderflow,
};

pub trait Float {
    fn add_float(&mut self);
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
        self.idx_flit = self.find("flit");
    }

    fn fvariable(&mut self) {
        self.define(VM::p_fvar);
        self.s_heap.push(self.f_heap.len() as isize);
        self.f_heap.push(0.0);
    }

    fn fconstant(&mut self) {
        match self.f_stack.pop() {
            Some(v) => {
                self.define(VM::p_fconst);
                self.s_heap.push(self.f_heap.len() as isize);
                self.f_heap.push(v);
            },
            None => self.abort_with_error(FloatingPointStackUnderflow)
        }
    }

// Floating point primitives

    fn ffetch(&mut self) {
        match self.s_stack.pop() {
            Some(t) => self.f_stack.push(self.f_heap[t as usize]),
            None => self.abort_with_error(StackUnderflow)
        };
    }

    fn fstore(&mut self) {
        match self.s_stack.pop() {
            Some(t) =>
                match self.f_stack.pop() {
                    Some(n) => self.f_heap[t as usize] = n,
                    None => self.abort_with_error(StackUnderflow)
                },
            None => self.abort_with_error(StackUnderflow)
        };
    }

    fn fabs(&mut self) {
        match self.f_stack.pop() {
            Some(t) => self.f_stack.push(t.abs()),
            None => self.abort_with_error(FloatingPointStackUnderflow)
        };
    }

    fn fsin(&mut self) {
        match self.f_stack.pop() {
            Some(t) => self.f_stack.push(t.sin()),
            None => self.abort_with_error(FloatingPointStackUnderflow)
        };
    }

    fn fcos(&mut self) {
        match self.f_stack.pop() {
            Some(t) => self.f_stack.push(t.cos()),
            None => self.abort_with_error(FloatingPointStackUnderflow)
        };
    }

    fn ftan(&mut self) {
        match self.f_stack.pop() {
            Some(t) => self.f_stack.push(t.tan()),
            None => self.abort_with_error(FloatingPointStackUnderflow)
        };
    }

    fn fasin(&mut self) {
        match self.f_stack.pop() {
            Some(t) => self.f_stack.push(t.asin()),
            None => self.abort_with_error(FloatingPointStackUnderflow)
        };
    }

    fn facos(&mut self) {
        match self.f_stack.pop() {
            Some(t) => self.f_stack.push(t.acos()),
            None => self.abort_with_error(FloatingPointStackUnderflow)
        };
    }

    fn fatan(&mut self) {
        match self.f_stack.pop() {
            Some(t) => self.f_stack.push(t.atan()),
            None => self.abort_with_error(FloatingPointStackUnderflow)
        };
    }

    fn fatan2(&mut self) {
        match self.f_stack.pop() {
            Some(t) => {
                match self.f_stack.pop() {
                    Some(n) => self.f_stack.push(n.atan2(t)),
                    None => self.abort_with_error(FloatingPointStackUnderflow)
                }
            },
            None => self.abort_with_error(FloatingPointStackUnderflow)
        };
    }

    fn fsqrt(&mut self) {
        match self.f_stack.pop() {
            Some(t) => self.f_stack.push(t.sqrt()),
            None => self.abort_with_error(FloatingPointStackUnderflow)
        };
    }

    fn fswap(&mut self) {
        match self.f_stack.pop() {
            Some(t) =>
                match self.f_stack.pop() {
                    Some(n) => { self.f_stack.push(t); self.f_stack.push(n); },
                    None => self.abort_with_error(FloatingPointStackUnderflow)
                },
            None => self.abort_with_error(FloatingPointStackUnderflow)
        }
    }

    fn fnip(&mut self) {
        match self.f_stack.pop() {
            Some(t) =>
                match self.f_stack.pop() {
                    Some(_) => self.f_stack.push(t),
                    None => self.abort_with_error(FloatingPointStackUnderflow)
                },
            None => self.abort_with_error(FloatingPointStackUnderflow)
        }
    }

    fn fdup(&mut self) {
        match self.f_stack.pop() {
            Some(t) => {
                self.f_stack.push(t);
                self.f_stack.push(t);
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
                            Some(x1) => {
                                self.f_stack.push(x2);
                                self.f_stack.push(x3);
                                self.f_stack.push(x1);
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
                    Some(n) => {
                        self.f_stack.push(n);
                        self.f_stack.push(t);
                        self.f_stack.push(n);
                    },
                    None => self.abort_with_error(FloatingPointStackUnderflow)
                },
            None => self.abort_with_error(FloatingPointStackUnderflow)
        }
    }

    fn n_to_f(&mut self) {
        match self.s_stack.pop() {
            Some(t) => self.f_stack.push(t as f64),
            None => self.abort_with_error(FloatingPointStackUnderflow)
        }
    }

    fn fplus(&mut self) {
        match self.f_stack.pop() {
            Some(t) =>
                match self.f_stack.pop() {
                    Some(n) => self.f_stack.push(n+t),
                    None => self.abort_with_error(FloatingPointStackUnderflow)
                },
            None => self.abort_with_error(FloatingPointStackUnderflow)
        }
    }

    fn fminus(&mut self) {
        match self.f_stack.pop() {
            Some(t) =>
                match self.f_stack.pop() {
                    Some(n) => self.f_stack.push(n-t),
                    None => self.abort_with_error(FloatingPointStackUnderflow)
                },
            None => self.abort_with_error(FloatingPointStackUnderflow)
        }
    }

    fn fstar(&mut self) {
        match self.f_stack.pop() {
            Some(t) =>
                match self.f_stack.pop() {
                    Some(n) => self.f_stack.push(n*t),
                    None => self.abort_with_error(FloatingPointStackUnderflow)
                },
            None => self.abort_with_error(FloatingPointStackUnderflow)
        }
    }

    fn fslash(&mut self) {
        match self.f_stack.pop() {
            Some(t) =>
                match self.f_stack.pop() {
                    Some(n) => self.f_stack.push(n/t),
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
            Some(t) =>self.s_stack.push(if t<0.0 {-1} else {0}),
            None => self.abort_with_error(FloatingPointStackUnderflow)
        }
    }

    fn f_zero_equals(&mut self) {
        match self.f_stack.pop() {
            Some(t) =>self.s_stack.push(if t==0.0 {-1} else {0}),
            None => self.abort_with_error(FloatingPointStackUnderflow)
        }
    }

    fn f_less_than(&mut self) {
        match self.f_stack.pop() {
            Some(t) =>
                match self.f_stack.pop() {
                    Some(n) => self.s_stack.push(if n<t {-1} else {0}),
                    None => self.abort_with_error(FloatingPointStackUnderflow)
                },
            None => self.abort_with_error(FloatingPointStackUnderflow)
        }
    }

}
