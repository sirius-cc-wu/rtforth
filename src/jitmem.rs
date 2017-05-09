extern crate libc;

use std::mem;
use std::ptr::Unique;
use std::slice;
use std::marker;
use std::ops::{Index, IndexMut};

extern "C" {
    fn memset(s: *mut libc::c_void, c: libc::uint32_t, n: libc::size_t) -> *mut libc::c_void;
}

const PAGE_SIZE: usize = 4096;

pub struct SystemVariables {
    null: isize,
    base: isize,
}

impl SystemVariables {
    pub fn base_addr(&self) -> usize {
        (&self.base as *const _ as usize) - (&self.null as *const _ as usize)
    }
}

#[allow(dead_code)]
pub struct DataSpace {
    pub inner: Unique<u8>,
    cap: usize,
    len: usize,
    marker: marker::PhantomData<SystemVariables>,
}

impl DataSpace {
    pub fn new(num_pages: usize) -> DataSpace {
        let mut ptr: *mut libc::c_void;
        let size = num_pages * PAGE_SIZE;
        unsafe {
            ptr = mem::uninitialized();
            libc::posix_memalign(&mut ptr, PAGE_SIZE, size);
            libc::mprotect(ptr, size, libc::PROT_READ | libc::PROT_WRITE);

            memset(ptr, 0x00, size);
        }
        let mut result = DataSpace {
            inner: unsafe { Unique::new(ptr as *mut u8) },
            cap: size,
            len: mem::size_of::<SystemVariables>(),
            marker: marker::PhantomData,
        };
        result.system_variables_mut().null = 0;
        result.system_variables_mut().base = 10;
        result
    }

    // Getter

    pub fn system_variables(&self) -> &SystemVariables {
        unsafe { &*(self.inner.offset(0) as *const SystemVariables) }
    }

    pub fn system_variables_mut(&mut self) -> &mut SystemVariables {
        unsafe { &mut *(self.inner.offset(0) as *mut SystemVariables) }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.len == mem::align_of::<usize>()
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

    pub fn get_isize(&self, addr: usize) -> isize {
        unsafe { *(self.inner.offset(addr as isize) as *mut isize) }
    }

    pub fn get_f64(&self, addr: usize) -> f64 {
        unsafe { *(self.inner.offset(addr as isize) as *mut f64) }
    }

    pub fn get_str(&self, addr: usize, len: usize) -> &str {
        unsafe {
            mem::transmute(slice::from_raw_parts::<u8>(self.inner.offset(addr as isize), len))
        }
    }

    // Basic operations

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


pub struct CodeSpace {
    pub inner: Unique<u8>,
    cap: usize,
    len: usize,
}

impl CodeSpace {
    #[cfg(target_arch = "x86")]
    pub fn new(num_pages: usize) -> CodeSpace {
        let mut ptr: *mut libc::c_void;
        let size = num_pages * PAGE_SIZE;
        unsafe {
            ptr = mem::uninitialized();
            libc::posix_memalign(&mut ptr, PAGE_SIZE, size);
            libc::mprotect(ptr,
                           size,
                           libc::PROT_EXEC | libc::PROT_READ | libc::PROT_WRITE);

            memset(ptr, 0xc3, size);
        }
        let result = CodeSpace {
            inner: unsafe { Unique::new(ptr as *mut u8) },
            cap: size,
            len: 0,
        };
        result
    }

    #[cfg(target_arch = "arm")]
    pub fn new(num_pages: usize) -> CodeSpace {
        unimplemented!()
    }
}

impl Index<usize> for CodeSpace {
    type Output = u8;

    fn index(&self, _index: usize) -> &u8 {
        unsafe { &*self.inner.offset(_index as isize) }
    }
}

impl IndexMut<usize> for CodeSpace {
    fn index_mut(&mut self, _index: usize) -> &mut u8 {
        unsafe { &mut *self.inner.offset(_index as isize) }
    }
}

#[cfg(target_arch = "x86")]
pub fn jit_3() -> (fn() -> i32) {
    let mut jit = CodeSpace::new(1);
    jit[0] = 0x31; // xor %eax, %eax
    jit[1] = 0xc0;
    jit[2] = 0x40; // inc %eax
    jit[3] = 0x40; // inc %eax
    jit[4] = 0x40; // inc %eax

    unsafe { mem::transmute(jit.inner.offset(0)) }
}

#[cfg(target_arch = "arm")]
pub fn jit_3() -> (fn() -> i32) {
    unimplemented!()
}
