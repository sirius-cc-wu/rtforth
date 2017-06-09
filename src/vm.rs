use output::Output;
use core::{Core, Word, ForwardReferences, Stack, State, Control};
use dataspace::DataSpace;
use codespace::CodeSpace;
use loader::HasLoader;
use tools::Tools;
use env::Environment;
use facility::Facility;
use float::Float;
use exception::Exception;

// Virtual machine
pub struct VM {
    last_error: Option<Exception>,
    handler: usize,
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
    state: State,
    references: ForwardReferences,
}

impl VM {
    pub fn new(data_pages: usize, code_pages: usize) -> VM {
        let mut vm = VM {
            last_error: None,
            handler: 0,
            s_stk: Stack::new(0x12345678),
            r_stk: Stack::new(0x12345678),
            c_stk: Stack::new(Control::Canary),
            f_stk: Stack::new(1.234567890),
            symbols: vec![],
            last_definition: 0,
            wordlist: vec![],
            data_space: DataSpace::new(data_pages),
            code_space: CodeSpace::new(code_pages),
            inbuf: Some(String::with_capacity(128)),
            tkn: Some(String::with_capacity(64)),
            outbuf: Some(String::with_capacity(128)),
            state: State::new(),
            references: ForwardReferences::new(),
        };
        vm.add_core();
        vm.add_output();
        vm.add_tools();
        vm.add_environment();
        vm.add_facility();
        vm.add_float();
        vm.patch_compilation_semanticses();
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
}

impl Environment for VM {}
impl Facility for VM {}
impl Float for VM {}
impl HasLoader for VM {}
impl Output for VM {}
impl Tools for VM {}
