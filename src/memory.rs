extern crate libc;

use std::alloc::{alloc_zeroed, dealloc, Layout};
use std::marker;
use std::mem;
use std::slice;

pub struct SystemVariables {
    null: isize,
    base: isize,
    compile_comma: isize,
    compile_integer: isize,
    compile_var: isize,
    compile_const: isize,
    compile_fconst: isize,
    compile_float: isize,
}

impl SystemVariables {
    pub fn base_addr(&self) -> usize {
        &self.base as *const _ as usize
    }

    pub fn compile_comma_vector(&self) -> usize {
        &self.compile_comma as *const _ as usize
    }

    pub fn compile_integer_vector(&self) -> usize {
        &self.compile_integer as *const _ as usize
    }

    pub fn compile_var_vector(&self) -> usize {
        &self.compile_var as *const _ as usize
    }

    pub fn compile_const_vector(&self) -> usize {
        &self.compile_const as *const _ as usize
    }

    pub fn compile_fconst_vector(&self) -> usize {
        &self.compile_fconst as *const _ as usize
    }

    pub fn compile_float_vector(&self) -> usize {
        &self.compile_float as *const _ as usize
    }
}

#[allow(dead_code)]
pub struct DataSpace {
    pub inner: *mut u8,
    layout: Layout,
    cap: usize,
    len: usize,
    marker: marker::PhantomData<SystemVariables>,
}

impl DataSpace {
    #[deprecated(
        since = "0.8.0",
        note = "Please use the with_capacity function instead"
    )]
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
}

impl Drop for DataSpace {
    fn drop(&mut self) {
        unsafe {
            dealloc(self.inner, self.layout);
        }
    }
}

impl Memory for DataSpace {
    fn start(&self) -> usize {
        unsafe { self.inner.offset(0) as usize }
    }

    fn limit(&self) -> usize {
        unsafe { self.inner.offset(self.cap as isize) as usize }
    }

    fn capacity(&self) -> usize {
        self.limit() - self.start()
    }

    fn here(&self) -> usize {
        unsafe { self.inner.offset(self.len as isize) as usize }
    }

    fn set_here(&mut self, pos: usize) {
        // here is allowed to be 1 place after the last memory address.
        if self.start() <= pos && pos <= self.limit() {
            let len = pos as isize - self.start() as isize;
            self.len = len as usize;
        }
    }
}

pub trait Memory {
    /// Start address
    fn start(&self) -> usize;

    /// Upper limit of address
    fn limit(&self) -> usize;

    /// Capacity
    fn capacity(&self) -> usize;

    /// Does memory contains addresss `pos`?
    ///
    /// True if self.start() <= pos < self.limit()
    fn has(&self, pos: usize) -> bool {
        self.start() <= pos && pos < self.limit()
    }

    /// Next free space
    fn here(&self) -> usize;

    /// Set next free space.
    fn set_here(&mut self, pos: usize);

    unsafe fn get_u8(&self, addr: usize) -> u8 {
        *(addr as *mut u8)
    }

    unsafe fn get_usize(&self, addr: usize) -> usize {
        *(addr as *mut usize)
    }

    unsafe fn get_isize(&self, addr: usize) -> isize {
        *(addr as *mut isize)
    }

    unsafe fn get_f64(&self, addr: usize) -> f64 {
        *(addr as *mut f64)
    }

    unsafe fn get_str(&self, addr: usize) -> &str {
        let len = self.get_usize(addr);
        let a = addr + mem::size_of::<usize>();
        self.str_from_raw_parts(a, len)
    }

    unsafe fn str_from_raw_parts(&self, addr: usize, len: usize) -> &str {
        mem::transmute(slice::from_raw_parts::<u8>(addr as *const u8, len))
    }

    unsafe fn buffer_from_raw_parts(&self, addr: usize, len: usize) -> &[u8] {
        slice::from_raw_parts::<u8>(addr as *const u8, len)
    }

    unsafe fn buffer_from_raw_parts_mut(&mut self, addr: usize, len: usize) -> &mut [u8] {
        slice::from_raw_parts_mut::<u8>(addr as *mut u8, len)
    }

    // Basic operations

    unsafe fn put_u8(&mut self, v: u8, pos: usize) {
        *(pos as *mut u8) = v;
    }

