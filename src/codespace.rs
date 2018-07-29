extern crate libc;

use std::mem;
use std::slice;

extern "C" {
    fn memset(s: *mut libc::c_void, c: libc::uint32_t, n: libc::size_t) -> *mut libc::c_void;
}

const PAGE_SIZE: usize = 4096;

#[allow(dead_code)]
pub struct CodeSpace {
    pub(crate) inner: *mut u8,
    cap: usize,
    len: usize,
}

impl CodeSpace {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    pub fn new(num_pages: usize) -> CodeSpace {
        let mut ptr: *mut libc::c_void;
        let size = num_pages * PAGE_SIZE;
        unsafe {
            ptr = mem::uninitialized();
            libc::posix_memalign(&mut ptr, PAGE_SIZE, size);
            libc::mprotect(
                ptr,
                size,
                libc::PROT_EXEC | libc::PROT_READ | libc::PROT_WRITE,
            );
            memset(ptr, 0xc3, size); // prepopulate with 'RET'
        }
        CodeSpace {
            inner: ptr as *mut u8,
            cap: size,
            len: 0,
        }
    }

    #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
    pub fn new(num_pages: usize) -> CodeSpace {
        let mut ptr: *mut libc::c_void;
        let size = num_pages * PAGE_SIZE;
        unsafe {
            ptr = mem::uninitialized();
            libc::posix_memalign(&mut ptr, PAGE_SIZE, size);
            libc::mprotect(
                ptr,
                size,
                libc::PROT_EXEC | libc::PROT_READ | libc::PROT_WRITE,
            );
            memset(ptr, 0x00, size);
        }
        let mut result = CodeSpace {
            inner: ptr as *mut u8,
            cap: size,
            len: 0,
        };
        result
    }

    // Getter

    pub fn capacity(&self) -> usize {
        self.cap
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn has(&self, pos: usize) -> bool {
        let lower_bound = unsafe { self.inner.offset(0) as usize };
        let upper_bound = unsafe { self.inner.offset(self.cap as isize) as usize };
        (lower_bound <= pos) & (pos < upper_bound)
    }

    pub(crate) unsafe fn get_u8(&self, addr: usize) -> u8 {
        *(addr as *mut u8)
    }

    #[allow(dead_code)]
    pub(crate) unsafe fn get_u32(&self, addr: usize) -> u32 {
        *(addr as *mut u32)
    }

    pub(crate) unsafe fn get_i32(&self, addr: usize) -> i32 {
        *(addr as *mut i32)
    }

    pub(crate) unsafe fn get_isize(&self, addr: usize) -> isize {
        *(addr as *mut isize)
    }

    pub(crate) unsafe fn get_f64(&self, addr: usize) -> f64 {
        *(addr as *mut f64)
    }

    pub(crate) unsafe fn get_str(&self, addr: usize, len: usize) -> &str {
        mem::transmute(slice::from_raw_parts::<u8>(addr as *mut u8, len))
    }

    // Basic operations

    pub(crate) unsafe fn put_u8(&mut self, v: u8, pos: usize) {
        *(pos as *mut u8) = v;
    }

    #[allow(dead_code)]
    pub fn compile_u8(&mut self, v: u8) {
        if self.len + mem::size_of::<u8>() <= self.cap {
            let here = self.here();
            unsafe {
                self.put_u8(v, here);
            }
            self.len += mem::size_of::<u8>();
        } else {
            panic!("Error: compile_u8 while code space is full.");
        }
    }

    pub(crate) unsafe fn put_u32(&mut self, v: u32, pos: usize) {
        *(pos as *mut u32) = v;
    }

    pub fn compile_u32(&mut self, v: u32) {
        if self.len + mem::size_of::<u32>() <= self.cap {
            let here = self.here();
            unsafe {
                self.put_u32(v, here);
            }
            self.len += mem::size_of::<u32>();
        } else {
            panic!("Error: compile_u32 while code space is full.");
        }
    }

    pub(crate) unsafe fn put_i32(&mut self, v: i32, pos: usize) {
        *(pos as *mut i32) = v;
    }

    pub fn compile_i32(&mut self, v: i32) {
        if self.len + mem::size_of::<i32>() <= self.cap {
            let here = self.here();
            unsafe {
                self.put_i32(v, here);
            }
            self.len += mem::size_of::<i32>();
        } else {
            panic!("Error: compile_i32 while code space is full.");
        }
    }

    pub(crate) unsafe fn put_f64(&mut self, v: f64, pos: usize) {
        *(pos as *mut f64) = v;
    }

    pub fn compile_f64(&mut self, v: f64) {
        if self.len + mem::size_of::<f64>() <= self.cap {
            let here = self.here();
            unsafe {
                self.put_f64(v, here);
            }
            self.len += mem::size_of::<f64>();
        } else {
            panic!("Error: compile_f64 while code space is full.");
        }
    }

    pub fn compile_str(&mut self, s: &str) {
        let bytes = s.as_bytes();
        if self.len + bytes.len() <= self.cap {
            for byte in bytes {
                self.compile_u8(*byte);
            }
        } else {
            panic!("Error: compile_str while code space is full.");
        }
    }

    pub fn compile_relative(&mut self, f: usize) {
        let there = self.here() + mem::size_of::<u32>();
        let diff = f.wrapping_sub(there) as u32;
        self.compile_u32(diff);
    }

    pub fn here(&mut self) -> usize {
        let len = self.len;
        unsafe { self.inner.offset(len as isize) as usize }
    }

    /// If the code-space pointer is not aligned, reserve enough space to align it.
    pub fn align(&mut self) {
        let align = mem::align_of::<i32>();
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
