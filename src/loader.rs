use std::fs::File;
use std::io::BufReader;
use std::io::BufRead;
use exception::Exception::{
    self,
    FileIOException,
};

pub trait HasLoader {
    fn load(&mut self, path_name: &str) -> Option<Exception>;
}

impl HasLoader for ::core::VM {
    fn load(&mut self, path_name: &str) -> Option<Exception> {
        let mut reader = match File::open(&path_name) {
            Err(_) => return Some(FileIOException),
            Ok(file) => {
                BufReader::new(file)
            }
        };
        loop {
            self.input_buffer.clear();
            let result = reader.read_line(&mut self.input_buffer);
            match result {
                Ok(_) => {
                    if self.input_buffer.is_empty() {
                        return None;
                    } else {
                        self.source_index = 0;
                        match self.evaluate() {
                            Some(e) => {
                                return Some(e)
                            },
                            None => {}
                        };
                    }
                },
                Err(_) => return Some(FileIOException)
            };
        }
    }
}
