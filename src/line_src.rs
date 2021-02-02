use std::path::Path;
use std::fs::{ metadata, File };
use std::io::{ self, Read, BufRead, BufReader };

pub trait LineSource: Iterator<Item = (String, u32, String)> {
    fn produce(&mut self) -> Option<(String, u32, String)>;
}

pub struct LinesFromStdin<'a> {
    handle: &'a io::Stdin,
    index: u32,
}

impl<'a> LinesFromStdin<'a> {
    pub fn new(handle: &'a io::Stdin) -> LinesFromStdin {
        LinesFromStdin { handle, index: 0 }
    }
}

impl<'a> LineSource for LinesFromStdin<'a> {
    fn produce(&mut self) -> Option<(String, u32, String)> {
        let mut line: String = String::new();
        
        match self.handle.read_line(&mut line) {
            Ok(_) => {
                if line.is_empty() {
                    None
                }
                else {
                    Some(("".to_string(), 0, line.trim().to_string()))
                }
            },
            Err(_) => None
        }
    }
}

impl<'a> Iterator for LinesFromStdin<'a> {
    type Item = (String, u32, String);
    fn next(&mut self) -> Option<(String, u32, String)> {
        self.produce()
    }
}

pub struct LinesFromFiles<'b> {
    files: &'b mut Vec<String>,
    current_filename: String,
    reader: Option<BufReader<File>>,
    index: u32,
}

impl<'b> LinesFromFiles<'b> {
    pub fn new(files: &'b mut Vec<String>) -> LinesFromFiles {
        LinesFromFiles { files, current_filename: "".to_string(), reader: None, index: 0 }
    }

    fn is_file_utf8(&mut self, file_path: &String) -> bool {
        let mut ret_val = false;
        
        if let Ok(mut file) = File::open(file_path) {
            let mut buf = vec![0 as u8; 2 as usize];
            if let Ok(n) = file.read(&mut buf[..]) {
                if n == 2 {
                    if buf[1] == 0xFE && buf[0] == 0xFF {
                        ret_val = false;
                    }
                    else {
                        ret_val = true;
                    }
                }
            }
        }

        ret_val
    }
}

impl<'b> LineSource for LinesFromFiles<'b> {
    fn produce(&mut self) -> Option<(String, u32, String)> {
        loop {
            if let Some(reader) = &mut self.reader {
                if let Some(line) = &reader.lines().next() {
                    self.index += 1;
                    return Some((self.current_filename.to_string(), self.index, line.as_ref().unwrap().to_string()));
                }
                else {
                    self.reader = None;
                }
            }
            else {
                let single_file = self.files.len() == 1;
                let file_path: String = match self.files.pop() {
                    Some(s) => s,
                    None => return None
                };

                let md = metadata(&file_path).unwrap();
                if md.is_dir() {
                    eprintln!("Info: \"{}\" is a directory!", file_path);
                    continue
                }

                if !Path::new(&file_path).exists() {
                    eprintln!("Error: \"{}\" - no such file!", file_path);
                }
                else {
                    if !self.is_file_utf8(&file_path) {
                        eprintln!("Error: \"{}\" is not a UTF-8 encoded file!", file_path);
                        continue
                    }

                    self.current_filename = match single_file {
                        false => Path::new(&file_path).file_name().unwrap().to_str().unwrap().to_string(),
                        true => "".to_string(),
                    };

                    if let Ok(file) = File::open(&file_path) {
                        self.index = 0;
                        self.reader = Some(BufReader::new(file));
                    }
                    else {
                        eprintln!("Error: Can't open \"{}\"!", file_path);
                    }
                }
            }
        }
    }
}

impl<'b> Iterator for LinesFromFiles<'b> {
    type Item = (String, u32, String);
    fn next(&mut self) -> Option<(String, u32, String)> {
        self.produce()
    }
}
