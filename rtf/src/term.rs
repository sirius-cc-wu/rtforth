use crossterm::{
    cursor,
    event::{read, Event, KeyCode, KeyModifiers},
    terminal, QueueableCommand,
};
use std::io::{stdout, Write};

#[derive(Debug)]
pub enum Error {
    Eof,
    Other,
}

pub struct Term {}

impl Term {
    pub fn new() -> Term {
        terminal::enable_raw_mode().expect("Could not turn on Raw mode");
        Term {}
    }

    pub fn read_line(&mut self) -> Result<String, Error> {
        let mut done = false;
        let mut buffer = String::with_capacity(128);
        let mut stdout = stdout();
        while !done {
            match read() {
                Ok(ev) => match ev {
                    Event::Key(key) => {
                        if key.modifiers == KeyModifiers::NONE {
                            match key.code {
                                KeyCode::Backspace => {
                                    let len = buffer.len();
                                    if len > 0 {
                                        buffer.remove(len - 1);
                                        stdout.queue(cursor::MoveLeft(1));
                                        stdout.flush();
                                    }
                                }
                                KeyCode::Enter => {
                                    done = true;
                                }
                                KeyCode::Char(ch) => {
                                    buffer.push(ch);
                                    print!("{}", ch);
                                    stdout.flush();
                                }
                                _ => {}
                            }
                        } else if key.modifiers == KeyModifiers::CONTROL {
                            if key.code == KeyCode::Char('d') {
                                return Err(Error::Eof);
                            }
                        }
                    }
                    _ => {}
                },
                Err(e) => {
                    return Err(Error::Other);
                }
            }
        }
        Ok(buffer)
    }
}

impl Drop for Term {
    fn drop(&mut self) {
        terminal::disable_raw_mode().expect("Could not disable raw mode")
    }
}
