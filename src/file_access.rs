use std::fs::{self, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use Core;
use Exception;
use Memory;

const PATH_NAME_MAX_LEN: usize = 256;

pub trait FileAccess: Core {
    fn add_file_access(&mut self) {
        self.add_primitive("file-size", FileAccess::file_size);
        self.add_primitive("file-position", FileAccess::file_position);
        self.add_primitive("close-file", FileAccess::close_file);
        self.add_primitive("create-file", FileAccess::create_file);
        self.add_primitive("delete-file", FileAccess::delete_file);
        self.add_primitive("open-file", FileAccess::open_file);
        self.add_primitive("read-file", FileAccess::read_file);
        self.add_primitive("write-file", FileAccess::write_file);
        self.add_primitive("resize-file", FileAccess::resize_file);
        self.add_primitive("reposition-file", FileAccess::reposition_file);
    }

    /// ( fileid -- ud ior )
    ///
    /// ud is the size, in characters, of the file identified by fileid. ior is
    /// the implementation-defined I/O result code. This operation does not
    /// affect the value returned by FILE- POSITION. ud is undefined if ior is
    /// non-zero.
    ///
    /// Note: As rtForth does not support double-length integers, the higher
    /// part of ud is 0. rtForth also does not support unsigned integers, the
    /// maximum value of ud allowed is isize::max_value(). So an exception
    /// ResultOutOfRange will be returned for a file size larger than
    /// isize::max_value().
    fn file_size(&mut self) {
        let fileid = self.s_stack().pop();
        if fileid <= 0 {
            self.s_stack()
                .push3(-1, -1, Exception::InvalidNumericArgument as _);
            return;
        }
        let fileid = fileid as usize - 1;
        if fileid < self.files().len() {
            let ud = match &self.files()[fileid] {
                &Some(ref f) => match f.metadata() {
                    Ok(m) => {
                        let ud = m.len();
                        if ud <= isize::max_value() as u64 {
                            Ok(ud)
                        } else {
                            Err(Exception::ResultOutOfRange)
                        }
                    }
                    Err(_) => Err(Exception::FileIOException),
                },
                &None => Err(Exception::InvalidNumericArgument),
            };
            match ud {
                Ok(ud) => {
                    self.s_stack().push3(ud as isize, 0, 0);
                }
                Err(e) => {
                    self.s_stack().push3(-1, -1, e as _);
                }
            }
        } else {
            self.s_stack()
                .push3(-1, -1, Exception::InvalidNumericArgument as _);
        }
    }

    /// ( fileid -- ud ior )
    ///
    /// ud is the current file position for the file identified by fileid. ior
    /// is the implementation-defined I/O result code. ud is undefined if ior
    /// is non-zero.
    ///
    /// Note: As rtForth does not support double-length integers, the higher
    /// part of ud is 0. rtForth also does not support unsigned integers, the
    /// maximum value of ud allowed is isize::max_value(). So an exception
    /// ResultOutOfRange will be returned for a file position larger than
    /// isize::max_value().
    fn file_position(&mut self) {
        let fileid = self.s_stack().pop();
        if fileid <= 0 {
            self.s_stack()
                .push3(-1, -1, Exception::InvalidNumericArgument as _);
            return;
        }
        let fileid = fileid as usize - 1;
        if fileid < self.files().len() {
            let ud = match &mut self.files_mut()[fileid] {
                &mut Some(ref mut f) => match f.seek(SeekFrom::Current(0)) {
                    Ok(ud) => {
                        if ud <= isize::max_value() as u64 {
                            Ok(ud)
                        } else {
                            Err(Exception::ResultOutOfRange)
                        }
                    }
                    Err(_) => Err(Exception::FileIOException),
                },
                &mut None => Err(Exception::InvalidNumericArgument),
            };
            match ud {
                Ok(ud) => {
                    self.s_stack().push3(ud as isize, 0, 0);
                }
                Err(e) => self.s_stack().push3(-1, -1, e as _),
            }
        } else {
            self.s_stack()
                .push3(-1, -1, Exception::InvalidNumericArgument as _);
        }
    }

    /// ( fileid -- ior )
    ///
    /// Close the file identified by fileid. ior is the implementation-defined
    /// I/O result code.
    fn close_file(&mut self) {
        let fileid = self.s_stack().pop();
        if fileid <= 0 {
            self.s_stack().push(Exception::InvalidNumericArgument as _);
            return;
        }
        let fileid = fileid as usize - 1;
        if fileid < self.files().len() && self.files()[fileid].is_some() {
            self.files_mut()[fileid] = None;
            self.s_stack().push(0);
        } else {
            self.s_stack().push(Exception::InvalidNumericArgument as _);
        }
    }

    /// ( c-addr u fam -- fileid ior )
    ///
    /// Create the file named in the character string specified by c-addr and
    /// u, and open it with file access method fam. The meaning of values of
    /// fam is implementation defined. If a file with the same name already
    /// exists, recreate it as an empty file.
    ///
    /// If the file was successfully created and opened, ior is zero, fileid
    /// is its identifier, and the file has been positioned to the start of
    /// the file.
    ///
    /// Otherwise, ior is the implementation-defined I/O result code and fileid
    /// is undefined.
    fn create_file(&mut self) {
        let (caddr, u, fam) = self.s_stack().pop3();
        let caddr = caddr as usize;
        let u = u as usize;
        if u > PATH_NAME_MAX_LEN {
            self.s_stack()
                .push2(-1, Exception::InvalidNumericArgument as _);
            return;
        }
        let mut options = OpenOptions::new();
        match fam {
            0 => {
                // Impossible to create a read-only file.
                self.s_stack()
                    .push2(-1, Exception::InvalidNumericArgument as _);
                return;
            }
            1 => {
                options.write(true).truncate(true).create(true);
            }
            2 => {
                options.read(true).write(true).truncate(true).create(true);
            }
            _ => {
                self.s_stack()
                    .push2(-1, Exception::InvalidNumericArgument as _);
                return;
            }
        };
        let file = {
            if self.data_space().start() <= caddr && caddr + u <= self.data_space().limit() {
                let path_name = unsafe { self.data_space().str_from_raw_parts(caddr, u) };
                match options.open(&path_name) {
                    Err(_) => Err(Exception::FileIOException),
                    Ok(file) => Ok(file),
                }
            } else {
                Err(Exception::InvalidMemoryAddress)
            }
        };
        match file {
            Err(e) => {
                self.s_stack().push2(-1, e as _);
            }
            Ok(file) => {
                let position = self.files().iter().position(|x| x.is_none());
                match position {
                    Some(p) => {
                        self.files_mut()[p] = Some(file);
                        self.s_stack().push2(p as isize + 1, 0);
                    }
                    None => {
                        let fileid = self.files().len() as isize;
                        self.s_stack().push2(fileid + 1, 0);
                        self.files_mut().push(Some(file));
                    }
                }
            }
        }
    }

    /// ( c-addr u -- ior )
    ///
    /// Delete the file named in the character string specified by c-addr u.
    /// ior is the implementation-defined I/O result code.
    fn delete_file(&mut self) {
        let (caddr, u) = self.s_stack().pop2();
        let caddr = caddr as usize;
        let u = u as usize;
        if u > PATH_NAME_MAX_LEN {
            self.s_stack()
                .push2(-1, Exception::InvalidNumericArgument as _);
        } else {
            let result = {
                if self.data_space().start() <= caddr && caddr + u <= self.data_space().limit() {
                    let path_name = unsafe { self.data_space().str_from_raw_parts(caddr, u) };
                    match fs::remove_file(path_name) {
                        Err(_) => Exception::FileIOException as _,
                        Ok(_) => 0,
                    }
                } else {
                    Exception::InvalidMemoryAddress as _
                }
            };
            self.s_stack().push(result);
        }
    }

    /// ( c-addr u fam -- fileid ior )
    /// Open the file named in the character string specified by c-addr u,
    /// with file access method indicated by fam. The meaning of values of fam
    /// is implementation defined.
    ///
    /// If the file is successfully opened, ior is zero, fileid is its
    /// identifier, and the file has been positioned to the start of the file.
    ///
    /// Otherwise, ior is the implementation-defined I/O result code and fileid
    /// is undefined.
    fn open_file(&mut self) {
        let (caddr, u, fam) = self.s_stack().pop3();
        let caddr = caddr as usize;
        let u = u as usize;
        if u > PATH_NAME_MAX_LEN {
            self.s_stack()
                .push2(-1, Exception::InvalidNumericArgument as _);
            return;
        }
        let mut options = OpenOptions::new();
        match fam {
            0 => {
                options.read(true);
            }
            1 => {
                options.write(true);
            }
            2 => {
                options.read(true).write(true);
            }
            _ => {
                self.s_stack()
                    .push2(-1, Exception::InvalidNumericArgument as _);
                return;
            }
        };
        let file = {
            if self.data_space().start() <= caddr && caddr + u <= self.data_space().limit() {
                let path_name = unsafe { self.data_space().str_from_raw_parts(caddr, u) };
                match options.open(&path_name) {
                    Err(_) => Err(Exception::FileIOException),
                    Ok(file) => Ok(file),
                }
            } else {
                Err(Exception::InvalidMemoryAddress)
            }
        };
        match file {
            Err(e) => {
                self.s_stack().push2(-1, e as _);
            }
            Ok(file) => {
                let position = self.files_mut().iter().position(|x| x.is_none());
                match position {
                    Some(p) => {
                        self.files_mut()[p] = Some(file);
                        self.s_stack().push2(p as isize + 1, 0);
                    }
                    None => {
                        let fileid = self.files().len() as isize;
                        self.s_stack().push2(fileid + 1, 0);
                        self.files_mut().push(Some(file));
                    }
                }
            }
        }
    }

    /// ( c-addr u1 fileid -- u2 ior )
    ///
    /// Read u1 consecutive characters to c-addr from the current position of
    /// the file identified by fileid.
    ///
    /// If u1 characters are read without an exception, ior is zero and u2 is
    /// equal to u1.
    ///
    /// If the end of the file is reached before u1 characters are read, ior is
    /// zero and u2 is the number of characters actually read.
    ///
    /// If the operation is initiated when the value returned by FILE-POSITION
    /// is equal to the value returned by FILE-SIZE for the file identified by
    /// fileid, ior is zero and u2 is zero.
    ///
    /// If an exception occurs, ior is the implementation-defined I/O result
    /// code, and u2 is the number of characters transferred to c-addr without
    /// an exception.
    ///
    /// An ambiguous condition exists if the operation is initiated when the
    /// value returned by FILE-POSITION is greater than the value returned by
    /// FILE-SIZE for the file identified by fileid, or if the requested
    /// operation attempts to read portions of the file not written.
    ///
    /// At the conclusion of the operation, FILE-POSITION returns the next file
    /// position after the last character read.
    fn read_file(&mut self) {
        let (caddr, u1, fileid) = self.s_stack().pop3();
        let caddr = caddr as usize;
        let u1 = u1 as usize;
        let fileid = fileid as usize;
        if fileid == 0 {
            self.s_stack()
                .push2(-1, Exception::InvalidNumericArgument as _);
            return;
        }
        let fileid = fileid - 1;
        if fileid >= self.files().len() || self.files()[fileid].is_none() {
            self.s_stack()
                .push2(0, Exception::InvalidNumericArgument as _);
        } else {
            let mut file = self.files_mut()[fileid].take().unwrap();
            let result = {
                if self.data_space().start() <= caddr && caddr + u1 <= self.data_space().limit() {
                    let mut buf = unsafe { self.data_space().buffer_from_raw_parts_mut(caddr, u1) };
                    file.read(&mut buf).or(Err(Exception::FileIOException as _))
                } else {
                    Err(Exception::InvalidMemoryAddress as _)
                }
            };
            match result {
                Ok(u2) => {
                    self.s_stack().push2(u2 as _, 0);
                }
                Err(e) => {
                    self.s_stack().push2(0, e);
                }
            }
            self.files_mut()[fileid] = Some(file);
        }
    }

    /// ( c-addr u fileid -- ior )
    ///
    /// Write u characters from c-addr to the file identified by fileid
    /// starting at its current position. ior is the implementation-defined I/O
    /// result code.
    ///
    /// At the conclusion of the operation, FILE-POSITION returns the next file
    /// position after the last character written to the file, and FILE-SIZE
    /// returns a value greater than or equal to the value returned by
    /// FILE-POSITION.
    fn write_file(&mut self) {
        let (caddr, u, fileid) = self.s_stack().pop3();
        let caddr = caddr as usize;
        let u = u as usize;
        if fileid <= 0 {
            self.s_stack().push(Exception::InvalidNumericArgument as _);
            return;
        }
        let fileid = fileid as usize - 1;
        if fileid < self.files().len() {
            match self.files_mut()[fileid].take() {
                Some(mut f) => {
                    let result = {
                        if self.data_space().start() <= caddr
                            && caddr + u <= self.data_space().limit()
                        {
                            let buf = unsafe {
                                self.data_space().buffer_from_raw_parts(caddr as _, u as _)
                            };
                            f.write_all(buf).or(Err(Exception::FileIOException))
                        } else {
                            Err(Exception::InvalidMemoryAddress)
                        }
                    };
                    match result {
                        Ok(_) => self.s_stack().push(0),
                        Err(_) => self.s_stack().push(Exception::FileIOException as _),
                    }
                    self.files_mut()[fileid] = Some(f);
                }
                None => {
                    self.s_stack().push(Exception::InvalidNumericArgument as _);
                }
            }
        } else {
            self.s_stack().push(Exception::InvalidNumericArgument as _);
        }
    }

    /// ( ud fileid -- ior )
    ///
    /// Set the size of the file identified by fileid to ud. ior is the
    /// implementation-defined I/O result code.
    ///
    /// If the resultant file is larger than the file before the operation,
    /// the portion of the file added as a result of the operation might not
    /// have been written.
    ///
    /// At the conclusion of the operation, FILE-SIZE returns the value ud and
    /// FILE- POSITION returns an unspecified value.
    ///
    /// Note: As rtForth does not support double-length integers, the higher
    /// part of ud is 0. rtForth also does not support unsigned integers, the
    /// maximum value of ud allowed is isize::max_value(). So an exception
    /// InvalidNumericArgument will be returned for a ud larger than
    /// isize::max_value().
    fn resize_file(&mut self) {
        let (ud_lower, ud_upper, fileid) = self.s_stack().pop3();
        if fileid <= 0 {
            self.s_stack().push(Exception::InvalidNumericArgument as _);
            return;
        }
        let fileid = fileid as usize - 1;
        let ud_lower = ud_lower as usize;
        if ud_upper != 0 {
            self.s_stack().push(Exception::InvalidNumericArgument as _);
        } else if ud_lower > isize::max_value() as usize {
            self.s_stack().push(Exception::InvalidNumericArgument as _);
        } else if fileid >= self.files().len() {
            self.s_stack().push(Exception::InvalidNumericArgument as _);
        } else {
            match self.files_mut()[fileid].take() {
                Some(f) => {
                    match f.set_len(ud_lower as u64) {
                        Ok(_) => {
                            self.s_stack().push(0);
                        }
                        Err(_) => {
                            self.s_stack().push(Exception::FileIOException as _);
                        }
                    }
                    self.files_mut()[fileid] = Some(f);
                }
                None => {
                    self.s_stack().push(Exception::InvalidNumericArgument as _);
                }
            }
        }
    }

    /// ( ud fileid -- ior )
    ///
    /// Reposition the file identified by fileid to ud. ior is the
    /// implementation-defined I/O result code. An ambiguous condition exists
    /// if the file is positioned outside the file boundaries.
    ///
    /// At the conclusion of the operation, FILE-POSITION returns the value ud.
    ///
    /// Note: As rtForth does not support double-length integers, the higher
    /// part of ud is 0. rtForth also does not support unsigned integers, the
    /// maximum value of ud allowed is isize::max_value(). So an exception
    /// InvalidNumericArgument will be returned for a ud larger than
    /// isize::max_value().
    fn reposition_file(&mut self) {
        let (ud_lower, ud_upper, fileid) = self.s_stack().pop3();
        if fileid <= 0 {
            self.s_stack().push(Exception::InvalidNumericArgument as _);
            return;
        }
        let fileid = fileid as usize - 1;
        let ud_lower = ud_lower as usize;
        if ud_upper != 0 {
            self.s_stack().push(Exception::InvalidNumericArgument as _);
        } else if ud_lower > isize::max_value() as usize {
            self.s_stack().push(Exception::InvalidNumericArgument as _);
        } else if fileid >= self.files().len() {
            self.s_stack().push(Exception::InvalidNumericArgument as _);
        } else {
            match self.files_mut()[fileid].take() {
                Some(mut f) => {
                    match f.seek(SeekFrom::Start(ud_lower as u64)) {
                        Ok(_) => {
                            self.s_stack().push(0);
                        }
                        Err(_) => {
                            self.s_stack().push(Exception::FileIOException as _);
                        }
                    }
                    self.files_mut()[fileid] = Some(f);
                }
                None => {
                    self.s_stack().push(Exception::InvalidNumericArgument as _);
                }
            }
        }
    }
}
