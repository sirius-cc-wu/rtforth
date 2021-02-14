use std::alloc::{alloc_zeroed, dealloc, Layout};
use std::mem;

const INVALID_EXECUTION_TOKEN: isize = -1;

struct State {
    instruction_pointer: usize,
    word_pointer: usize,
}

struct Word {
    action: fn(&mut VM),
}

impl Word {
    fn action(&self) -> fn(&mut VM) {
        self.action
    }
}
struct DataSpace {
    pub inner: *mut u8,
    layout: Layout,
    cap: usize,
    len: usize,
}

impl DataSpace {
    pub fn new(num_pages: usize) -> Self {
        let cap = num_pages * page_size::get();
        Self::with_capacity(cap)
    }

    pub fn with_capacity(cap: usize) -> Self {
        let ptr: *mut u8;
        let layout = Layout::from_size_align(cap, page_size::get()).unwrap();
        unsafe {
            ptr = alloc_zeroed(layout);
            if ptr.is_null() {
                panic!("Cannot allocate data space");
            }
        }
        let mut result = DataSpace {
            inner: ptr,
            layout,
            cap,
            len: 0,
        };

        result
    }

    fn start(&self) -> usize {
        unsafe { self.inner.offset(0) as usize }
    }

    fn limit(&self) -> usize {
        unsafe { self.inner.offset(self.cap as isize) as usize }
    }

    unsafe fn get_isize(&self, addr: usize) -> isize {
        *(addr as *mut isize)
    }
}

fn drop(vm: &mut VM, ip: isize) {}

struct VM {
    state: State,
    data_space: DataSpace,
    wordlist: Vec<Word>,
}

impl VM {
    fn state(&mut self) -> &mut State {
        &mut self.state
    }

    fn wordlist(&self) -> &Vec<Word> {
        &self.wordlist
    }

    fn data_space(&mut self) -> &DataSpace {
        &self.data_space
    }

    fn abort_with(&mut self, _code: isize) {}

    #[inline(never)]
    fn run(&mut self) {
        let mut ip = self.state().instruction_pointer;
        while self.data_space().start() <= ip
            && ip + mem::size_of::<isize>() <= self.data_space().limit()
        {
            let w = unsafe { self.data_space().get_isize(ip) as usize };
            self.state().instruction_pointer += mem::size_of::<isize>();
            self.execute_word(w);
            ip = self.state().instruction_pointer;
        }
    }

    #[inline(never)]
    fn execute_word(&mut self, i: usize) {
        self.state().word_pointer = i;
        if i < self.wordlist().len() {
            (self.wordlist()[i].action())(self);
        } else {
            self.abort_with(INVALID_EXECUTION_TOKEN);
        }
    }
}

fn main() {
    let mut vm = VM {
        state: State {
            instruction_pointer: 0,
            word_pointer: 0,
        },
        data_space: DataSpace::new(20),
        wordlist: Vec::new(),
    };

    vm.run();

    println!("Hello, world!");
}
