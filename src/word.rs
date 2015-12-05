use ::core::VM;
use ::exception::Exception;
use std::fmt;

// Word
pub struct Word {
    pub link: usize,
    pub is_immediate: bool,
    pub is_compile_only: bool,
    pub hidden: bool,
    pub nfa: usize,
    pub dfa: usize,
    pub name_len: usize,
    pub action: fn(& mut VM) -> Option<Exception>
}

impl Word {
    pub fn new(nfa: usize, name_len: usize, dfa: usize, action: fn(& mut VM) -> Option<Exception>) -> Word {
        Word {
            link: 0,
            is_immediate: false,
            is_compile_only: false,
            hidden: false,
            nfa: nfa,
            dfa: dfa,
            name_len: name_len,
            action: action
        }
    }

    pub fn nfa(&self) -> usize {
        self.nfa
    }

    pub fn dfa(&self) -> usize {
        self.dfa
    }

    pub fn name_len(&self) -> usize {
        self.name_len
    }

}
impl fmt::Debug for Word {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,
             "Word imm {}, hidden {}, cmponly {}, nfa {}, dfa {}",
             self.is_immediate, self.hidden, self.is_compile_only, self.nfa, self.dfa)
    }
}
