extern crate libc;

use std::mem;
use std::ptr::Unique;
use std::slice;
use std::str::from_utf8_unchecked;
use ::word::Word;
use ::core::VM;
use ::exception::Exception;

extern {
    fn memset(s: *mut libc::c_void, c: libc::uint32_t, n: libc::size_t) -> *mut libc::c_void;
}

/// Memory Map
const PAGE_SIZE: usize = 4096;

const HALT_OFFSET: isize = 0;
const INPUT_BUFFER_OFFSET: isize = 128;
const OUTPUT_BUFFER_OFFSET: isize = 256;
const LAST_TOKEN_BUFFER_OFFSET: isize = 512;
const DICTIONARY_OFFSET: isize = 576;

const INPUT_BUFFER_LEN: usize = (OUTPUT_BUFFER_OFFSET-INPUT_BUFFER_OFFSET) as usize;
const OUTPUT_BUFFER_LEN: usize = (LAST_TOKEN_BUFFER_OFFSET-OUTPUT_BUFFER_OFFSET) as usize;
const LAST_TOKEN_BUFFER_LEN: usize = (DICTIONARY_OFFSET-LAST_TOKEN_BUFFER_OFFSET) as usize;

struct Buffer {
  data: *const u8,
  len: usize,
  cap: usize,
}

#[allow(dead_code)]
pub struct JitMemory {
    pub inner: Unique<u8>,
    cap: usize,
    len: usize,
    input_buffer: Buffer,
    output_buffer: Buffer,
    last_token_buffer: Buffer,
    source_index: usize,
    // last word in current word list
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
        let inner = unsafe { Unique::new(ptr as *mut u8) };
        let input_data = unsafe{ inner.offset(INPUT_BUFFER_OFFSET) as *const u8 };
        let output_data = unsafe{ inner.offset(OUTPUT_BUFFER_OFFSET) as *const u8 };
        let last_token_data = unsafe{ inner.offset(LAST_TOKEN_BUFFER_OFFSET) as *const u8 };
        let mut result = JitMemory {
            inner: inner,
            cap: size,
            // Space at 0 is reserved for halt.
            len: mem::align_of::<usize>(),
            input_buffer: Buffer { data: input_data, len: 0, cap: INPUT_BUFFER_LEN },
            output_buffer: Buffer { data: output_data, len: 0, cap: OUTPUT_BUFFER_LEN },
            last_token_buffer: Buffer { data: last_token_data, len: 0, cap: LAST_TOKEN_BUFFER_LEN },
            source_index: 0,
            last: 0,
        };
        result.len = DICTIONARY_OFFSET as usize;
        result
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

    pub fn last_token(&self) -> &str {
      let value = unsafe{ slice::from_raw_parts(self.last_token_buffer.data, self.last_token_buffer.len) };
      unsafe{ from_utf8_unchecked(value) }
    }

    pub fn clear_last_token(&mut self) {
      self.last_token_buffer.len = 0;
    }

    pub fn extend_last_token(&mut self, b: u8) {
      if self.last_token_buffer.len == self.last_token_buffer.cap {
        panic!("extend_last_token failed");
      } else {
        let len = self.last_token_buffer.len;
        unsafe{ *(self.last_token_buffer.data.offset(len as isize) as *mut u8) = b };
        self.last_token_buffer.len += 1;
      }
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

    /// Compile a word of action with name in last_token.
    pub fn compile_word(&mut self, action: fn(& mut VM) -> Option<Exception>) {
        unsafe {
            // name
            self.align();
            let ptr = self.inner.offset(self.len as isize);
            let mut len = 0;
            for b in self.last_token().bytes() {
              *ptr.offset(len) = b;
              len += 1;
            }
            self.len += len as usize;
            let s = mem::transmute(slice::from_raw_parts::<u8>(ptr, len as usize));
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
