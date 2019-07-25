use std::fs::OpenOptions;
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
        self.add_primitive("read-line", FileAccess::read_line);
        self.add_primitive("write-file", FileAccess::write_file);
        self.add_primitive("write-line", FileAccess::write_line);
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
                let fileid = self.files().len() as _;
                self.s_stack().push2(fileid, 0);
                self.files_mut().push(Some(file));
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
                let fileid = self.files().len() as _;
                self.s_stack().push2(fileid, 0);
                self.files_mut().push(Some(file));
            },
        };
    }}

    primitive!{fn read_file(&mut self) {
    }}

    primitive!{fn read_line(&mut self) {
    }}

    primitive!{fn write_file(&mut self) {
    }}

    primitive!{fn write_line(&mut self) {
    }}

}