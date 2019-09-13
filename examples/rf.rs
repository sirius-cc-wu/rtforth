#![feature(duration_as_u128)]

extern crate getopts;
#[macro_use(primitive)]
extern crate rtforth;
extern crate rustyline;
extern crate time;
extern crate hibitset;

use getopts::Options;
use rtforth::memory::{CodeSpace, DataSpace};
use rtforth::core::{Control, Core, ForwardReferences, Stack, State, Wordlist, Label};
use rtforth::env::Environment;
use rtforth::exception::Exception;
use rtforth::facility::Facility;
use rtforth::float::Float;
use rtforth::loader::{HasLoader, Source};
use rtforth::output::Output;
use rtforth::tools::Tools;
use rtforth::units::Units;
use rtforth::file_access::FileAccess;
use rtforth::NUM_TASKS;
use std::env;
use std::fmt::Write;
use std::process;
use std::fs::File;
use hibitset::BitSet;

const BUFFER_SIZE: usize = 0x400;
const LABEL_COUNT: u32 = 1000;

/// Task
///
/// Each task has its own input buffer but shares the
/// dictionary and output buffer owned by virtual machine.
pub struct Task {
    awake: bool,
    state: State,
    regs: [usize; 2],
    s_stk: Stack<isize>,
    r_stk: Stack<isize>,
    c_stk: Stack<Control>,
    f_stk: Stack<f64>,
    inbuf: Option<String>,
    files: Vec<Option<File>>,
    sources: Vec<Option<Source>>,
    lines: Vec<Option<String>>,
}

impl Task {
    /// Create a task without input buffer.
    pub fn new_background() -> Task {
        Task {
            awake: false,
            state: State::new(),
            regs: [0, 0],
            s_stk: Stack::new(0x12345678),
            r_stk: Stack::new(0x12345678),
            c_stk: Stack::new(Control::Default),
            f_stk: Stack::new(1.234567890),
            inbuf: None,
            files: Vec::new(),
            sources: Vec::new(),
            lines: Vec::new(),
        }
    }

    /// Create a task with input buffer.
    pub fn new_terminal() -> Task {
        let mut task = Task::new_background();
        task.inbuf = Some(String::with_capacity(BUFFER_SIZE));
        task
    }
}

/// Virtual machine
pub struct VM {
    current_task: usize,
    tasks: [Task; NUM_TASKS],
    editor: rustyline::Editor<()>,
    last_error: Option<Exception>,
    handler: usize,
    wordlist: Wordlist<VM>,
    data_space: DataSpace,
    code_space: CodeSpace,
    tkn: Option<String>,
    outbuf: Option<String>,
    hldbuf: String,
    references: ForwardReferences,
    now: time::Tm,
    bitset: BitSet,
    labels: Vec<Label>,
}