    #[allow(dead_code)]
    fn compile_u8(&mut self, v: u8) {
        let here = self.here();
        if here < self.limit() {
            unsafe {
                self.put_u8(v, here);
            }
            self.allot(mem::size_of::<u8>() as isize);
        } else {
            panic!("Error: compile_u8 while space is full.");
        }
    }

    unsafe fn put_usize(&mut self, v: usize, pos: usize) {
        *(pos as *mut usize) = v;
    }

    fn compile_usize(&mut self, v: usize) {
        let here = self.here();
        if here + mem::size_of::<usize>() <= self.limit() {
            unsafe {
                self.put_usize(v, here);
            }
            self.allot(mem::size_of::<usize>() as isize);
        } else {
            panic!("Error: compile_usize while space is full.");
        }
    }

    fn compile_relative(&mut self, f: usize) {
        let there = self.here() + mem::size_of::<usize>();
        let diff = f.wrapping_sub(there) as usize;
        self.compile_usize(diff);
    }

    unsafe fn put_relative(&mut self, f: usize, pos: usize) {
        let there = pos + mem::size_of::<usize>();
        let diff = f.wrapping_sub(there) as usize;
        self.put_usize(diff, pos);
    }

    unsafe fn put_isize(&mut self, v: isize, pos: usize) {
        *(pos as *mut isize) = v;
    }

    fn compile_isize(&mut self, v: isize) {
        let here = self.here();
        if here + mem::size_of::<isize>() <= self.limit() {
            unsafe {
                self.put_isize(v, here);
            }
            self.allot(mem::size_of::<isize>() as isize);
        } else {
            panic!("Error: compile_isize while space is full.");
        }
    }
    unsafe fn put_f64(&mut self, v: f64, pos: usize) {
        *(pos as *mut f64) = v;
    }

    fn compile_f64(&mut self, v: f64) {
        let here = self.here();
        if here + mem::size_of::<f64>() <= self.limit() {
            unsafe {
                self.put_f64(v, here);
            }
            self.allot(mem::size_of::<f64>() as isize);
        } else {
            panic!("Error: compile_f64 while space is full.");
        }
    }

    // Put counted string.
    fn put_cstr(&mut self, s: &str, pos: usize) {
        let bytes = s.as_bytes();
        let len = bytes.len().min(255);
        if pos + len + mem::size_of::<usize>() <= self.limit() {
            let mut p = pos;
            unsafe {
                *(p as *mut u8) = len as u8;
            }
            for byte in &bytes[0..len] {
                p += 1;
                unsafe {
                    *(p as *mut u8) = *byte;
                }
            }
        } else {
            panic!("Error: put_cstr while space is full.");
        }
    }

    fn compile_str(&mut self, s: &str) -> usize {
        let bytes = s.as_bytes();
        let here = self.here();
        let len = bytes.len();
        if here + len + mem::size_of::<usize>() <= self.limit() {
            self.compile_usize(len);
            for byte in bytes {
                self.compile_u8(*byte);
            }
            here
        } else {
            panic!("Error: compile_str while space is full.");
        }
    }

    /// First aligned address greater than or equal to `pos`.
    fn aligned(pos: usize) -> usize {
        let align = mem::align_of::<isize>();
        (pos + align - 1) & align.wrapping_neg()
    }

    /// If the data-space pointer is not aligned, reserve enough space to align it.
    fn align(&mut self) {
        let here = self.here();
        self.set_here(Self::aligned(here));
    }

    /// First float-aligned address greater than or equal to `pos`.
    fn aligned_f64(pos: usize) -> usize {
        let align = mem::align_of::<f64>();
        (pos + align - 1) & align.wrapping_neg()
    }

    /// If the data-space pointer is not float-aligned, reserve enough space to align it.
    fn align_f64(&mut self) {
        let here = self.here();
        self.set_here(Self::aligned_f64(here));
    }

    /// First address aligned to 16-byte boundary greater than or equal to `pos`.
    fn aligned_16(pos: usize) -> usize {
        let align = 16;
        (pos + align - 1) & align.wrapping_neg()
    }

    /// If the space pointer is not aligned to 16-byte boundary, reserve enough space to align it.
    fn align_16(&mut self) {
        let here = self.here();
        self.set_here(Self::aligned_16(here));
    }

    fn allot(&mut self, v: isize) {
        let here = (self.here() as isize + v) as usize;
        self.set_here(here);
    }

    fn truncate(&mut self, pos: usize) {
        self.set_here(pos);
    }
}

unsafe impl Send for DataSpace {}
unsafe impl Sync for DataSpace {}
