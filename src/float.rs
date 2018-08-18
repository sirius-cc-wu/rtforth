use core::Core;
use dataspace::DataSpace;
use exception::Exception::InvalidMemoryAddress;
use std::mem;
use std::f64::consts::PI;
use {FALSE, TRUE};

pub trait Float: Core {
    fn add_float(&mut self) {
        self.add_primitive("fconstant", Float::fconstant);
        self.add_primitive("float+", Float::float_plus);
        self.add_primitive("floats", Float::floats);
        self.add_primitive("faligned", Float::faligned);
        self.add_primitive("falign", Float::falign);
        self.add_primitive("pi", Float::pi);
        self.add_primitive("f!", Float::fstore);
        self.add_primitive("f@", Float::ffetch);
        self.add_primitive("fabs", Float::fabs);
        self.add_primitive("fsin", Float::fsin);
        self.add_primitive("fcos", Float::fcos);
        self.add_primitive("ftan", Float::ftan);
        self.add_primitive("fsincos", Float::fsincos);
        self.add_primitive("fasin", Float::fasin);
        self.add_primitive("facos", Float::facos);
        self.add_primitive("fatan", Float::fatan);
        self.add_primitive("fatan2", Float::fatan2);
        self.add_primitive("fsqrt", Float::fsqrt);
        self.add_primitive("fdrop", Float::fdrop);
        self.add_primitive("fdup", Float::fdup);
        self.add_primitive("fswap", Float::fswap);
        self.add_primitive("fnip", Float::fnip);
        self.add_primitive("fover", Float::fover);
        self.add_primitive("frot", Float::frot);
        self.add_primitive("fpick", Float::fpick);
        self.add_primitive("s>f", Float::s_to_f);
        self.add_primitive("f>s", Float::f_to_s);
        self.add_primitive("f+", Float::fplus);
        self.add_primitive("f-", Float::fminus);
        self.add_primitive("f*", Float::fstar);
        self.add_primitive("f/", Float::fslash);
        self.add_primitive("f**", Float::fpowf);
        self.add_primitive("f~", Float::fproximate);
        self.add_primitive("f0<", Float::f_zero_less_than);
        self.add_primitive("f0=", Float::f_zero_equals);
        self.add_primitive("f<", Float::f_less_than);
        self.add_primitive("fmin", Float::fmin);
        self.add_primitive("fmax", Float::fmax);
        self.add_primitive("floor", Float::floor);
        self.add_primitive("fround", Float::fround);
        self.add_primitive("fceil", Float::fceil);
        self.add_primitive("fnegate", Float::fnegate);
    }

    // Defining words

    primitive!{fn p_fconst(&mut self) {
        let wp = self.state().word_pointer();
        let pos = DataSpace::aligned_f64(self.wordlist()[wp].dfa());
        let v = self.data_space().get_f64(pos);
        self.f_stack().push(v);
    }}

    primitive!{fn fconstant(&mut self) {
        let v = self.f_stack().pop();
        self.define(Float::p_fconst, Core::compile_fconst);
        self.data_space().align_f64();
        self.data_space().compile_f64(v);
    }}

    /// Run-time: ( a-addr1 -- a-addr2 )
    ///
    /// Add the size in address units of a float to `a-addr1`, giving `a-addr2`.
    primitive!{fn float_plus(&mut self) {
        let v = self.s_stack().pop();
        self.s_stack().push(v + mem::size_of::<f64>() as isize);
    }}

    /// Run-time: ( n1 -- n2 )
    ///
    /// `n2` is the size in address units of `n1` floats.
    primitive!{fn floats(&mut self) {
        let v = self.s_stack().pop();
        self.s_stack().push(v * mem::size_of::<f64>() as isize);
    }}

    /// Run-time: ( addr -- a-addr )
    ///
    /// Return `a-addr`, the first float-aligned address greater than or equal to `addr`.
    primitive!{fn faligned(&mut self) {
        let pos = self.s_stack().pop();
        let pos = DataSpace::aligned_f64(pos as usize);
        self.s_stack().push(pos as isize);
    }}

    /// Run-time: ( -- )
    ///
    /// If the data-space pointer is not float-aligned, reserve enough space to align it.
    primitive!{fn falign(&mut self) {
        self.data_space().align_f64();
    }}

    primitive!{fn pi(&mut self) {
        self.f_stack().push(PI);
    }}

