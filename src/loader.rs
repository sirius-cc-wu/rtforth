use core::Core;
use exception::Exception;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;

pub trait HasLoader: Core {
    fn add_loader(&mut self) {
        self.add_primitive("open-source", HasLoader::open_source);
        self.add_primitive("close-source", HasLoader::close_source);
        self.add_primitive("load-line", HasLoader::p_load_line);
    }

    /// ( file-id -- source-id )
    ///
    /// Open input source from file.
    /// 
    /// Note: different from the Forth 2012 standard, after open, the file
    /// is owned by the input source, the file-id associated with the file is
    /// also gone, so it can no more be used with file access words like
    /// CLOSE-FILE, READ-FILE, WRITE-FILE, RESIZE-FILE...
    /// 
    /// Also note that it is not checked if the file corresponding to file-id is opened read
    /// or opened read-write.
    primitive!{fn open_source(&mut self) {
        let id = self.s_stack().pop();
        if id > 0 && id - 1 < self.files().len() as isize {
            match self.files_mut()[id as usize - 1].take() {
                Some(file) => {
                    let reader = BufReader::new(file);
                    let position = self.readers().iter().position(|x| {
                        x.is_none()
                    });
                    match position {
                        Some(sid) => {
                            self.readers_mut()[sid] = Some(reader);
                            self.s_stack().push(sid as isize + 1);
                        }
                        None => {
                            let sid = self.readers().len() as isize;
                            self.s_stack().push(sid as isize + 1);
                            self.readers_mut().push(Some(reader));
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
    primitive!{fn close_source(&mut self) {
        let id = self.s_stack().pop();
        if id > 0 && id - 1 < self.readers().len() as isize && self.readers()[id as usize - 1].is_some() {
            let _ = self.readers_mut()[id as usize - 1].take();
        } else {
            self.abort_with(Exception::InvalidNumericArgument);
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
        let mut reader = match self.readers_mut()[source_id-1].take() {
            Some(reader) => reader,
            None => {
                return Err(Exception::InvalidNumericArgument);
            }
        };
        let mut line = match self.lines_mut()[source_id-1].take() {
            Some(line) => line,
            None => {
                return Err(Exception::InvalidNumericArgument);
            }
        };
        line.clear();
        let result = match reader.read_line(&mut line) {
            Ok(len) => {
                let not_eof = !(len == 0);
                if line.ends_with('\n') {
                    if line.ends_with('\r') {
                        line.truncate(len-2);
                        Ok((len-2, not_eof))
                    } else {
                        line.truncate(len-1);
                        Ok((len-1, not_eof))
                    }
                } else {
                    Ok((len, not_eof))
                }
            },
            Err(_) => Err(Exception::FileIOException)
        };
        self.readers_mut()[source_id-1] = Some(reader);
        self.lines_mut()[source_id-1] = Some(line);
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