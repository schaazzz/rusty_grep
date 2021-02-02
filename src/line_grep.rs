use regex::{ self, Regex };

pub struct LineGrep {
    re: Regex,
}

impl LineGrep {
    pub fn new(pattern: String) -> Option<LineGrep> {
        if let Ok(re) = Regex::new(pattern.as_str()) {
            println!("re: {:?}", re);
            Some(LineGrep { re: re })
        }
        else {
            None
        }
    }

    pub fn feed(&mut self, line: &String) -> Option<(usize, usize)> {
        if let Some(matched) = self.re.find(line.as_str()) {
            Some((matched.start(), matched.end()))
        }
        else
        {
            None
        }
    }
}
