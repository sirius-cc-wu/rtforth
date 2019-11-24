use std::fs::File;
use hibitset::BitSet;
use loader::Source;
use memory::{CodeSpace, DataSpace};
use NUM_TASKS;
use core::{Control, Core, ForwardReferences, Stack, State, Wordlist};
use env::Environment;
use exception::Exception;
use facility::Facility;
use float::Float;
use file_access::FileAccess;
use loader::HasLoader;
use output::Output;
use tools::Tools;
use units::Units;

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
            c_stk: Stack::new(Control::Canary),
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
    last_error: Option<Exception>,
    handler: usize,
    wordlist: Wordlist<VM>,
    data_space: DataSpace,
    code_space: CodeSpace,
    tkn: Option<String>,
    outbuf: Option<String>,
    hldbuf: String,
    references: ForwardReferences,
    now: u64,
    forward_bitset: BitSet,
    resolved_bitset: BitSet,
    labels: Vec<usize>,
}

impl VM {
    /// Create a VM with data and code space size specified
    /// by `data_pages` and `code_pages`.
    pub fn new(data_pages: usize, code_pages: usize) -> VM {
        let mut labels = Vec::with_capacity(LABEL_COUNT as _);
        labels.resize(LABEL_COUNT as _, 0);
        let mut vm = VM {
            current_task: 0,
            tasks: [
                // Only operator task has its own input buffer.
                Task::new_terminal(),
                Task::new_background(),
                Task::new_background(),
                Task::new_background(),
                Task::new_background(),
                Task::new_background(),
                Task::new_background(),
                Task::new_background(),
            ],
            last_error: None,
            handler: 0,
            wordlist: Wordlist::with_capacity(1000),
            data_space: DataSpace::new(data_pages),
            code_space: CodeSpace::new(code_pages),
            tkn: Some(String::with_capacity(64)),
            outbuf: Some(String::with_capacity(128)),
            hldbuf: String::with_capacity(128),
            references: ForwardReferences::new(),
            now: 0,
            forward_bitset: BitSet::with_capacity(LABEL_COUNT),
            resolved_bitset: BitSet::with_capacity(LABEL_COUNT),
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

        vm.load_core_fs();

        vm
    }

    /// Advance time for 1ms.
    pub fn advance(&mut self) {
        self.now += 1_000_000;
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
        self.tasks[self.current_task & (NUM_TASKS-1)].state.source_id
    }
    fn input_buffer(&mut self) -> &mut Option<String> {
        let source_id = self.source_id();
        if source_id > 0 {
            &mut self.lines_mut()[source_id as usize - 1]
        } else {
            &mut self.tasks[self.current_task & (NUM_TASKS-1)].inbuf
        }
    }
    fn set_input_buffer(&mut self, buffer: String) {
        // self.tasks[self.current_task].inbuf = Some(buffer);
        *self.input_buffer() = Some(buffer);
    }
    fn files(&self) -> &Vec<Option<File>> {
        &self.tasks[self.current_task & (NUM_TASKS-1)].files
    }
    fn files_mut(&mut self) -> &mut Vec<Option<File>> {
        &mut self.tasks[self.current_task & (NUM_TASKS-1)].files
    }
    fn sources(&self) -> &Vec<Option<Source>> {
        &self.tasks[self.current_task & (NUM_TASKS-1)].sources
    }
    fn sources_mut(&mut self) -> &mut Vec<Option<Source>> {
        &mut self.tasks[self.current_task & (NUM_TASKS-1)].sources
    }
    fn lines(&self) -> &Vec<Option<String>> {
        &self.tasks[self.current_task & (NUM_TASKS-1)].lines
    }
    fn lines_mut(&mut self) -> &mut Vec<Option<String>> {
        &mut self.tasks[self.current_task & (NUM_TASKS-1)].lines
    }
    fn last_token(&mut self) -> &mut Option<String> {
        &mut self.tkn
    }
    fn set_last_token(&mut self, buffer: String) {
        self.tkn = Some(buffer);
    }
    fn regs(&mut self) -> &mut [usize; 2] {
        &mut self.tasks[self.current_task & (NUM_TASKS-1)].regs
    }
    fn s_stack(&mut self) -> &mut Stack<isize> {
        &mut self.tasks[self.current_task & (NUM_TASKS-1)].s_stk
    }
    fn r_stack(&mut self) -> &mut Stack<isize> {
        &mut self.tasks[self.current_task & (NUM_TASKS-1)].r_stk
    }
    fn c_stack(&mut self) -> &mut Stack<Control> {
        &mut self.tasks[self.current_task & (NUM_TASKS-1)].c_stk
    }
    fn f_stack(&mut self) -> &mut Stack<f64> {
        &mut self.tasks[self.current_task & (NUM_TASKS-1)].f_stk
    }
    fn wordlist_mut(&mut self) -> &mut Wordlist<Self> {
        &mut self.wordlist
    }
    fn wordlist(&self) -> &Wordlist<Self> {
        &self.wordlist
    }
    fn state(&mut self) -> &mut State {
        &mut self.tasks[self.current_task & (NUM_TASKS-1)].state
    }
    fn references(&mut self) -> &mut ForwardReferences {
        &mut self.references
    }
    fn system_time_ns(&self) -> u64 {
        self.now
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
    fn forward_bitset(&self) -> &BitSet {
        &self.forward_bitset
    }
    fn forward_bitset_mut(&mut self) -> &mut BitSet {
        &mut self.forward_bitset
    }
    fn resolved_bitset(&self) -> &BitSet {
        &self.resolved_bitset
    }
    fn resolved_bitset_mut(&mut self) -> &mut BitSet {
        &mut self.resolved_bitset
    }
    fn labels(&self) -> &Vec<usize> {
        &self.labels
    }
    fn labels_mut(&mut self) -> &mut Vec<usize> {
        &mut self.labels
    }
}

impl Environment for VM {}
impl Facility for VM {}
impl Float for VM {}
impl Units for VM {}
impl FileAccess for VM {}
impl HasLoader for VM {}
impl Output for VM {}
impl Tools for VM {}
