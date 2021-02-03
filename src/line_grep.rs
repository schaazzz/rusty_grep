use regex::{ self, Regex, RegexBuilder, Error };

pub struct LineGrep {
    re: Regex,
}

impl LineGrep {
    pub fn new(pattern: String, ignore_case: bool) -> Result<LineGrep, Error> {
        let re: Regex;
        if ignore_case {
            re = RegexBuilder::new(pattern.as_str()).case_insensitive(true).build()?;
        }
        else {
            re = Regex::new(pattern.as_str())?;
        }

        Ok(LineGrep { re: re })
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