    // Floating point primitives

    primitive!{fn ffetch(&mut self) {
        let t = DataSpace::aligned_f64(self.s_stack().pop() as usize);
        if (t + mem::size_of::<f64>()) <= self.data_space().capacity() {
            let value = self.data_space().get_f64(t);
            self.f_stack().push(value);
        } else {
            self.abort_with(InvalidMemoryAddress);
        }
    }}

    primitive!{fn fstore(&mut self) {
        let t = DataSpace::aligned_f64(self.s_stack().pop() as usize);
        let n = self.f_stack().pop();
        if (t as usize + mem::size_of::<f64>()) <= self.data_space().capacity() {
            self.data_space().put_f64(n, t as usize);
        } else {
            self.abort_with(InvalidMemoryAddress);
        }
    }}

    primitive!{fn fabs(&mut self) {
        let t = self.f_stack().pop();
        self.f_stack().push(t.abs());
    }}

    primitive!{fn fsin(&mut self) {
        let t = self.f_stack().pop();
        self.f_stack().push(t.sin());
    }}

    primitive!{fn fcos(&mut self) {
        let t = self.f_stack().pop();
        self.f_stack().push(t.cos());
    }}

    primitive!{fn ftan(&mut self) {
        let t = self.f_stack().pop();
        self.f_stack().push(t.tan());
    }}

    primitive!{fn fsincos(&mut self) {
        let t = self.f_stack().pop();
        let (s, c) = t.sin_cos();
        self.f_stack().push2(s, c);
    }}

    primitive!{fn fasin(&mut self) {
        let t = self.f_stack().pop();
        self.f_stack().push(t.asin());
    }}

    primitive!{fn facos(&mut self) {
        let t = self.f_stack().pop();
        self.f_stack().push(t.acos());
    }}

    primitive!{fn fatan(&mut self) {
        let t = self.f_stack().pop();
        self.f_stack().push(t.atan());
    }}

    primitive!{fn fatan2(&mut self) {
        let t = self.f_stack().pop();
        let n = self.f_stack().pop();
        self.f_stack().push(n.atan2(t));
    }}

    primitive!{fn fsqrt(&mut self) {
        let t = self.f_stack().pop();
        self.f_stack().push(t.sqrt());
    }}

    primitive!{fn fswap(&mut self) {
        let t = self.f_stack().pop();
        let n = self.f_stack().pop();
        self.f_stack().push2(t, n);
    }}

    primitive!{fn fnip(&mut self) {
        let t = self.f_stack().pop();
        let _ = self.f_stack().pop();
        self.f_stack().push(t);
    }}

    primitive!{fn fdup(&mut self) {
        let t = self.f_stack().pop();
        self.f_stack().push2(t, t);
    }}

    primitive!{fn fdrop(&mut self) {
        let _ = self.f_stack().pop();
    }}

    primitive!{fn frot(&mut self) {
        let x3 = self.f_stack().pop();
        let x2 = self.f_stack().pop();
        let x1 = self.f_stack().pop();
        self.f_stack().push3(x2, x3, x1);
    }}

    primitive!{fn fover(&mut self) {
        let t = self.f_stack().pop();
        let n = self.f_stack().pop();
        self.f_stack().push3(n, t, n);
    }}

    /// Place a copy of the nth floating point stack entry on top of the floating point stack. `fpick ( n -- ) ( F: ... -- x )`
    ///
    /// `0 fpick` is equivalent to `fdup`.
    primitive!{fn fpick(&mut self) {
        let t = self.s_stack().pop() as u8;
        let len = self.f_stack().len;
        let x = self.f_stack()[len.wrapping_sub(t.wrapping_add(1))];
        self.f_stack().push(x);
    }}

    primitive!{fn s_to_f(&mut self) {
        let t = self.s_stack().pop();
        self.f_stack().push(t as f64);
    }}

    primitive!{fn f_to_s(&mut self) {
        let t = self.f_stack().pop();
        self.s_stack().push(t as isize);
    }}

    primitive!{fn fplus(&mut self) {
        let t = self.f_stack().pop();
        let n = self.f_stack().pop();
        self.f_stack().push(n + t);
    }}

    primitive!{fn fminus(&mut self) {
        let t = self.f_stack().pop();
        let n = self.f_stack().pop();
        self.f_stack().push(n - t);
    }}

