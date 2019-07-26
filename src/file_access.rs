use std::fs::OpenOptions;
use std::io::Read;
use Exception;
use Core;
use Memory;

const PATH_NAME_MAX_LEN: isize = 256;

pub trait FileAccess: Core {
    fn add_file_access(&mut self) {
        self.add_primitive("close-file", FileAccess::close_file);
        self.add_primitive("create-file", FileAccess::create_file);
        self.add_primitive("delete-file", FileAccess::delete_file);
        self.add_primitive("open-file", FileAccess::open_file);
        self.add_primitive("read-file", FileAccess::read_file);
        self.add_primitive("write-file", FileAccess::write_file);
    }

    /// ( fileid -- ior )
    ///
    /// Close the file identified by fileid. ior is the implementation-defined
    /// I/O result code. 
    primitive!{fn close_file(&mut self) {
        let fileid = self.s_stack().pop() as usize;
        if fileid < self.files().len() && self.files()[fileid].is_some() {
            self.files_mut()[fileid] = None;
            self.s_stack().push(0);
        } else {
            self.s_stack().push(Exception::InvalidNumericArgument as _);
        }
    }}

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
    primitive!{fn create_file(&mut self) {
        let (caddr, u, fam) = self.s_stack().pop3();
        if u < 0 || u > PATH_NAME_MAX_LEN {
            self.s_stack().push2(-1, Exception::InvalidNumericArgument as _);
            return;            
        }
        let path_name = unsafe{ self.data_space().str_from_raw_parts(caddr as _, u as _) };
        let mut options = OpenOptions::new();
        match fam {
            0 => {
                // Impossible to create a read-only file.
                self.s_stack().push2(-1, Exception::InvalidNumericArgument as _);
                return;
            },
            1 => {
                options.write(true).truncate(true).create(true);
            }
            2 => {
                options.read(true).write(true).truncate(true).create(true);
            },
            _ => {
                self.s_stack().push2(-1, Exception::InvalidNumericArgument as _);
                return;
            }
        };
        match options.open(&path_name) {
            Err(_) => {
                self.s_stack().push2(-1, Exception::FileIOException as _);
            }
            Ok(file) => {
                let position = self.files_mut().iter().position(|x| {
                    x.is_none()
                });
                match position {
                    Some(p) => {
                        self.files_mut()[p] = Some(file);
                        self.s_stack().push2(p as _, 0);
                    }
                    None => {
                        let fileid = self.files().len() as _;
                        self.s_stack().push2(fileid, 0);
                        self.files_mut().push(Some(file));
                    }
                }
            },
        };
    }}

    /// ( c-addr u -- ior )
    ///
    /// Delete the file named in the character string specified by c-addr u.
    /// ior is the implementation-defined I/O result code.
    primitive!{fn delete_file(&mut self) {
        let (caddr, u) = self.s_stack().pop2();
        if u < 0 || u > PATH_NAME_MAX_LEN {
            self.s_stack().push2(-1, Exception::InvalidNumericArgument as _);
        } else {
            let path_name = unsafe{ self.data_space().str_from_raw_parts(caddr as _, u as _) };
            match std::fs::remove_file(path_name) {
                Err(_) => self.s_stack().push(Exception::FileIOException as _),
                Ok(_) => self.s_stack().push(0)
            }
        }
    }}

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
    primitive!{fn open_file(&mut self) {
        let (caddr, u, fam) = self.s_stack().pop3();
        if u < 0 || u > PATH_NAME_MAX_LEN {
            self.s_stack().push2(-1, Exception::InvalidNumericArgument as _);
            return;            
        }
        let path_name = unsafe{ self.data_space().str_from_raw_parts(caddr as _, u as _) };
        let mut options = OpenOptions::new();
        match fam {
            0 => {
                options.read(true);
            },
            1 => {
                options.write(true);
            }
            2 => {
                options.read(true).write(true);
            },
            _ => {
                self.s_stack().push2(-1, Exception::InvalidNumericArgument as _);
                return;
            }
        };
        match options.open(&path_name) {
            Err(_) => {
                self.s_stack().push2(-1, Exception::FileIOException as _);
            }
            Ok(file) => {
                let position = self.files_mut().iter().position(|x| {
                    x.is_none()
                });
                match position {
                    Some(p) => {
                        self.files_mut()[p] = Some(file);
                        self.s_stack().push2(p as _, 0);
                    }
                    None => {
                        let fileid = self.files().len() as _;
                        self.s_stack().push2(fileid, 0);
                        self.files_mut().push(Some(file));
                    }
                }
            },
        };
    }}

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
    primitive!{fn read_file(&mut self) {
        let (caddr, u1, fileid) = self.s_stack().pop3();
        let fileid = fileid as usize;
        if u1 < 0 {
            self.s_stack().push2(0, Exception::FileIOException as _);
        } else if fileid >= self.files().len() || self.files()[fileid].is_none() {
            self.s_stack().push2(0, Exception::InvalidNumericArgument as _);
        } else {
            let mut file = self.files_mut()[fileid].take().unwrap();
            let mut buf = unsafe{ self.data_space().buffer_mut_from_raw_parts(caddr as _, u1 as _) };
            match file.read(&mut buf) {
                Ok(u2) => {
                    self.s_stack().push2(u2 as _, 0);
                }
                Err(_) => {
                    self.s_stack().push2(0, Exception::FileIOException as _);
                }
            }
            self.files_mut()[fileid] = Some(file);
        }
    }}

    primitive!{fn write_file(&mut self) {
    }}

}