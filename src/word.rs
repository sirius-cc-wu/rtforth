use ::core::VM;
use ::exception::Exception;
use std::fmt;

// Word
pub struct Word<'a> {
    pub link: usize,
    pub name: &'a str,
    pub is_immediate: bool,
    pub is_compile_only: bool,
    pub hidden: bool,
    pub dfa: usize,
    pub action: fn(& mut VM) -> Option<Exception>
}

impl<'a> Word<'a> {
    pub fn new(name: &str, dfa: usize, action: fn(& mut VM) -> Option<Exception>) -> Word {
        Word {
            link: 0,
            name: name,
            is_immediate: false,
            is_compile_only: false,
            hidden: false,
            dfa: dfa,
            action: action
        }
    }

    pub fn dfa(&self) -> usize {
        self.dfa
    }

}

impl<'a> fmt::Debug for Word<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,
             "Word {} imm {}, hidden {}, cmponly {}, dfa {}",
             self.name, self.is_immediate, self.hidden, self.is_compile_only, self.dfa)
    }
}