    primitive!{fn fstar(&mut self) {
        let t = self.f_stack().pop();
        let n = self.f_stack().pop();
        self.f_stack().push(n * t);
    }}

    primitive!{fn fslash(&mut self) {
        let t = self.f_stack().pop();
        let n = self.f_stack().pop();
        self.f_stack().push(n / t);
    }}

    primitive!{fn fpowf(&mut self) {
        let t = self.f_stack().pop();
        let n = self.f_stack().pop();
        self.f_stack().push(n.powf(t));
    }}

    primitive!{fn fproximate(&mut self) {
        let (x1, x2, x3) = self.f_stack().pop3();
        if x3 > 0.0 {
            self.s_stack()
                .push(if (x1 - x2).abs() < x3 { TRUE } else { FALSE });
        } else if x3 == 0.0 {
            self.s_stack().push(if x1 == x2 { TRUE } else { FALSE });
        } else {
            self.s_stack()
                .push(if (x1 - x2).abs() < (x3.abs() * (x1.abs() + x2.abs())) {
                          TRUE
                      } else {
                          FALSE
                      });
        }
    }}

    primitive!{fn f_zero_less_than(&mut self) {
        let t = self.f_stack().pop();
        self.s_stack().push(if t < 0.0 { TRUE } else { FALSE });
    }}

    primitive!{fn f_zero_equals(&mut self) {
        let t = self.f_stack().pop();
        self.s_stack().push(if t == 0.0 { TRUE } else { FALSE });
    }}

    primitive!{fn f_less_than(&mut self) {
        let t = self.f_stack().pop();
        let n = self.f_stack().pop();
        self.s_stack().push(if n < t { TRUE } else { FALSE });
    }}

    primitive!{fn fmin(&mut self) {
        let (n, t) = self.f_stack().pop2();
        self.f_stack().push(t.min(n));
    }}

    primitive!{fn fmax(&mut self) {
        let (n, t) = self.f_stack().pop2();
        self.f_stack().push(t.max(n));
    }}

    primitive!{fn fround(&mut self) {
        let t = self.f_stack().pop();
        self.f_stack().push(t.round());
    }}

    primitive!{fn floor(&mut self) {
        let t = self.f_stack().pop();
        self.f_stack().push(t.floor());
    }}

    primitive!{fn fceil(&mut self) {
        let t = self.f_stack().pop();
        self.f_stack().push(t.ceil());
    }}

    primitive!{fn fnegate(&mut self) {
        let t = self.f_stack().pop();
        self.f_stack().push(-t);
    }}
}

#[cfg(test)]
mod tests {
    use super::Float;
    use core::Core;
    use exception::Exception::UndefinedWord;
    use vm::VM;

    #[test]
    fn test_ans_forth_float() {
        let vm = &mut VM::new(16, 16);
        vm.set_source("1E");
        vm.evaluate();
        assert_eq!(vm.f_stack().len(), 1);
        assert_ulps_eq!(vm.f_stack().pop(), 1.0);
        vm.set_source("1.E");
        vm.evaluate();
        assert_eq!(vm.f_stack().len(), 1);
        assert_ulps_eq!(vm.f_stack().pop(), 1.0);
        vm.set_source("1.E+");
        vm.evaluate();
        assert_eq!(vm.f_stack().len(), 1);
        assert_ulps_eq!(vm.f_stack().pop(), 1.0);
        vm.set_source("1.E-");
        vm.evaluate();
        assert_eq!(vm.f_stack().len(), 1);
        assert_ulps_eq!(vm.f_stack().pop(), 1.0);
        vm.set_source("1.E2");
        vm.evaluate();
        assert_eq!(vm.f_stack().len(), 1);
        assert_ulps_eq!(vm.f_stack().pop(), 100.0);
        vm.set_source("1.0E");
        vm.evaluate();
        assert_eq!(vm.f_stack().len(), 1);
        assert_ulps_eq!(vm.f_stack().pop(), 1.0);
        vm.set_source("-1E");
        vm.evaluate();
        assert_eq!(vm.f_stack().len(), 1);
        assert_ulps_eq!(vm.f_stack().pop(), -1.0);
        vm.set_source("1.23E");
        vm.evaluate();
        assert_eq!(vm.f_stack().len(), 1);
        assert_ulps_eq!(vm.f_stack().pop(), 1.23);
        vm.set_source("12.3E-2");
        vm.evaluate();
        assert_eq!(vm.f_stack().len(), 1);
        assert_ulps_eq!(vm.f_stack().pop(), 0.123);
        vm.set_source("-12.3E+2");
        vm.evaluate();
        assert_eq!(vm.f_stack().len(), 1);
        assert_ulps_eq!(vm.f_stack().pop(), -1230.0);
        vm.set_source(".3E");
        vm.evaluate();
        assert_eq!(vm.last_error(), Some(UndefinedWord));
        assert_eq!(vm.f_stack().len(), 0);
    }