impl VM {
    /// Create a VM with data and code space size specified
    /// by `data_pages` and `code_pages`.
    pub fn new(data_pages: usize, code_pages: usize) -> VM {
        let mut labels = Vec::with_capacity(LABEL_COUNT as _);
        labels.resize(LABEL_COUNT as _, Label::None);
        let mut vm = VM {
            current_task: 0,
            tasks: [
                // Only the operator task is a terminal task
                // with its own input buffer.
                Task::new_terminal(),
                Task::new_background(),
                Task::new_background(),
                Task::new_background(),
                Task::new_background(),
            ],
            editor: rustyline::Editor::<()>::new(),
            last_error: None,
            handler: 0,
            wordlist: Wordlist::with_capacity(1000),
            data_space: DataSpace::new(data_pages),
            code_space: CodeSpace::new(code_pages),
            tkn: Some(String::with_capacity(64)),
            outbuf: Some(String::with_capacity(128)),
            hldbuf: String::with_capacity(128),
            references: ForwardReferences::new(),
            now: time::now(),
            bitset: BitSet::with_capacity(LABEL_COUNT),
            labels,
        };
        vm.add_core();
        vm.add_output();
        vm.add_tools();
        vm.add_environment();
        vm.add_facility();
        vm.add_float();
        vm.add_units();
        vm.add_file_access();
        vm.add_loader();
        vm.add_primitive("receive", receive);
        vm.add_primitive("bye", bye);

        vm.load_core_fs();

        let rffs = include_str!("./rf.fs");
        vm.load_str(rffs);
        if vm.last_error().is_some() {
            panic!("Error {:?} {:?}", vm.last_error().unwrap(), vm.last_token());
        }

        vm.flush_output();

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
    fn source_id(&self) -> isize {
        self.tasks[self.current_task].state.source_id
    }
    fn input_buffer(&mut self) -> &mut Option<String> {
        let source_id = self.source_id();
        if source_id > 0 {
            &mut self.lines_mut()[source_id as usize - 1]
        } else {
            &mut self.tasks[self.current_task].inbuf
        }
    }
    fn set_input_buffer(&mut self, buffer: String) {
        // self.tasks[self.current_task].inbuf = Some(buffer);
        *self.input_buffer() = Some(buffer);
    }
    fn files(&self) -> &Vec<Option<File>> {
        &self.tasks[self.current_task].files
    }
    fn files_mut(&mut self) -> &mut Vec<Option<File>> {
        &mut self.tasks[self.current_task].files
    }
    fn sources(&self) -> &Vec<Option<Source>> {
        &self.tasks[self.current_task].sources
    }
    fn sources_mut(&mut self) -> &mut Vec<Option<Source>> {
        &mut self.tasks[self.current_task].sources
    }
    fn lines(&self) -> &Vec<Option<String>> {
        &self.tasks[self.current_task].lines
    }
    fn lines_mut(&mut self) -> &mut Vec<Option<String>> {
        &mut self.tasks[self.current_task].lines
    }
    fn last_token(&mut self) -> &mut Option<String> {
        &mut self.tkn
    }
    fn set_last_token(&mut self, buffer: String) {
        self.tkn = Some(buffer);
    }
    fn regs(&mut self) -> &mut [usize; 2] {
        &mut self.tasks[self.current_task].regs
    }
    fn s_stack(&mut self) -> &mut Stack<isize> {
        &mut self.tasks[self.current_task].s_stk
    }
    fn r_stack(&mut self) -> &mut Stack<isize> {
        &mut self.tasks[self.current_task].r_stk
    }
    fn c_stack(&mut self) -> &mut Stack<Control> {
        &mut self.tasks[self.current_task].c_stk
    }
    fn f_stack(&mut self) -> &mut Stack<f64> {
        &mut self.tasks[self.current_task].f_stk
    }
    fn wordlist_mut(&mut self) -> &mut Wordlist<Self> {
        &mut self.wordlist
    }
    fn wordlist(&self) -> &Wordlist<Self> {
        &self.wordlist
    }
    fn state(&mut self) -> &mut State {
        &mut self.tasks[self.current_task].state
    }
    fn references(&mut self) -> &mut ForwardReferences {
        &mut self.references
    }
    fn system_time_ns(&self) -> u64 {
        let elapsed = time::now() - self.now;
        match elapsed.num_nanoseconds() {
            Some(d) => d as u64,
            None => 0
        }
    }
    fn current_task(&self) -> usize {
        self.current_task
    }
    fn set_current_task(&mut self, i: usize) {
        if i < NUM_TASKS {
            self.current_task = i;
        } else {
            // Do nothing.
        }
    }
    fn awake(&self, i: usize) -> bool {
        if i < NUM_TASKS {
            self.tasks[i].awake
        } else {
            false
        }
    }
    fn set_awake(&mut self, i: usize, v: bool) {
        if i < NUM_TASKS {
            self.tasks[i].awake = v;
        } else {
            // Do nothing.
        }
    }
    fn bitset(&self) -> &BitSet {
        &self.bitset
    }
    fn bitset_mut(&mut self) -> &mut BitSet {
        &mut self.bitset
    }
    fn labels(&self) -> &Vec<Label> {
        &self.labels
    }
    fn labels_mut(&mut self) -> &mut Vec<Label> {
        &mut self.labels
    }
}

impl Environment for VM {}
impl Facility for VM {}
impl Float for VM {}
impl Units for VM {}
impl HasLoader for VM {}
impl Output for VM {}
impl Tools for VM {}
impl FileAccess for VM {}

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
        for word in matches.free {
            match vm.input_buffer().take() {
                Some(mut buf) => {
                    buf.push_str(&word);
                    buf.push_str(" ");
                    vm.set_input_buffer(buf);
                }
                None => { /* Unreachable */ }
            }
        }
        repl(vm);
    } else {
        print_version();
        println!("Type 'bye' or press Ctrl-D to exit.");
        repl(vm);
    }
}

fn print_version() {
    println!("rtForth v0.6.1, Copyright (C) 2019 Mapacode Inc.");
}

primitive!{fn receive(vm: &mut VM) {
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
    vm.flush_output();
    process::exit(0);
}}

#[inline(never)]
fn repl(vm: &mut VM) {
    let cold = vm.find("COLD").expect("COlD");
    vm.execute_word(cold);
    vm.run();
}

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} [forth words] [options]", program);
    print!("{}", opts.usage(&brief));
}
