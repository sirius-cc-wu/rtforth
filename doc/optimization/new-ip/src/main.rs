// Plan:
//
// data space:
//
// Switching between call threading and subroutine threading
//
//  |<- call threading  ->|<- subroutine threading ------------------>|<- call thr...
//  +----------+----------+---+---------+-----------------------------+-------------+
//  | action 1 | action 2 | . | asm ... | add length of asm to ip ret | action3 ... |
//  +----------+----------+---+---------+-----------------------------+-------------+
//                          |  ^
//                          +--+
//
//
// vm in ECX, ip in EDX
//
// fn run(vm: &mut VM, ip: usize);
// fn drop(vm: &mut VM, ip: usize);
// fn execute_word(vm: &mut VM, ip: usize);
//
// Two optimization:
// * use assembler to acheive tail call to run the call threading.
// * switch to subroutine threading for inline assembler (jit).
//

use std::alloc::{alloc_zeroed, dealloc, Layout};
use std::mem;

const INVALID_EXECUTION_TOKEN: isize = -1;

struct Word {
    action: extern "fastcall" fn(&mut VM, ip: &mut usize),
}

impl Word {
    fn action(&self) -> extern "fastcall" fn(&mut VM, ip: &mut usize) {
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

struct VM {
    data_space: DataSpace,
    wordlist: Vec<Word>,
    stack: [isize; 256],
    top: u8,
}

impl VM {
    fn wordlist(&self) -> &Vec<Word> {
        &self.wordlist
    }

    fn data_space(&mut self) -> &DataSpace {
        &self.data_space
    }

    fn abort_with(&mut self, _code: isize) {}

    #[inline(never)]
    extern "fastcall" fn run(&mut self, ip: &mut usize) {
        loop {
            let w = unsafe { self.data_space().get_isize(*ip) as usize };
            *ip += 4;
            if w < self.wordlist().len() {
                (self.wordlist()[w].action())(self, ip);
            } else {
                self.abort_with(INVALID_EXECUTION_TOKEN);
            }
        }
    }
}

fn main() {
    let mut vm = VM {
        data_space: DataSpace::new(20),
        wordlist: Vec::new(),
        stack: [0; 256],
        top: 0,
    };

    vm.wordlist.push(Word { action: p_false });
    vm.wordlist.push(Word { action: one });
    vm.wordlist.push(Word { action: one_plus });
    vm.wordlist.push(Word { action: two_star });
    vm.wordlist.push(Word { action: dup });
    vm.wordlist.push(Word { action: swap });
    vm.wordlist.push(Word { action: drop });

    let mut ip = vm.data_space().start();
    vm.run(&mut ip);

    println!("Hello, world!");
}

extern "fastcall" fn p_false(vm: &mut VM, ip: &mut usize) {
    let new_top = vm.top.wrapping_sub(1);
    vm.stack[new_top as usize] = 0;
    vm.top = new_top;
}

extern "fastcall" fn one(vm: &mut VM, ip: &mut usize) {
    let new_top = vm.top.wrapping_sub(1);
    vm.stack[new_top as usize] = 1;
    vm.top = new_top;
}

extern "fastcall" fn one_plus(vm: &mut VM, ip: &mut usize) {
    vm.stack[vm.top as usize] += 1;
}

extern "fastcall" fn two_star(vm: &mut VM, ip: &mut usize) {
    vm.stack[vm.top as usize] *= 2;
}

extern "fastcall" fn dup(vm: &mut VM, ip: &mut usize) {
    let new_top = vm.top.wrapping_sub(1);
    vm.stack[new_top as usize] = vm.stack[vm.top as usize];
    vm.top = new_top;
}

extern "fastcall" fn drop(vm: &mut VM, ip: &mut usize) {
    let new_top = vm.top.wrapping_add(1);
    vm.top = new_top;
}

extern "fastcall" fn swap(vm: &mut VM, ip: &mut usize) {
    let next = vm.top.wrapping_add(1);
    let tmp = vm.stack[next as usize];
    vm.stack[next as usize] = vm.stack[vm.top as usize];
    vm.stack[vm.top as usize] = tmp;
}