    #[test]
    fn test_evaluate_f64() {
        let vm = &mut VM::new(16, 16);
        vm.set_source("1.0E 2.5E");
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
        let vm = &mut VM::new(16, 16);
        vm.set_source("1.1E fconstant x x x");
        vm.evaluate();
        assert_eq!(vm.last_error(), None);
        assert_eq!(vm.f_stack().as_slice(), [1.1, 1.1]);
    }

    #[test]
    fn test_fstore_ffetch() {
        let vm = &mut VM::new(16, 16);
        vm.set_source("3.3e here f!  0.0e  here f@");
        vm.evaluate();
        assert_eq!(vm.last_error(), None);
        assert_eq!(vm.f_stack().as_slice(), [0.0, 3.3]);
    }

    #[test]
    fn test_fabs() {
        let vm = &mut VM::new(16, 16);
        vm.set_source("-3.14E fabs");
        vm.evaluate();
        assert_eq!(vm.last_error(), None);
        assert_eq!(vm.f_stack().len(), 1);
        assert!(match vm.f_stack().pop() {
            t => t > 3.13999 && t < 3.14001,
        });
    }

    #[test]
    fn test_fsin() {
        let vm = &mut VM::new(16, 16);
        vm.set_source("3.14E fsin");
        vm.evaluate();
        assert_eq!(vm.last_error(), None);
        assert_eq!(vm.f_stack().len(), 1);
        assert!(match vm.f_stack().pop() {
            t => t > 0.0015925 && t < 0.0015927,
        });
    }

    #[test]
    fn test_fcos() {
        let vm = &mut VM::new(16, 16);
        vm.set_source("3.0E fcos");
        vm.evaluate();
        assert_eq!(vm.last_error(), None);
        assert_eq!(vm.f_stack().len(), 1);
        assert!(match vm.f_stack().pop() {
            t => t > -0.989993 && t < -0.989991,
        });
    }

    #[test]
    fn test_ftan() {
        let vm = &mut VM::new(16, 16);
        vm.set_source("3.0E ftan");
        vm.evaluate();
        assert_eq!(vm.last_error(), None);
        assert_eq!(vm.f_stack().len(), 1);
        assert!(match vm.f_stack().pop() {
            t => t > -0.142547 && t < -0.142545,
        });
    }

    #[test]
    fn test_fasin() {
        let vm = &mut VM::new(16, 16);
        vm.set_source("0.3E fasin");
        vm.evaluate();
        assert_eq!(vm.last_error(), None);
        assert_eq!(vm.f_stack().len(), 1);
        assert!(match vm.f_stack().pop() {
            t => t > 0.304691 && t < 0.304693,
        });
    }

    #[test]
    fn test_facos() {
        let vm = &mut VM::new(16, 16);
        vm.set_source("0.3E facos");
        vm.evaluate();
        assert_eq!(vm.last_error(), None);
        assert_eq!(vm.f_stack().len(), 1);
        assert!(match vm.f_stack().pop() {
            t => t > 1.266102 && t < 1.266104,
        });
    }

    #[test]
    fn test_fatan() {
        let vm = &mut VM::new(16, 16);
        vm.set_source("0.3E fatan");
        vm.evaluate();
        assert_eq!(vm.last_error(), None);
        assert_eq!(vm.f_stack().len(), 1);
        assert!(match vm.f_stack().pop() {
            t => t > 0.291455 && t < 0.291457,
        });
    }

    #[test]
    fn test_fatan2() {
        let vm = &mut VM::new(16, 16);
        vm.set_source("3.0E 4.0E fatan2");
        vm.evaluate();
        assert_eq!(vm.last_error(), None);
        assert_eq!(vm.f_stack().len(), 1);
        assert!(match vm.f_stack().pop() {
            t => t > 0.643500 && t < 0.643502,
        });
    }

