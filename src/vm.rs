use std::mem;
use std::ptr;
use output::Output;
use core::{Core, Word, ForwardReferences, Stack, State};
use jitmem::DataSpace;
use loader::HasLoader;
use tools::Tools;
use env::Environment;
use facility::Facility;
use float::Float;
use bc::*;
use exception::Exception::{self, StackUnderflow, StackOverflow, ReturnStackUnderflow,
                           ReturnStackOverflow, FloatingPointStackOverflow, InvalidMemoryAddress,
                           Nest, Quit, Pause};

// Virtual machine
pub struct VM {
    last_error: Option<Exception>,
    structure_depth: usize,
    s_stk: Stack<isize>,
    r_stk: Stack<isize>,
    f_stk: Stack<f64>,
    symbols: Vec<String>,
    last_definition: usize,
    wordlist: Vec<Word<VM>>,
    data_space: DataSpace,
    inbuf: Option<String>,
    tkn: Option<String>,
    outbuf: Option<String>,
    state: State,
    references: ForwardReferences,
    evals: Option<Vec<fn(&mut VM, token: &str)>>,
    evaluation_limit: isize,
}

impl VM {
    pub fn new(pages: usize) -> VM {
        VM {
            last_error: None,
            structure_depth: 0,
            s_stk: Stack::with_capacity(64),
            r_stk: Stack::with_capacity(64),
            f_stk: Stack::with_capacity(16),
            symbols: vec![],
            last_definition: 0,
            wordlist: vec![],
            data_space: DataSpace::new(pages),
            inbuf: Some(String::with_capacity(128)),
            tkn: Some(String::with_capacity(64)),
            outbuf: Some(String::with_capacity(128)),
            state: State::new(),
            references: ForwardReferences::new(),
            evals: None,
            evaluation_limit: 0isize,
        }
    }
}

impl Core for VM {
    fn last_error(&self) -> Option<Exception> {
        self.last_error
    }
    fn set_error(&mut self, e: Option<Exception>) {
        self.last_error = e;
    }
    fn structure_depth(&self) -> usize {
        self.structure_depth
    }
    fn set_structure_depth(&mut self, depth: usize) {
        self.structure_depth = depth
    }
    fn data_space(&mut self) -> &mut DataSpace {
        &mut self.data_space
    }
    fn data_space_const(&self) -> &DataSpace {
        &self.data_space
    }
    fn output_buffer(&mut self) -> &mut Option<String> {
        &mut self.outbuf
    }
    fn set_output_buffer(&mut self, buffer: String) {
        self.outbuf = Some(buffer);
    }
    fn input_buffer(&mut self) -> &mut Option<String> {
        &mut self.inbuf
    }
    fn set_input_buffer(&mut self, buffer: String) {
        self.inbuf = Some(buffer);
    }
    fn last_token(&mut self) -> &mut Option<String> {
        &mut self.tkn
    }
    fn set_last_token(&mut self, buffer: String) {
        self.tkn = Some(buffer);
    }
    fn s_stack(&mut self) -> &mut Stack<isize> {
        &mut self.s_stk
    }
    fn r_stack(&mut self) -> &mut Stack<isize> {
        &mut self.r_stk
    }
    fn f_stack(&mut self) -> &mut Stack<f64> {
        &mut self.f_stk
    }
    fn symbols_mut(&mut self) -> &mut Vec<String> {
        &mut self.symbols
    }
    fn symbols(&self) -> &Vec<String> {
        &self.symbols
    }
    fn last_definition(&self) -> usize {
        self.last_definition
    }
    fn set_last_definition(&mut self, n: usize) {
        self.last_definition = n;
    }
    fn wordlist_mut(&mut self) -> &mut Vec<Word<Self>> {
        &mut self.wordlist
    }
    fn wordlist(&self) -> &Vec<Word<Self>> {
        &self.wordlist
    }
    fn state(&mut self) -> &mut State {
        &mut self.state
    }
    fn references(&mut self) -> &mut ForwardReferences {
        &mut self.references
    }
    fn evaluators(&mut self) -> &mut Option<Vec<fn(&mut Self, token: &str)>> {
        &mut self.evals
    }
    fn set_evaluators(&mut self, evaluators: Vec<fn(&mut Self, token: &str)>) {
        self.evals = Some(evaluators)
    }
    fn evaluation_limit(&self) -> isize {
        self.evaluation_limit
    }
}

