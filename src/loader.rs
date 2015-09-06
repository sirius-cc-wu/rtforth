use std::fs::File;
use std::io::BufReader;
use std::io::BufRead;
use exception::Exception::FileIOException;

pub trait HasLoader {
    fn load(&mut self, path_name: &str);
}

impl HasLoader for ::core::VM {
    fn load(&mut self, path_name: &str) {
        let mut reader = match File::open(&path_name) {
            Err(_) => panic!("Cannot open file"),
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
                        break;
                    } else {
                        self.source_index = 0;
                        self.evaluate();
                        if self.has_error() {
                            break;
                        }
                    }
                },
                Err(_) => {
                    self.abort_with_error(FileIOException);
                    // println!(_.description()));
                    break;
                }
            }
        }
    }
}