    #[test]
    fn test_fsqrt() {
        let vm = &mut VM::new(16, 16);
        vm.set_source("0.3E fsqrt");
        vm.evaluate();
        assert_eq!(vm.last_error(), None);
        assert_eq!(vm.f_stack().len(), 1);
        assert!(match vm.f_stack().pop() {
            t => t > 0.547721 && t < 0.547723,
        });
    }

    #[test]
    fn test_fdrop() {
        let vm = &mut VM::new(16, 16);
        vm.f_stack().push(1.0);
        vm.fdrop();
        assert_eq!(vm.last_error(), None);
        assert_eq!(vm.f_stack().as_slice(), []);
    }

    #[test]
    fn test_fnip() {
        let vm = &mut VM::new(16, 16);
        vm.f_stack().push2(1.0, 2.0);
        vm.check_stacks();
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
        let vm = &mut VM::new(16, 16);
        vm.f_stack().push2(1.0, 2.0);
        vm.check_stacks();
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
        let vm = &mut VM::new(16, 16);
        vm.f_stack().push(1.0);
        vm.fdup();
        vm.check_stacks();
        assert_eq!(vm.last_error(), None);
        assert_eq!(vm.f_stack().as_slice(), [1.0, 1.0]);
    }

    #[test]
    fn test_fover() {
        let vm = &mut VM::new(16, 16);
        vm.f_stack().push2(1.0, 2.0);
        vm.fover();
        vm.check_stacks();
        assert_eq!(vm.last_error(), None);
        assert_eq!(vm.f_stack().as_slice(), [1.0, 2.0, 1.0]);
    }

    #[test]
    fn test_frot() {
        let vm = &mut VM::new(16, 16);
        vm.f_stack().push3(1.0, 2.0, 3.0);
        vm.frot();
        vm.check_stacks();
        assert_eq!(vm.last_error(), None);
        assert_eq!(vm.f_stack().as_slice(), [2.0, 3.0, 1.0]);
    }

    #[test]
    fn test_fpick() {
        let vm = &mut VM::new(16, 16);
        vm.f_stack().push(1.0);
        vm.s_stack().push(0);
        vm.fpick();
        vm.check_stacks();
        assert_eq!(vm.last_error(), None);
        assert_eq!(vm.s_stack().as_slice(), []);
        assert_eq!(vm.f_stack().as_slice(), [1.0, 1.0]);

        let vm = &mut VM::new(16, 16);
        vm.f_stack().push(1.0);
        vm.f_stack().push(0.0);
        vm.s_stack().push(1);
        vm.fpick();
        vm.check_stacks();
        assert_eq!(vm.last_error(), None);
        assert_eq!(vm.s_stack().as_slice(), []);
        assert_eq!(vm.f_stack().as_slice(), [1.0, 0.0, 1.0]);
    }

    #[test]
    fn test_fplus_fminus_fstar_fslash() {
        let vm = &mut VM::new(16, 16);
        vm.set_source("9.0e 10.0e f+ 11.0e f- 12.0e f* 13.0e f/");
        vm.evaluate();
        assert_eq!(vm.last_error(), None);
        assert_eq!(vm.f_stack().len(), 1);
        assert!(match vm.f_stack().pop() {
            t => t > 7.384614 && t < 7.384616,
        });
        vm.check_stacks();
        assert_eq!(vm.last_error(), None);
    }

    #[test]
    fn test_f_zero_less_than() {
        let vm = &mut VM::new(16, 16);
        vm.set_source("0.0e f0<   0.1e f0<   -0.1e f0<");
        vm.evaluate();
        assert_eq!(vm.last_error(), None);
        assert_eq!(vm.s_stack().len(), 3);
        assert_eq!(vm.s_stack().pop(), -1);
        assert_eq!(vm.s_stack().pop(), 0);
        assert_eq!(vm.s_stack().pop(), 0);
        assert_eq!(vm.f_stack().as_slice(), []);
        vm.check_stacks();
        assert_eq!(vm.last_error(), None);
    }

    #[test]
    fn test_f_zero_equals() {
        let vm = &mut VM::new(16, 16);
        vm.set_source("0.0e f0=   0.1e f0=   -0.1e f0=");
        vm.evaluate();
        assert_eq!(vm.last_error(), None);
        assert_eq!(vm.s_stack().len(), 3);
        assert_eq!(vm.s_stack().pop(), 0);
        assert_eq!(vm.s_stack().pop(), 0);
        assert_eq!(vm.s_stack().pop(), -1);
        assert_eq!(vm.f_stack().as_slice(), []);
        vm.check_stacks();
        assert_eq!(vm.last_error(), None);
    }

