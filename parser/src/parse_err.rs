use lexer::SpannedToken;
use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub struct ParseErr {
    pub message: String,
    pub line: usize,
    pub column: usize,
}

impl ParseErr {
    pub fn new(message: String, line: usize, column: usize) -> Self {
        ParseErr {
            message,
            line,
            column,
        }
    }

    pub fn expected(expected: impl ToString, found: &SpannedToken) -> Self {
        ParseErr::new(
            format!(
                "expected {}, found '{}'",
                expected.to_string(),
                found.lexeme
            ),
            found.line_num,
            found.col_start,
        )
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
