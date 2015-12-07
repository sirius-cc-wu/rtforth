extern crate libc;

use std::mem;
use std::ptr::Unique;
use std::slice;
use ::word::Word;
use ::core::VM;
use ::exception::Exception;

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
            libc::mprotect(ptr, size, libc::PROT_READ | libc::PROT_WRITE);

            memset(ptr, 0xcc, size);  // prepopulate with 'int3'
        }
        JitMemory {
            inner: unsafe { Unique::new(ptr as *mut u8) },
            cap: size,
            // Space at 0 is reserved for halt.
            len: mem::align_of::<usize>(),
            last: 0,
        }
    }

    // Getter
    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == mem::align_of::<usize>()
    }

    pub fn last(&self) -> usize {
        self.last
    }

    pub fn get_u8(&self, addr: usize) -> u8 {
        unsafe { *(self.inner.offset(addr as isize) as *mut u8) }
    }

    pub fn get_u32(&self, addr: usize) -> u32 {
        unsafe { *(self.inner.offset(addr as isize) as *mut u32) }
    }

    pub fn get_i32(&self, addr: usize) -> i32 {
        unsafe { *(self.inner.offset(addr as isize) as *mut i32) }
    }

    pub fn get_f64(&self, addr: usize) -> f64 {
        unsafe { *(self.inner.offset(addr as isize) as *mut f64) }
    }

    pub fn get_str(&self, addr: usize, len: usize) -> &str {
        unsafe { mem::transmute(slice::from_raw_parts::<u8>(self.inner.offset(addr as isize), len)) }
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

    pub fn compile_word(&mut self, name: &str, action: fn(& mut VM) -> Option<Exception>) {
        unsafe {
            // name
            self.align();
            let ptr = self.inner.offset(self.len as isize);
            self.compile_str(name);
            let s = mem::transmute(slice::from_raw_parts::<u8>(ptr, name.len()));
            // Word
            self.align();
            let len = self.len;
            let w = Word::new(s, 0, action);
            let w1 = self.inner.offset(len as isize) as *mut Word;
            *w1 = w;
            (*w1).link = self.last;
            self.last = len;
            self.len += mem::size_of::<Word>();
            // Dfa
            self.align();
            (*w1).dfa = self.len;
        }
    }

    pub fn put_u8(&mut self, v: u8, pos: usize) {
        unsafe {
            let v1 = self.inner.offset(pos as isize) as *mut u8;
            *v1 = v;
        }
    }

    pub fn compile_u8(&mut self, v: u8) {
        let len = self.len;
        self.put_u8(v, len);
        self.len += mem::size_of::<u8>();
    }

    pub fn put_u32(&mut self, v: u32, pos: usize) {
        unsafe {
            let v1 = self.inner.offset(pos as isize) as *mut u32;
            *v1 = v;
        }
    }

    pub fn compile_u32(&mut self, v: u32) {
        let len = self.len;
        self.put_u32(v, len);
        self.len += mem::size_of::<u32>();
    }

    pub fn put_i32(&mut self, v: i32, pos: usize) {
        unsafe {
            let v1 = self.inner.offset(pos as isize) as *mut i32;
            *v1 = v;
        }
    }

    pub fn compile_i32(&mut self, v: i32) {
        let len = self.len;
        self.put_i32(v, len);
        self.len += mem::size_of::<i32>();
    }

    pub fn put_f64(&mut self, v: f64, pos: usize) {
        unsafe {
            let v1 = self.inner.offset(pos as isize) as *mut f64;
            *v1 = v;
        }
    }

    pub fn compile_f64(&mut self, v: f64) {
        let len = self.len;
        self.put_f64(v, len);
        self.len += mem::size_of::<f64>();
    }

    pub fn compile_str(&mut self, s: &str) {
        let mut len = self.len;
        let bytes = s.as_bytes();
        unsafe {
            for byte in bytes {
                *self.inner.offset(len as isize) = *byte;
                len += mem::size_of::<u8>();
            }
        }
        self.len += bytes.len();
    }

    pub fn align(&mut self) {
        let align = mem::align_of::<usize>();
        self.len = (self.len + align - 1) & align.wrapping_neg();
    }

    pub fn allot(&mut self, v: isize) {
        let len = (self.len() as isize + v) as usize;
        self.len = len;
    }

    pub fn truncate(&mut self, i: usize) {
        self.len = i;
    }
}
