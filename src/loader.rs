use std::fs::File;
use std::io::BufReader;
use std::io::BufRead;
use ::core::Core;
use exception::Exception::{
    self,
    FileIOException,
};

pub trait HasLoader : Core {
    fn load(&mut self, path_name: &str) -> Option<Exception> {
        let mut reader = match File::open(&path_name) {
            Err(_) => return Some(FileIOException),
            Ok(file) => {
                BufReader::new(file)
            }
        };
        loop {
            let mut input_buffer = self.input_buffer().take().unwrap();
            input_buffer.clear();
            let result = reader.read_line(&mut input_buffer);
            match result {
                Ok(_) => {
                    if input_buffer.is_empty() {
                        self.set_input_buffer(input_buffer);
                        return None;
                    } else {
                        self.set_input_buffer(input_buffer);
                        self.state().source_index = 0;
                        match self.evaluate() {
                            Some(e) => {
                                return Some(e);
                            },
                            None => {}
                        };
                    }
                },
                Err(_) => {
                  self.set_input_buffer(input_buffer);
                  return Some(FileIOException);
                }
            };
        }
    }
}
