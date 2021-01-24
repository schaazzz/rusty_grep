use std::path::Path;
use std::fs::{ self, metadata, File };
use std::io::{ self, Read, BufRead, Seek, SeekFrom, BufReader };

pub trait LineSource: Iterator<Item = String> {
    fn produce(&mut self) -> Option<String>;
}

pub struct LinesFromStdin<'a> {
    handle: &'a io::Stdin,
}

impl<'a> LinesFromStdin<'a> {
    pub fn new(handle: &'a io::Stdin) -> LinesFromStdin {
        LinesFromStdin { handle }
    }
}

impl<'a> LineSource for LinesFromStdin<'a> {
    fn produce(&mut self) -> Option<String> {
        let mut line: String = String::new();
        
        match self.handle.read_line(&mut line) {
            Ok(_) => {
                if line.is_empty() {
                    None
                }
                else {
                    Some(line.trim().to_string())
                }
            },
            Err(_) => None
        }
    }
}

impl<'a> Iterator for LinesFromStdin<'a> {
    type Item = String;
    fn next(&mut self) -> Option<String> {
        self.produce()
    }
}

pub struct LinesFromFiles<'b> {
    files: &'b mut Vec<String>,
    reader: Option<BufReader<File>>,
    //lines: Option<std::io::Lines<std::io::BufReader<std::fs::File>>>,
}

impl<'b> LinesFromFiles<'b> {
    pub fn new(files: &'b mut Vec<String>) -> LinesFromFiles {
        LinesFromFiles { files, reader: None }
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
    fn produce(&mut self) -> Option<String> {
        loop {
            if let Some(reader) = &mut self.reader {
                if let Some(line) = &reader.lines().next() {
                    return Some(line.as_ref().unwrap().to_string());
                }
                else {
                    self.reader = None;
                }
            }
            else {
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

                    if let Ok(file) = File::open(&file_path) {
                        println!("\n\"{}\":", file_path);
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
    type Item = String;
    fn next(&mut self) -> Option<String> {
        self.produce()
    }
}
