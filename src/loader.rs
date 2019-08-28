use core::Core;
use output::Output;
use exception::Exception;
use memory::Memory;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;

pub struct Source {
    reader: BufReader<File>,
    path: String,
}

pub trait HasLoader: Core + Output {
    fn add_loader(&mut self) {
        self.add_primitive("open-source", HasLoader::open_source);
        self.add_primitive("close-source", HasLoader::close_source);
        self.add_primitive(".source-path", HasLoader::dot_source_path);
        self.add_primitive("load-line", HasLoader::p_load_line);
        self.add_primitive(".source-line", HasLoader::dot_source_line);
    }

    /// ( c-addr u file-id -- source-id )
    ///
    /// Open input source from file.
    ///
    /// The source path has length u and is stored in the data space starts
    /// with c-addr.
    ///
    /// Note: different from the Forth 2012 standard, after open, the file
    /// is owned by the input source, the file-id associated with the file is
    /// also gone, so it can no more be used with file access words like
    /// CLOSE-FILE, READ-FILE, WRITE-FILE, RESIZE-FILE...
    ///
    /// Also note that it is not checked if the file corresponding to file-id is opened read
    /// or opened read-write.
    primitive!{fn open_source(&mut self) {
        let (caddr, u, id )= self.s_stack().pop3();
        if id > 0 && id - 1 < self.files().len() as isize {
            match self.files_mut()[id as usize - 1].take() {
                Some(file) => {
                    let position = self.sources().iter().position(|x| {
                        x.is_none()
                    });
                    let reader = BufReader::new(file);
                    let path = String::from( unsafe {
                            self.data_space().str_from_raw_parts(caddr as _, u as _)
                        });
                    match position {
                        Some(sid) => {
                            self.sources_mut()[sid] = Some(Source {reader, path});
                            self.s_stack().push(sid as isize + 1);
                        }
                        None => {
                            let sid = self.sources().len() as isize;
                            self.s_stack().push(sid as isize + 1);
                            self.sources_mut().push(Some(Source {reader, path}));
                            self.lines_mut().push(Some(String::with_capacity(128)));
                        }
                    }
                }
                None => {
                    self.abort_with(Exception::InvalidNumericArgument);
                }
            }
        } else {
            self.abort_with(Exception::InvalidNumericArgument);
        }
    }}

    /// ( source-id -- )
    ///
    /// Close input source.
    ///
    /// The file owned by the resource is also closed.
    ///
    /// Failed if the source-id is the current source id or if the source-id doesn't exist.
    primitive!{fn close_source(&mut self) {
        let id = self.s_stack().pop();
        if self.source_id() == id {
            self.abort_with(Exception::InvalidNumericArgument);
        } else {
            if id > 0 && id - 1 < self.sources().len() as isize
                && self.sources()[id as usize - 1].is_some() {
                let _ = self.sources_mut()[id as usize - 1].take();
            } else {
                self.abort_with(Exception::InvalidNumericArgument);
            }
        }
    }}

    /// ( source-id -- )
    primitive!{fn dot_source_path(&mut self) {
        let id = self.s_stack().pop();
        if id > 0 && id - 1 < self.sources().len() as isize {
            let source = self.sources_mut()[id as usize - 1].take();
            match source {
                Some(s) => {
                    self.push_output(&s.path);
                    self.sources_mut()[id as usize - 1] = Some(s);
                }
                None => {
                    self.abort_with(Exception::InvalidNumericArgument)
                }
            }
        } else {
            self.abort_with(Exception::InvalidNumericArgument)
        }
    }}

    /// ( source-id -- )
    primitive!{fn dot_source_line(&mut self) {
        let id = self.s_stack().pop();
        if id > 0 && id - 1 < self.lines().len() as isize {
            let line = self.lines_mut()[id as usize - 1].take();
            match line {
                Some(s) => {
                    self.push_output(&s);
                    self.lines_mut()[id as usize - 1] = Some(s);
                }
                None => {
                    self.abort_with(Exception::InvalidNumericArgument)
                }
            }
        } else {
            self.abort_with(Exception::InvalidNumericArgument)
        }
    }}

    /// ( source-id -- count not-eof? )
    ///
    /// Load one line from source to input buffer.
    primitive!{fn p_load_line(&mut self) {
        let id = self.s_stack().pop() as usize;
        match self.load_line(id) {
            Err(e) => self.abort_with(e),
            Ok((len, not_eof)) => {
                self.s_stack().push2(len as isize, if not_eof { -1 } else { 0 });
            }
        }
    }}

    /// Load a line from file into input buffer.
    ///
    /// Returns Ok((length, not-eof)) if successful.
    fn load_line(&mut self, source_id: usize) -> Result<(usize, bool), Exception> {
        // Read line
        if !(source_id > 0 && source_id - 1 < self.sources().len()) {
            return Err(Exception::InvalidNumericArgument);
        }
        let mut source = match self.sources_mut()[source_id - 1].take() {
            Some(s) => s,
            None => {
                return Err(Exception::InvalidNumericArgument);
            }
        };
        let mut line = match self.lines_mut()[source_id - 1].take() {
            Some(line) => line,
            None => {
                self.sources_mut()[source_id - 1] = Some(source);
                return Err(Exception::InvalidNumericArgument);
            }
        };
        line.clear();
        let result = match source.reader.read_line(&mut line) {
            Ok(len) => {
                let not_eof = !(len == 0);
                if line.ends_with('\n') {
                    line.truncate(len - 1);
                    if line.ends_with('\r') {
                        line.truncate(len - 2);
                        Ok((len - 2, not_eof))
                    } else {
                        Ok((len - 1, not_eof))
                    }
                } else {
                    Ok((len, not_eof))
                }
            }
            Err(_) => Err(Exception::FileIOException),
        };
        self.lines_mut()[source_id - 1] = Some(line);
        self.sources_mut()[source_id - 1] = Some(source);
        result
    }

    fn load_str(&mut self, script: &str) {
        let mut input_buffer = self.input_buffer().take().unwrap();
        input_buffer.clear();
        input_buffer.push_str(script);
        self.state().source_index = 0;
        self.set_input_buffer(input_buffer);
        self.evaluate_input();
    }

    fn load(&mut self, path_name: &str) -> Result<(), Exception> {
        let mut reader = match File::open(&path_name) {
            Err(_) => {
                return Err(Exception::FileIOException);
            }
            Ok(file) => BufReader::new(file),
        };
        loop {
            let mut input_buffer = self.input_buffer().take().unwrap();
            input_buffer.clear();
            self.state().source_index = 0;
            let result = reader.read_line(&mut input_buffer);
            match result {
                Ok(_) => {
                    if input_buffer.is_empty() {
                        self.set_input_buffer(input_buffer);
                        return Ok(());
                    } else {
                        self.set_input_buffer(input_buffer);
                        self.evaluate_input();
                        if let Some(e) = self.last_error() {
                            return Err(e);
                        }
                    }
                }
                Err(_) => {
                    self.set_input_buffer(input_buffer);
                    return Err(Exception::FileIOException);
                }
            };
        }
    }

    fn load_core_fs(&mut self) {
        let libfs = include_str!("../core.fs");
        self.load_str(libfs);
        if self.last_error().is_some() {
            panic!(
                "Error {:?} {:?}",
                self.last_error().unwrap(),
                self.last_token()
            );
        }
    }
}
