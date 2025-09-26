use std::fmt;
use std::error::Error;

#[derive(Debug)]
pub struct ParseErr {
    pub message: String,
    pub line: usize,
    pub column: usize,
}

impl ParseErr {
    pub fn new(message: String, line: usize, column: usize) -> Self {
        ParseErr { message, line, column }
    }
}

impl fmt::Display for ParseErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Parse error at line {}, column {}: {}",
            self.line, self.column, self.message
        )
    }
}

impl Error for ParseErr {}
