use crossterm::{
    cursor::{self, MoveTo},
    event::{read, Event, KeyCode, KeyModifiers},
    queue,
    terminal::{self, Clear, ClearType},
};
use directories::ProjectDirs;
use std::{
    fs,
    io::{self, stdout, BufRead, LineWriter, Write},
};
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

#[derive(Debug)]
pub enum Error {
    Eof,
    Other,
}

pub struct Term {
    history: Vec<String>,
}

impl Term {
    pub fn new() -> Term {
        terminal::enable_raw_mode().expect("Could not turn on Raw mode");
        let mut opt_history: Option<Vec<String>> = None;
        if let Some(dirs) = ProjectDirs::from("", "", "rtforth") {
            let mut path = dirs.cache_dir().to_path_buf();
            path.push("history.txt");
            if let Ok(file) = fs::File::open(path) {
                let lines = io::BufReader::new(file).lines();
                opt_history = Some(
                    lines
                        .map(|x| if let Ok(s) = x { s } else { String::new() })
                        .collect(),
                );
            }
        }
        let history;
        match opt_history {
            Some(h) => history = h,
            None => history = Vec::with_capacity(32),
        }
        Term { history }
    }

    pub fn read_line(&mut self) -> Result<String, Error> {
        let mut done = false;
        let mut buffer = String::with_capacity(128);
        let mut stdout = stdout();
        let mut h = self.history.len();
        while !done {
            match read() {
                Ok(ev) => match ev {
                    Event::Key(key) => {
                        if key.modifiers == KeyModifiers::NONE
                            || key.modifiers == KeyModifiers::SHIFT
                        {
                            match key.code {
                                KeyCode::Backspace => {
                                    let buf = if h == self.history.len() {
                                        &mut buffer
                                    } else {
                                        &mut self.history[h]
                                    };

                                    let len = buf.len();
                                    if len > 0 {
                                        let mut boundary = len - 1;
                                        while !buf.is_char_boundary(boundary) {
                                            boundary -= 1;
                                        }
                                        buf.remove(boundary);
                                    }
                                }
                                KeyCode::Enter => {
                                    if h != self.history.len() {
                                        buffer.clear();
                                        buffer.push_str(&self.history[h]);
                                    }
                                    done = true;
                                }
                                KeyCode::Char(ch) => {
                                    let buf = if h == self.history.len() {
                                        &mut buffer
                                    } else {
                                        &mut self.history[h]
                                    };
                                    buf.push(ch);
                                }
                                KeyCode::Up => {
                                    if h != 0 {
                                        h -= 1;
                                    }
                                }
                                KeyCode::Down => {
                                    if h < self.history.len() {
                                        h += 1;
                                    }
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
                Err(_) => {
                    return Err(Error::Other);
                }
            }

            let buf = if h == self.history.len() {
                &mut buffer
            } else {
                &mut self.history[h]
            };
            let p = cursor::position().unwrap();
            let width = buf.width();
            queue!(stdout, Clear(ClearType::CurrentLine), MoveTo(0, p.1)).unwrap();
            print!("{}", buf);
            stdout.flush().unwrap();
            queue!(stdout, MoveTo(width as u16 + 1, p.1)).unwrap();
        }
        self.history.push(buffer.clone());
        Ok(buffer)
    }
}

impl Drop for Term {
    fn drop(&mut self) {
        print!("\r");
        terminal::disable_raw_mode().expect("Could not disable raw mode");
        if let Some(dirs) = ProjectDirs::from("", "", "rtforth") {
            let mut path = dirs.cache_dir().to_path_buf();
            if fs::create_dir_all(&path).is_ok() {
                path.push("history.txt");
                match fs::File::create(&path) {
                    Ok(file) => {
                        let mut file = LineWriter::new(file);
                        // Keep at most 32 lines.
                        let mut start = self.history.len();
                        if start > 32 {
                            start = start - 32;
                        } else {
                            start = 0;
                        }
                        for line in &self.history[start..] {
                            if line.len() > 0 {
                                file.write_all(line.as_bytes()).unwrap();
                                file.write(b"\n").unwrap();
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("create file failed: {:?}", e);
                    }
                }
            }
        }
    }
}
