extern crate rtforth;
extern crate futures;
#[macro_use(try_nb)]
extern crate tokio_core;

const BUFFER_SIZE: usize = 0x400;

mod vm {
    use rtforth::core::{Core, Stack, State, ForwardReferences, Word};
    use rtforth::jitmem::DataSpace;
    use rtforth::float::Float;
    use rtforth::env::Environment;
    use rtforth::exception::Exception;
    use rtforth::facility::Facility;
    use rtforth::loader::HasLoader;
    use rtforth::output::Output;
    use rtforth::tools::Tools;
    use super::BUFFER_SIZE;

    struct Task {
        last_error: Option<Exception>,
        state: State,
        s_stk: Stack<isize>,
        r_stk: Stack<isize>,
        f_stk: Stack<f64>,
        inbuf: Option<String>,
        tkn: Option<String>,
        outbuf: Option<String>,
    }

    // Virtual machine
    pub struct VM {
        current_task: usize,
        tasks_used: [bool; 3],
        tasks: [Task; 3],
        symbols: Vec<String>,
        structure_depth: usize,
        last_definition: usize,
        wordlist: Vec<Word<VM>>,
        jitmem: DataSpace,
        references: ForwardReferences,
        evals: Option<Vec<fn(&mut VM, token: &str)>>,
        // Evalution limit for tasks[1]
        evaluation_limit: isize,
    }

    impl VM {
        pub fn new(pages: usize) -> VM {
            let mut vm = VM {
                current_task: 0,
                tasks_used: [false; 3],
                tasks: [Task {
                            last_error: None,
                            state: State::new(),
                            s_stk: Stack::with_capacity(64),
                            r_stk: Stack::with_capacity(64),
                            f_stk: Stack::with_capacity(16),
                            inbuf: Some(String::with_capacity(BUFFER_SIZE)),
                            tkn: Some(String::with_capacity(64)),
                            outbuf: Some(String::with_capacity(BUFFER_SIZE)),
                        },
                        Task {
                            last_error: None,
                            state: State::new(),
                            s_stk: Stack::with_capacity(64),
                            r_stk: Stack::with_capacity(64),
                            f_stk: Stack::with_capacity(16),
                            inbuf: Some(String::with_capacity(BUFFER_SIZE)),
                            tkn: Some(String::with_capacity(64)),
                            outbuf: Some(String::with_capacity(BUFFER_SIZE)),
                        },
                        Task {
                            last_error: None,
                            state: State::new(),
                            s_stk: Stack::with_capacity(64),
                            r_stk: Stack::with_capacity(64),
                            f_stk: Stack::with_capacity(16),
                            inbuf: Some(String::with_capacity(BUFFER_SIZE)),
                            tkn: Some(String::with_capacity(64)),
                            outbuf: Some(String::with_capacity(BUFFER_SIZE)),
                        }],
                symbols: vec![],
                structure_depth: 0,
                last_definition: 0,
                wordlist: vec![],
                jitmem: DataSpace::new(pages),
                references: ForwardReferences::new(),
                evals: None,
                evaluation_limit: 80,
            };
            vm.add_core();
            vm.add_output();
            vm.add_tools();
            vm.add_environment();
            vm.add_facility();
            vm.add_float();
            vm
        }

        pub fn alloc_task(&mut self) -> Option<usize> {
            match self.tasks_used.iter().position(|&b| b == false) {
                Some(i) => {
                    self.tasks_used[i] = true;
                    Some(i)
                }
                None => None,
            }
        }

        pub fn free_task(&mut self, i: usize) {
            self.tasks_used[i] = false;
        }

        pub fn current_task(&self) -> usize {
            self.current_task
        }

        pub fn set_current_task(&mut self, n: usize) {
            self.current_task = n;
        }
    }

    impl Core for VM {
        fn last_error(&self) -> Option<Exception> {
            self.tasks[self.current_task].last_error
        }
        fn set_error(&mut self, e: Option<Exception>) {
            self.tasks[self.current_task].last_error = e;
        }
        fn structure_depth(&self) -> usize {
            self.structure_depth
        }
        fn set_structure_depth(&mut self, depth: usize) {
            self.structure_depth = depth
        }
        fn data_space(&mut self) -> &mut DataSpace {
            &mut self.jitmem
        }
        fn data_space_const(&self) -> &DataSpace {
            &self.jitmem
        }
        fn output_buffer(&mut self) -> &mut Option<String> {
            &mut self.tasks[self.current_task].outbuf
        }
        fn set_output_buffer(&mut self, buffer: String) {
            self.tasks[self.current_task].outbuf = Some(buffer);
        }
        fn input_buffer(&mut self) -> &mut Option<String> {
            &mut self.tasks[self.current_task].inbuf
        }
        fn set_input_buffer(&mut self, buffer: String) {
            self.tasks[self.current_task].inbuf = Some(buffer);
        }
        fn last_token(&mut self) -> &mut Option<String> {
            &mut self.tasks[self.current_task].tkn
        }
        fn set_last_token(&mut self, buffer: String) {
            self.tasks[self.current_task].tkn = Some(buffer);
        }
        fn s_stack(&mut self) -> &mut Stack<isize> {
            &mut self.tasks[self.current_task].s_stk
        }
        fn r_stack(&mut self) -> &mut Stack<isize> {
            &mut self.tasks[self.current_task].r_stk
        }
        fn f_stack(&mut self) -> &mut Stack<f64> {
            &mut self.tasks[self.current_task].f_stk
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
        fn wordlist_mut(&mut self) -> &mut Vec<Word<VM>> {
            &mut self.wordlist
        }
        fn wordlist(&self) -> &Vec<Word<VM>> {
            &self.wordlist
        }
        fn state(&mut self) -> &mut State {
            &mut self.tasks[self.current_task].state
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
            if self.current_task == 0 {
                0
            } else {
                self.evaluation_limit
            }
        }
    }

