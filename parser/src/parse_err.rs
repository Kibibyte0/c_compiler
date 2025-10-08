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
    pub fn new(message: String, token: &SpannedToken) -> Self {
        let (line, column) =
            ParseErr::calculate_token_pos(token.get_source_code(), token.get_span().start)
                .unwrap_or((0, 0));

        ParseErr {
            message,
            file_name: token.get_file_name(),
            line,
            column,
        }
    }

    fn calculate_token_pos(source_code: &str, span_start: usize) -> Option<(usize, usize)> {
        if span_start > source_code.len() {
            return None;
        }

        let mut line_number = 1;
        let mut last_line_start = 0;

        for (i, c) in source_code[..span_start].char_indices() {
            if c == '\n' {
                line_number += 1;
                last_line_start = i + 1;
            }
        }

        Some((line_number, span_start - last_line_start + 1))
    }

    pub fn expected_found(expected: impl ToString, found: &SpannedToken) -> Self {
        ParseErr::new(
            format!(
                "expected: '{}', found '{}'",
                expected.to_string(),
                found.get_lexeme()
            ),
            found,
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
