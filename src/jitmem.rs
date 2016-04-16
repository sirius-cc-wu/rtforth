extern crate libc;

use std::mem;
use std::ptr::Unique;
use std::slice;
use std::ascii::AsciiExt;
use std::marker;
use ::exception::Exception;

extern {
    fn memset(s: *mut libc::c_void, c: libc::uint32_t, n: libc::size_t) -> *mut libc::c_void;
}

// JitWord
pub struct JitWord<Target> {
    link: usize,
    symbol: usize,
    is_immediate: bool,
    is_compile_only: bool,
    hidden: bool,
    dfa: usize,
    action: fn(& mut Target) -> Option<Exception>
}

impl<Target> JitWord<Target> {
    pub fn new(symbol: usize, action: fn(& mut Target) -> Option<Exception>) -> JitWord<Target> {
        JitWord {
            link: 0,
            symbol: symbol,
            is_immediate: false,
            is_compile_only: false,
            hidden: false,
            dfa: 0,
            action: action
        }
    }

    pub fn link(&self) -> usize {
        self.link
    }

    pub fn symbol(&self) -> usize {
        self.symbol
    }

    pub fn is_immediate(&self) -> bool {
        self.is_immediate
    }

    pub fn set_immediate(&mut self, flag: bool) {
        self.is_immediate = flag;
    }

    pub fn is_compile_only(&self) -> bool {
        self.is_compile_only
    }

    pub fn set_compile_only(&mut self, flag: bool) {
        self.is_compile_only = flag;
    }

    pub fn is_hidden(&self) -> bool {
        self.hidden
    }

    pub fn set_hidden(&mut self, flag: bool) {
        self.hidden = flag;
    }

    pub fn dfa(&self) -> usize {
        self.dfa
    }

    pub fn action(&self) -> (fn(& mut Target) -> Option<Exception>) {
        self.action
    }

}

const PAGE_SIZE: usize = 4096;

#[allow(dead_code)]
pub struct JitMemory<Target> {
    pub inner: Unique<u8>,
    cap: usize,
    len: usize,
    // last word in current word list
    last: usize,
    marker: marker::PhantomData<Target>,
}

impl<Target> JitMemory<Target> {
    pub fn new(num_pages: usize) -> JitMemory<Target> {
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
            marker: marker::PhantomData,
        }
    }

    // Getter
    pub fn len(&self) -> usize {
        self.len
    }

    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.len == mem::align_of::<usize>()
    }

    pub fn last(&self) -> usize {
        self.last
    }

    pub fn get_u8(&self, addr: usize) -> u8 {
        unsafe { *(self.inner.offset(addr as isize) as *mut u8) }
    }

    #[allow(dead_code)]
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

    pub fn set_last(&mut self, addr: usize) {
        self.last = addr;
    }
    // Basic operations

    pub fn word(&self, pos: usize) -> &JitWord<Target> {
        unsafe {
            &*(self.inner.offset(pos as isize) as *const JitWord<Target>)
        }
    }

    pub fn mut_word(&mut self, pos: usize) -> &mut JitWord<Target> {
        unsafe {
            &mut *(self.inner.offset(pos as isize) as *mut JitWord<Target>)
        }
    }

    pub fn compile_word(&mut self, symbol: usize, action: fn(& mut Target) -> Option<Exception>) {
        unsafe {
            self.align();
            let len = self.len;
            let w = JitWord::new(symbol, action);
            let w1 = self.inner.offset(len as isize) as *mut JitWord<Target>;
            *w1 = w;
            (*w1).link = self.last;
            self.last = len;
            self.len += mem::size_of::<JitWord<Target>>();
            // Dfa
            self.align();
            (*w1).dfa = self.len;
        }
    }

    pub fn find(&mut self, symbol: usize) -> Option<usize>{
        let mut i = self.last();
        while !(i==0) {
            let w = self.word(i);
            if !w.hidden && w.symbol == symbol {
                return Some(i);
            } else {
              i = w.link;
            }
        }
        None
    }

    pub fn put_u8(&mut self, v: u8, pos: usize) {
        unsafe {
            let v1 = self.inner.offset(pos as isize) as *mut u8;
            *v1 = v;
        }
    }

    #[allow(dead_code)]
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
