#![feature(duration_as_u128)]

extern crate getopts;
#[macro_use(primitive)]
extern crate rtforth;
extern crate rustyline;

use getopts::Options;
use rtforth::memory::{CodeSpace, DataSpace};
use rtforth::core::{Control, Core, ForwardReferences, Stack, State, Word};
use rtforth::env::Environment;
use rtforth::exception::Exception;
use rtforth::facility::Facility;
use rtforth::float::Float;
use rtforth::loader::HasLoader;
use rtforth::output::Output;
use rtforth::tools::Tools;
use rtforth::units::Units;
use std::env;
use std::fmt::Write;
use std::process;
use std::time::SystemTime;

// Virtual machine
pub struct VM {
    editor: rustyline::Editor<()>,
    last_error: Option<Exception>,
    handler: usize,
    regs: [usize; 2],
    s_stk: Stack<isize>,
    r_stk: Stack<isize>,
    c_stk: Stack<Control>,
    f_stk: Stack<f64>,
    symbols: Vec<String>,
    last_definition: usize,
    wordlist: Vec<Word<VM>>,
    data_space: DataSpace,
    code_space: CodeSpace,
    inbuf: Option<String>,
    tkn: Option<String>,
    outbuf: Option<String>,
    hldbuf: String,
    state: State,
    references: ForwardReferences,
    now: SystemTime,
}

impl VM {
    pub fn new(data_pages: usize, code_pages: usize) -> VM {
        let mut vm = VM {
            editor: rustyline::Editor::<()>::new(),
            last_error: None,
            handler: 0,
            regs: [0, 0],
            s_stk: Stack::new(0x12345678),
            r_stk: Stack::new(0x12345678),
            c_stk: Stack::new(Control::Default),
            f_stk: Stack::new(1.234567890),
            symbols: vec![],
            last_definition: 0,
            wordlist: vec![],
            data_space: DataSpace::new(data_pages),
            code_space: CodeSpace::new(code_pages),
            inbuf: Some(String::with_capacity(128)),
            tkn: Some(String::with_capacity(64)),
            outbuf: Some(String::with_capacity(128)),
            hldbuf: String::with_capacity(128),
            state: State::new(),
            references: ForwardReferences::new(),
            now: SystemTime::now(),
        };
        vm.add_core();
        vm.add_output();
        vm.add_tools();
        vm.add_environment();
        vm.add_facility();
        vm.add_float();
        vm.add_units();
        vm.add_primitive("accept", p_accept);
        vm.add_primitive("bye", bye);

        vm.load_core_fs();

        let libfs = include_str!("./lib.fs");
        vm.load_str(libfs);
        if vm.last_error().is_some() {
            panic!("Error {:?} {:?}", vm.last_error().unwrap(), vm.last_token());
        }

        vm.flush();

        vm
    }

}

impl Core for VM {
    fn last_error(&self) -> Option<Exception> {
        self.last_error
    }
    fn set_error(&mut self, e: Option<Exception>) {
        self.last_error = e;
    }
    fn handler(&self) -> usize {
        self.handler
    }
    fn set_handler(&mut self, h: usize) {
        self.handler = h;
    }
    fn data_space(&mut self) -> &mut DataSpace {
        &mut self.data_space
    }
    fn data_space_const(&self) -> &DataSpace {
        &self.data_space
    }
    fn code_space(&mut self) -> &mut CodeSpace {
        &mut self.code_space
    }
    fn code_space_const(&self) -> &CodeSpace {
        &self.code_space
    }
    fn hold_buffer(&mut self) -> &mut String {
        &mut self.hldbuf
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
    fn regs(&mut self) -> &mut [usize; 2] {
        &mut self.regs
    }
    fn s_stack(&mut self) -> &mut Stack<isize> {
        &mut self.s_stk
    }
    fn r_stack(&mut self) -> &mut Stack<isize> {
        &mut self.r_stk
    }
    fn c_stack(&mut self) -> &mut Stack<Control> {
        &mut self.c_stk
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
    fn system_time_ns(&self) -> i64 {
        match self.now.elapsed() {
            Ok(d) => {
                d.as_nanos() as i64
            }
            Err(_) => {
                0i64
            }
        }
    }
}

impl Environment for VM {}
impl Facility for VM {}
impl Float for VM {}
impl Units for VM {}
impl HasLoader for VM {}
impl Output for VM {}
impl Tools for VM {}

fn main() {
    let vm = &mut VM::new(1024, 1024);

    let mut bye = false;

    let args: Vec<_> = env::args().collect();
    let program = args[0].clone();
    let mut opts = Options::new();
    opts.optflag("h", "help", "print help menu");
    opts.optflag("v", "version", "print version number");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => {
            panic!(f.to_string());
        }
    };
    if matches.opt_present("h") {
        print_usage(&program, opts);
    } else if matches.opt_present("v") {
        print_version();
    } else if !matches.free.is_empty() {
        for file in matches.free {
            if let Err(e) = vm.load(&file) {
                vm.clear_stacks();
                vm.reset();
                println!("{} ", e.description());
                bye = true;
                break;
            }
        }
        if !bye {
            repl(vm);
        }
    } else {
        print_version();
        println!("Type 'bye' or press Ctrl-D to exit.");
        repl(vm);
    }
}

fn print_version() {
    println!("rtForth v0.4.0, Copyright (C) 2018 Mapacode Inc.");
}

primitive!{fn p_accept(vm: &mut VM) {
    match vm.editor.readline("rf> ") {
        Ok(line) => {
            vm.editor.add_history_entry(&line);
            vm.set_source(&line);
        }
        Err(rustyline::error::ReadlineError::Eof) => {
            bye(vm);
        }
        Err(err) => {
            match vm.output_buffer().as_mut() {
                Some(ref mut buf) => {
                    write!(buf, "{}", err).unwrap();
                }
                None => {}
            }
        }
    }
}}

/// Terminate process.
primitive!{fn bye(vm: &mut VM) {
    vm.flush();
    process::exit(0);
}}

#[inline(never)]
fn repl(vm: &mut VM) {
    let quit = vm.find("QUIT").expect("QUIT");
    vm.execute_word(quit);
    vm.run();
}

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} [files] [options]", program);
    print!("{}", opts.usage(&brief));
}
