use std::fs::File;
use std::io::BufReader;
use std::io::BufRead;
use core::Core;
use exception::Exception::FileIOException;

pub trait HasLoader: Core {
    fn load(&mut self, path_name: &str) {
        let mut reader = match File::open(&path_name) {
            Err(_) => {
                self.set_error(Some(FileIOException));
                return;
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
                        return;
                    } else {
                        self.set_input_buffer(input_buffer);
                        self.evaluate();
                        if self.last_error().is_some() {
                            return;
                        }
                    }
                }
                Err(_) => {
                    self.set_input_buffer(input_buffer);
                    self.set_error(Some(FileIOException));
                    return;
                }
            };
        }
    }
}
