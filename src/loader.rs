use core::Core;
use exception::Exception::FileIOException;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;

pub trait HasLoader: Core {
    fn load_str(&mut self, script: &str) {
        let mut input_buffer = self.input_buffer().take().unwrap();
        input_buffer.clear();
        input_buffer.push_str(script);
        self.state().source_index = 0;
        self.set_input_buffer(input_buffer);
        self.evaluate();
    }

    fn load(&mut self, path_name: &str) {
        let mut reader = match File::open(&path_name) {
            Err(_) => {
                self.abort_with(FileIOException);
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
                    self.abort_with(FileIOException);
                    return;
                }
            };
        }
    }

    fn load_core_fs(&mut self) {
        let libfs = include_str!("../core.fs");
        self.load_str(libfs);
        if self.last_error().is_some() {
            panic!("Error {:?} {:?}", self.last_error().unwrap(), self.last_token());
        }
    }

}
