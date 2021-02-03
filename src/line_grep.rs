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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn non_regex_case_sensitive() {
        let test_string = "This is a test string...".to_string();
        let mut grep = LineGrep::new(r"test string".to_string(), false).unwrap();
        let (start, end) = grep.feed(&test_string).unwrap_or_else(|| (0, 0) );
        assert_eq!((start, end), (10, 21));
    }

    #[test]
    fn non_regex_case_insensitive() {
        let test_string = "This is a test string...".to_string();
        let mut grep = LineGrep::new(r"tEsT sTrInG".to_string(), true).unwrap();
        let (start, end) = grep.feed(&test_string).unwrap_or_else(|| (0, 0) );
        assert_eq!((start, end), (10, 21));
    }
    #[test]
    fn regex_case_sensitive() {
        let test_string = "This is a test string...".to_string();
        let mut grep = LineGrep::new(r"^.+:?((is ){2}).+$".to_string(), false).unwrap();
        let (start, end) = grep.feed(&test_string).unwrap_or_else(|| (0, 0) );
        assert_eq!((start, end), (0, test_string.len()));
    }

    #[test]
    fn regex_case_insensitive() {
        let test_string = "This is a test string...".to_string();
        let mut grep = LineGrep::new(r"^.+:?((IS ){2}).+$".to_string(), true).unwrap();
        let (start, end) = grep.feed(&test_string).unwrap_or_else(|| (0, 0) );
        assert_eq!((start, end), (0, test_string.len()));    }
}