    impl Environment for VM {}
    impl Facility for VM {}
    impl Float for VM {}
    impl HasLoader for VM {}
    impl Output for VM {}
    impl Tools for VM {}
}

mod server {
    use std::io;
    use futures::{Future, Poll};
    use vm::VM;
    use std::sync::{Arc, Mutex};
    use super::BUFFER_SIZE;
    use std::str;
    use rtforth::core::Core;
    use std::fmt::Write;

    pub struct Eval<R, W> {
        vm: Arc<Mutex<VM>>,
        tsk: Option<usize>,
        reader: R,
        done: bool,
        writer: W,
        pos: usize,
        cap: usize,
        inbuf: Box<[u8]>,
        outbuf: Box<[u8]>,
    }
    impl<R, W> Drop for Eval<R, W> {
        fn drop(&mut self) {
            match self.tsk {
                Some(i) => {
                    let mut v = self.vm.lock().unwrap();
                    (*v).free_task(i);
                }
                None => {}
            }
        }
    }
    pub fn eval<R, W>(reader: R, writer: W, vm: Arc<Mutex<VM>>) -> Eval<R, W>
        where R: io::Read,
              W: io::Write
    {
        let tsk = {
            let mut v = vm.lock().unwrap();
            (*v).alloc_task()
        };
        Eval {
            vm: vm,
            tsk: tsk,
            reader: reader,
            done: false,
            writer: writer,
            pos: 0,
            cap: 0,
            inbuf: Box::new([0; BUFFER_SIZE]),
            outbuf: Box::new([0; BUFFER_SIZE]),
        }
    }

    impl<R, W> Future for Eval<R, W>
        where R: io::Read,
              W: io::Write
    {
        type Item = ();
        type Error = io::Error;

        fn poll(&mut self) -> Poll<(), io::Error> {
            loop {
                while self.pos < self.cap {
                    let i = try_nb!(self.writer.write(&self.outbuf[self.pos..self.cap]));
                    self.pos += i;
                }

                if self.done {
                    try_nb!(self.writer.flush());
                    return Ok(().into());
                }

                let n = try_nb!(self.reader.read(&mut self.inbuf));
                if n == 0 {
                    self.done = true;
                } else {
                    match self.tsk {
                        Some(i) => {
                            let mut vm = self.vm.lock().unwrap();
                            (*vm).set_current_task(i);
                            (*vm).set_source(str::from_utf8(&self.inbuf[0..n]).unwrap());
                            (*vm).evaluate();
                            let mut outbuf = (*vm).output_buffer().take().unwrap();
                            match vm.last_error() {
                                Some(e) => {
                                    writeln!(outbuf, "{:?}", e);
                                }
                                None => {}
                            }
                            self.pos = 0;
                            self.cap = outbuf.len();
                            self.outbuf[..self.cap].copy_from_slice(outbuf.as_bytes());
                            outbuf.clear();
                            (*vm).set_output_buffer(outbuf);
                        }
                        None => {
                            let msg = "Cannot allocate task.";
                            self.cap = msg.len();
                            self.pos = 0;
                            self.outbuf[..self.cap].copy_from_slice(msg.as_bytes());
                            self.done = true;
                        }
                    }
                }
            }
        }
    }
}

use futures::{Future, Stream};
use tokio_core::io::Io;
use tokio_core::net::TcpListener;
use tokio_core::reactor::Core;
use vm::VM;
use std::sync::{Arc, Mutex};

fn main() {
    let mut core = Core::new().unwrap();
    let handle = core.handle();

    let addr = "127.0.0.1:12345".parse().unwrap();
    let sock = TcpListener::bind(&addr, &handle).unwrap();

    let vm = Arc::new(Mutex::new(VM::new(0x100)));

    let server = sock.incoming().for_each(|(sock, _)| {
        let (reader, writer) = sock.split();

        let future = server::eval(reader, writer, vm.clone());

        let handle_conn = future.map(|_| println!("done"))
            .map_err(|err| println!("IO error {:?}", err));

        handle.spawn(handle_conn);

        Ok(())
    });

    core.run(server).unwrap();
}
