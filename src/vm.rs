use output::Output;
use core::{Result, Core, Word, ForwardReferences, Stack, State};
use jitmem::DataSpace;
use loader::HasLoader;
use tools::Tools;
use env::Environment;
use facility::Facility;
use float::Float;
use exception::Exception;

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
