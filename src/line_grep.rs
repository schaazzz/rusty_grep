use regex::Regex;

pub struct LineGrep {
    pattern: String,
    re: Option<Regex>,
}

impl LineGrep {
    pub fn new(pattern: String) -> LineGrep {
        LineGrep { pattern, re: None }
    }

    pub fn feed(&mut self, line: String) -> (u32, u32, bool) {
        println!("input: {}", line);
        (0, 0, false)
    }
}