impl Environment for VM {}
impl Facility for VM {}
impl Float for VM {}
impl HasLoader for VM {}
impl Output for VM {}
impl Tools for VM {}


// ------------------
// Inner interpreter
// ------------------

/// Evaluate a compiled program following self.state().instruction_pointer.
/// Any exception other than Nest causes termination of inner loop.
/// Quit is aspecially used for this purpose.
/// Never return None and Some(Nest).
#[inline(never)]
fn switch_threading_run(vm: &mut VM) {
    let mut ip = vm.state().instruction_pointer;
    while 0 < ip && ip < vm.data_space().len() {
        let w = vm.data_space().get_i32(ip) as usize;
        ip += mem::size_of::<i32>();
        match w {
            BC_NOOP => {}
            BC_EXIT => {
                if vm.r_stack().len == 0 {
                    vm.set_error(Some(ReturnStackUnderflow));
                } else {
                    vm.r_stack().len -= 1;
                    unsafe {
                        ip = ptr::read(vm.r_stack()
                            .inner
                            .offset(vm.r_stack().len as isize)) as
                             usize;
                    }
                }
            }
            BC_HALT => {
                ip = 0;
                vm.set_error(Some(Quit));
            }
            BC_LIT => {
                if vm.s_stack().is_full() {
                    vm.set_error(Some(StackOverflow));
                } else {
                    unsafe {
                        let v = vm.data_space().get_i32(ip) as isize;
                        ptr::write(vm.s_stack().inner.offset((vm.s_stack().len) as isize), v);
                    }
                    vm.s_stack().len += 1;
                    ip += mem::size_of::<i32>();
                }
            }
            BC_FLIT => {
                let v = vm.data_space().get_f64(ip);
                match vm.f_stack().push(v) {
                    Err(_) => vm.set_error(Some(FloatingPointStackOverflow)),
                    Ok(()) => {
                            ip += mem::size_of::<f64>();
                        }
                }
            }
            BC_S_QUOTE => {
                let cnt = vm.data_space().get_i32(ip);
                let addr = ip + mem::size_of::<i32>();
                match vm.s_stack().push2(addr as isize, cnt as isize) {
                    Err(_) => vm.set_error(Some(StackOverflow)),
                    Ok(()) => {
                            ip +=  mem::size_of::<i32>() + cnt as usize;
                        }
                }
            }
            BC_BRANCH => {
                ip = vm.data_space().get_i32(ip) as usize;
            }
            BC_ZBRANCH => {
                match vm.s_stack().pop() {
                    Ok(v) => {
                        if v == 0 {
                            ip = vm.data_space().get_i32(ip) as usize;
                        } else {
                            ip += mem::size_of::<i32>();
                        }
                    }
                    Err(_) => vm.set_error(Some(StackUnderflow)),
                }
            }
            BC_DO => {
                match vm.r_stack().push(ip as isize) {
                    Err(_) => vm.set_error(Some(ReturnStackOverflow)),
                    Ok(()) => {
                            ip += mem::size_of::<i32>();
                            vm.two_to_r();
                        }
                }
            }
            BC_LOOP => {
                match vm.r_stack().pop2() {
                    Ok((rn, rt)) => {
                        if rt + 1 < rn {
                            if let Err(e) = vm.r_stack().push2(rn, rt + 1) {
                                vm.set_error(Some(e));
                                return;
                            }
                            ip = vm.data_space().get_i32(ip) as usize;
                        } else {
                            match vm.r_stack().pop() {
                                Ok(_) => {
                                    ip += mem::size_of::<i32>();
                                }
                                Err(_) => vm.set_error(Some(ReturnStackUnderflow)),
                            }
                        }
                    }
                    Err(_) => vm.set_error(Some(ReturnStackUnderflow)),
                }
            }
            BC_PLUS_LOOP => {
                match vm.r_stack().pop2() {
                    Ok((rn, rt)) => {
                        match vm.s_stack().pop() {
                            Ok(t) => {
                                if rt + t < rn {
                                    if let Err(e) = vm.r_stack().push2(rn, rt + t) {
                                        vm.set_error(Some(e));
                                        return;
                                    }
                                    ip = vm.data_space().get_i32(ip) as usize;
                                } else {
                                    match vm.r_stack().pop() {
                                        Ok(_) => {
                                            ip += mem::size_of::<i32>();
                                        }
                                        Err(_) => vm.set_error(Some(ReturnStackUnderflow)),
                                    }
                                }
                            }
                            Err(_) => vm.set_error(Some(StackUnderflow)),
                        }
                    }
                    Err(_) => vm.set_error(Some(ReturnStackUnderflow)),
                }
            }
            BC_UNLOOP => {
                match vm.r_stack().pop3() {
                    Ok(_) => {}
                    Err(_) => vm.set_error(Some(ReturnStackUnderflow)),
                }
            }
            BC_LEAVE => {
                match vm.r_stack().pop3() {
                    Ok((third, _, _)) => {
                        ip = vm.data_space().get_i32(third as usize) as usize;
                    }
                    Err(_) => vm.set_error(Some(ReturnStackUnderflow)),
                }
            }
            BC_I => {
                match vm.r_stack().last() {
                    Some(i) => {
                        match vm.s_stack().push(i) {
                            Err(_) => vm.set_error(Some(StackOverflow)),
                            Ok(()) => {}
                        }
                    }
                    None => vm.set_error(Some(ReturnStackUnderflow)),
                }
            }
            BC_J => {
                let pos = vm.r_stack().len() - 4;
                match vm.r_stack().get(pos) {
                    Some(j) => {
                        match vm.s_stack().push(j) {
                            Err(_) => vm.set_error(Some(StackOverflow)),
                            Ok(()) => {}
                        }
                    }
                    None => vm.set_error(Some(ReturnStackUnderflow)),
                }
            }
            BC_TO_R => {
                match vm.s_stack().pop() {
                    Ok(v) => {
                        if vm.r_stack().is_full() {
                            vm.set_error(Some(ReturnStackOverflow));
                        } else {
                            unsafe {
                                ptr::write(vm.r_stack()
                                               .inner
                                               .offset(vm.r_stack().len as isize),
                                           v);
                            }
                            vm.r_stack().len += 1;
                        }
                    }
                    Err(_) => vm.set_error(Some(StackUnderflow)),
                }
            }
            BC_R_FROM => {
                if vm.r_stack().len == 0 {
                    vm.set_error(Some(ReturnStackUnderflow));
                } else if vm.s_stack().is_full() {
                    vm.set_error(Some(StackOverflow));
                } else {
                    vm.r_stack().len -= 1;
                    unsafe {
                        let r0 = vm.r_stack().inner.offset(vm.r_stack().len as isize);
                        match vm.s_stack().push(ptr::read(r0)) {
                            Err(e) => vm.set_error(Some(e)),
                            Ok(()) => {}
                        }
                    }
                }
            } 
            BC_R_FETCH => {
                if vm.r_stack().len == 0 {
                    vm.set_error(Some(ReturnStackUnderflow));
                } else if vm.s_stack().is_full() {
                    vm.set_error(Some(StackOverflow));
                } else {
                    unsafe {
                        let r1 = vm.r_stack().inner.offset((vm.r_stack().len - 1) as isize);
                        match vm.s_stack().push(ptr::read(r1)) {
                            Err(e) => vm.set_error(Some(e)),
                            Ok(()) => {}
                        }
                    }
                }
            }
            BC_TWO_TO_R => {
                vm.two_to_r();
            }
            BC_TWO_R_FROM => {
                // TODO: check overflow.
                if vm.r_stack().len < 2 {
                    vm.set_error(Some(ReturnStackUnderflow));
                } else {
                    vm.r_stack().len -= 2;
                    unsafe {
                        let r0 = vm.r_stack().inner.offset(vm.r_stack().len as isize);
                        match vm.s_stack().push(ptr::read(r0)) {
                            Err(e) => vm.set_error(Some(e)),
                            Ok(()) => {}
                        }
                        let r1 = vm.r_stack().inner.offset((vm.r_stack().len + 1) as isize);
                        match vm.s_stack().push(ptr::read(r1)) {
                            Err(e) => vm.set_error(Some(e)),
                            Ok(()) => {}
                        }
                    }
                }
            }
            BC_TWO_R_FETCH => {
                if vm.r_stack().len < 2 {
                    vm.set_error(Some(ReturnStackUnderflow));
                } else {
                    unsafe {
                        let r2 = vm.r_stack().inner.offset((vm.r_stack().len - 2) as isize);
                        match vm.s_stack().push(ptr::read(r2)) {
                            Err(e) => vm.set_error(Some(e)),
                            Ok(()) => {}
                        }
                        let r1 = vm.r_stack().inner.offset((vm.r_stack().len - 1) as isize);
                        match vm.s_stack().push(ptr::read(r1)) {
                            Err(e) => vm.set_error(Some(e)),
                            Ok(()) => {}
                        }
                    }
                }
            }
            _ => {
                vm.state().instruction_pointer = ip;
                vm.execute_word(w);
                ip = vm.state().instruction_pointer;
            }
        }
        match vm.last_error() {
            Some(e) => {
                match e {
                    Nest => {
                        vm.set_error(None);
                    }
                    _ => {
                        break;
                    }
                }
            }
            None => {}
        }
    }
    vm.state().instruction_pointer = ip;
    if vm.state().instruction_pointer != 0 {
        match vm.last_error() {
            None => {
                vm.set_error(Some(InvalidMemoryAddress));
                vm.state().instruction_pointer = 0;
            }
            Some(Pause) => {}
            _ => {
                vm.state().instruction_pointer = 0;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    extern crate test;
    use vm::VM;
    use core::Core;
    use output::Output;
    use loader::HasLoader;
    use self::test::Bencher;
    use exception::Exception::Quit;
    use super::switch_threading_run;

    #[bench]
    fn bench_fib(b: &mut Bencher) {
        let mut vm = &mut VM::new(16);
        vm.add_core();
        vm.set_source(": fib dup 2 < if drop 1 else dup 1- recurse swap 2 - recurse + then ;");
        vm.evaluate();
        assert!(vm.last_error().is_none());
        vm.set_source(": main 7 fib drop ;");
        vm.evaluate();
        vm.set_source("' main");
        vm.evaluate();
        b.iter(|| {
            vm.dup();
            vm.execute();
            switch_threading_run(&mut vm);
            match vm.last_error() {
                Some(e) => {
                    match e {
                        Quit => {}
                        _ => {
                            assert!(false);
                        }
                    }
                }
                None => assert!(true),
            };
        });
    }

    #[bench]
    fn bench_repeat(b: &mut Bencher) {
        let mut vm = &mut VM::new(16);
        vm.add_core();
        vm.set_source(": bench 0 begin over over > while 1 + repeat drop drop ;");
        vm.evaluate();
        vm.set_source(": main 8000 bench ;");
        vm.evaluate();
        vm.set_source("' main");
        vm.evaluate();
        b.iter(|| {
            vm.dup();
            vm.execute();
            switch_threading_run(&mut vm);
            match vm.last_error() {
                Some(e) => {
                    match e {
                        Quit => {}
                        _ => {
                            assert!(false);
                        }
                    }
                }
                None => assert!(true),
            };
        });
    }

    #[bench]
    fn bench_sieve(b: &mut Bencher) {
        let mut vm = &mut VM::new(16);
        vm.add_core();
        vm.add_output();
        vm.load("./lib.fs");
        assert_eq!(vm.last_error(), None);
        vm.set_source("CREATE FLAGS 8190 ALLOT   VARIABLE EFLAG");
        vm.evaluate();
        assert_eq!(vm.last_error(), None);
        vm.set_source("
            : PRIMES  ( -- n )  FLAGS 8190 1 FILL  0 3  EFLAG @ FLAGS
                DO   I C@
                    IF  DUP I + DUP EFLAG @ <
                        IF    EFLAG @ SWAP
                            DO  0 I C! DUP  +LOOP
                        ELSE  DROP  THEN  SWAP 1+ SWAP
                    THEN  2 +
                LOOP  DROP ;
        ");
        vm.evaluate();
        assert_eq!(vm.last_error(), None);
        vm.set_source("
            : BENCHMARK  0 1 0 DO  PRIMES NIP  LOOP ;
        ");
        vm.evaluate();
        assert_eq!(vm.last_error(), None);
        vm.set_source("
            : MAIN 
                FLAGS 8190 + EFLAG !
                BENCHMARK DROP
            ;
        ");
        vm.evaluate();
        assert_eq!(vm.last_error(), None);
        vm.set_source("' main");
        vm.evaluate();
        b.iter(|| {
            vm.dup();
            vm.execute();
            switch_threading_run(&mut vm);
            match vm.last_error() {
                Some(e) => {
                    match e {
                        Quit => {}
                        _ => {
                            assert!(false);
                        }
                    }
                }
                None => assert!(true),
            };
        });
    }
}