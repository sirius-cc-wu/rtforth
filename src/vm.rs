use ::output::Output;
use ::core::{Result, Core, ForwardReferences, Stack, State};
use ::jitmem::JitMemory;
use ::loader::HasLoader;
use ::tools::Tools;
use ::env::Environment;
use ::facility::Facility;
use ::float::Float;
use exception::Exception;

// Virtual machine
pub struct VM {
    s_stk: Stack<isize>,
    r_stk: Stack<isize>,
    f_stk: Stack<f64>,
    jitmem: JitMemory<VM>,
    inbuf: Option<String>,
    tkn: Option<String>,
    outbuf: Option<String>,
    state: State,
    references: ForwardReferences,
    evals: Option<Vec<fn(&mut VM, token: &str) -> Result<()>>>,
}

impl VM {
    pub fn new(pages: usize) -> VM {
        VM {
            s_stk: Stack::with_capacity(64),
            r_stk: Stack::with_capacity(64),
            f_stk: Stack::with_capacity(16),
            jitmem: JitMemory::new(pages),
            inbuf: Some(String::with_capacity(128)),
            tkn: Some(String::with_capacity(64)),
            outbuf: Some(String::with_capacity(128)),
            state: State::new(),
            references: ForwardReferences::new(),
            evals: None,
        }
    }
}

impl Core for VM {
  fn jit_memory(&mut self) -> &mut JitMemory<Self> { &mut self.jitmem }
  fn jit_memory_const(&self) -> &JitMemory<Self> { &self.jitmem }
  fn output_buffer(&mut self) -> &mut Option<String> { &mut self.outbuf }
  fn set_output_buffer(&mut self, buffer: String) {
    self.outbuf = Some(buffer);
  }
  fn input_buffer(&mut self) -> &mut Option<String> {
    &mut self.inbuf
  }
  fn set_input_buffer(&mut self, buffer: String) {
    self.inbuf = Some(buffer);
  }
  fn last_token(&mut self) -> &mut Option<String> { &mut self.tkn }
  fn set_last_token(&mut self, buffer: String) { self.tkn = Some(buffer); }
  fn s_stack(&mut self) -> &mut Stack<isize> { &mut self.s_stk }
  fn r_stack(&mut self) -> &mut Stack<isize> { &mut self.r_stk }
  fn f_stack(&mut self) -> &mut Stack<f64> { &mut self.f_stk }
  fn state(&mut self) -> &mut State { &mut self.state }
  fn references(&mut self) -> &mut ForwardReferences { &mut self.references }
  fn evaluators(&mut self) -> &mut Option<Vec<fn(&mut Self, token: &str) -> Result<()>>> {
    &mut self.evals
  }
  fn set_evaluators(&mut self, evaluators: Vec<fn(&mut Self, token: &str) -> Result<()>>) {
    self.evals = Some(evaluators)
  }
}

impl Environment for VM {}
impl Facility for VM {}
impl Float for VM {}
impl HasLoader for VM {}
impl Output for VM {}
impl Tools for VM {}
