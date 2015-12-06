extern crate libc;

use std::mem;
use std::ptr::Unique;
use ::word::Word;

extern {
    fn memset(s: *mut libc::c_void, c: libc::uint32_t, n: libc::size_t) -> *mut libc::c_void;
}

const PAGE_SIZE: usize = 4096;

pub struct JitMemory {
    pub inner: Unique<u8>,
    cap: usize,
    len: usize,
    last: usize,
}

impl JitMemory {
    pub fn new(num_pages: usize) -> JitMemory {
        let mut ptr : *mut libc::c_void;
        let size = num_pages * PAGE_SIZE;
        unsafe {
            ptr = mem::uninitialized();
            libc::posix_memalign(&mut ptr, PAGE_SIZE, size);
            libc::mprotect(ptr, size, libc::PROT_EXEC | libc::PROT_READ | libc::PROT_WRITE);

            memset(ptr, 0xcc, size);  // prepopulate with 'int3'
        }
        JitMemory {
            inner: unsafe { Unique::new(ptr as *mut u8) },
            cap: size,
            len: mem::align_of::<usize>(),
            last: 0,
        }
    }

    // Getter
    pub fn len(&self) -> usize {
        self.len
    }

    pub fn last(&self) -> usize {
        self.last
    }

    // Setter
    pub fn reset(&mut self) {
        self.forget_last_word();
        self.len = mem::align_of::<usize>();
    }

    pub fn forget_last_word(&mut self) {
        self.last = 0;
    }

    // Basic operations

    pub fn word(&self, pos: usize) -> &Word {
        unsafe {
            &*(self.inner.offset(pos as isize) as *const Word)
        }
    }

    pub fn mut_word(&mut self, pos: usize) -> &mut Word {
        unsafe {
            &mut *(self.inner.offset(pos as isize) as *mut Word)
        }
    }

    pub fn last_word(&mut self) -> Option<&mut Word> {
        if self.last != 0 {
            let last = self.last;
            Some(self.mut_word(last))
        } else {
            None
        }
    }

    pub fn compile_word(&mut self, w: Word) {
        self.align();
        let len = self.len;
        unsafe {
            let w1 = self.inner.offset(len as isize) as *mut Word;
            *w1 = w;
            (*w1).link = self.last;
            self.last = len;
        }
        self.len = len + mem::size_of::<Word>();
    }

    pub fn compile_u8(&mut self, v: u8) {
        let len = self.len;
        unsafe {
            let v1 = self.inner.offset(len as isize) as *mut u8;
            *v1 = v;
        }
        self.len = len + mem::size_of::<u8>();
    }

    pub fn align(&mut self) {
        let align = mem::align_of::<usize>();
        self.len = (self.len + align - 1) & align.wrapping_neg();
    }

    pub fn truncate(&mut self, i: usize) {
        self.len = i;
    }
}