    #[test]
    fn test_f_less_than() {
        let vm = &mut VM::new(16, 16);
        vm.set_source("0.0e 0.0e f<   0.1e 0.0e f<   -0.1e 0.0e f<");
        vm.evaluate();
        assert_eq!(vm.last_error(), None);
        assert_eq!(vm.s_stack().len(), 3);
        assert_eq!(vm.s_stack().pop(), -1);
        assert_eq!(vm.s_stack().pop(), 0);
        assert_eq!(vm.s_stack().pop(), 0);
        assert_eq!(vm.f_stack().as_slice(), []);
        vm.check_stacks();
        assert_eq!(vm.last_error(), None);
    }

    #[test]
    fn test_fproximate() {
        let vm = &mut VM::new(16, 16);
        vm.set_source("0.1e 0.1e 0.0e f~   0.1e 0.1000000001e 0.0e f~");
        vm.evaluate();
        assert_eq!(vm.last_error(), None);
        assert_eq!(vm.s_stack().len(), 2);
        assert_eq!(vm.s_stack().pop(), 0);
        assert_eq!(vm.s_stack().pop(), -1);
        assert_eq!(vm.f_stack().as_slice(), []);
        vm.check_stacks();
        assert_eq!(vm.last_error(), None);
        vm.s_stack().reset();
        vm.set_source("0.1e 0.1e 0.001e f~   0.1e 0.109e 0.01e f~   0.1e 0.111e  0.01e f~");
        vm.evaluate();
        assert_eq!(vm.last_error(), None);
        assert_eq!(vm.s_stack().len(), 3);
        assert_eq!(vm.s_stack().pop(), 0);
        assert_eq!(vm.s_stack().pop(), -1);
        assert_eq!(vm.s_stack().pop(), -1);
        assert_eq!(vm.f_stack().as_slice(), []);
        vm.check_stacks();
        assert_eq!(vm.last_error(), None);
        vm.s_stack().reset();
        vm.set_source("0.1e 0.1e -0.001e f~   0.1e 0.109e -0.1e f~   0.1e 0.109e  -0.01e f~");
        vm.evaluate();
        assert_eq!(vm.last_error(), None);
        assert_eq!(vm.s_stack().len(), 3);
        assert_eq!(vm.s_stack().pop(), 0);
        assert_eq!(vm.s_stack().pop(), -1);
        assert_eq!(vm.s_stack().pop(), -1);
        assert_eq!(vm.f_stack().as_slice(), []);
        vm.check_stacks();
        assert_eq!(vm.last_error(), None);
        vm.s_stack().reset();
    }

    #[test]
    #[cfg(not(target_arch = "x86_64"))]
    fn test_very_long_float() {
        let vm = &mut VM::new(16, 16);
        vm.set_source("0.10000000000000001e");
        vm.evaluate();
        assert_eq!(vm.last_error(), Some(UndefinedWord));
    }

    #[test]
    #[cfg(target_arch = "x86_64")]
    fn test_very_long_float() {
        let vm = &mut VM::new(16, 16);
        vm.set_source("0.10000000000000001e");
        vm.evaluate();
        assert_eq!(vm.last_error(), None);
    }


    #[test]
    fn test_n_to_f() {
        let vm = &mut VM::new(16, 16);
        vm.set_source("0 s>f -1 s>f 1 s>f");
        vm.evaluate();
        assert_eq!(vm.last_error(), None);
        assert_eq!(vm.f_stack().as_slice(), [0.0, -1.0, 1.0]);
    }

    #[test]
    fn test_f_to_n() {
        let vm = &mut VM::new(16, 16);
        vm.set_source("0.0e f>s -1.0e f>s 1.0e f>s");
        vm.evaluate();
        assert_eq!(vm.last_error(), None);
        assert_eq!(vm.s_stack().as_slice(), [0, -1, 1]);
    }

    #[test]
    fn test_flit_and_compile_float() {
        let vm = &mut VM::new(16, 16);
        vm.set_source(": test 1.0e 2.0e ; test");
        vm.evaluate();
        assert_eq!(vm.last_error(), None);
        assert_eq!(vm.f_stack().as_slice(), [1.0, 2.0]);
    }
}
