use lexer::SpannedToken;
use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub struct ParseErr {
    message: String,
    file_name: String,
    line: usize,
    column: usize,
}

impl ParseErr {
    pub fn new(message: String, file_name: String, line: usize, column: usize) -> Self {
        ParseErr {
            message,
            file_name,
            line,
            column,
        }
    }

    pub fn expected_found(expected: impl ToString, found: &SpannedToken) -> Self {
        ParseErr::new(
            format!(
                "expected '{}', found '{}'",
                expected.to_string(),
                found.lexeme
            ),
            found.file_name.to_string(),
            found.line_num,
            found.col_start,
        )
    }

    pub fn expected(
        expected: impl ToString,
        file_name: String,
        line: usize,
        column: usize,
    ) -> Self {
        ParseErr::new(
            format!("expected {}", expected.to_string()),
            file_name,
            line,
            column,
        )
    }
}

impl fmt::Display for ParseErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}: Parse error at line {}, column {}: {}",
            self.file_name, self.line, self.column, self.message
        )
    }
}

impl Error for ParseErr {}
