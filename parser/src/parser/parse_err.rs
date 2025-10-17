use lexer::SpannedToken;
use shared_context::source_map::SourceMap;
use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub struct ParseErr {
    formated_error: String,
}

impl ParseErr {
    pub fn new(message: String, token: &SpannedToken, source_map: &SourceMap) -> Self {
        Self {
            formated_error: source_map.format_message(message, token.get_span().clone()),
        }
    }

    pub fn expected(expected: impl ToString, found: &SpannedToken, source_map: &SourceMap) -> Self {
        ParseErr::new(
            format!("expected: '{}'", expected.to_string()),
            found,
            source_map,
        )
    }
}

impl fmt::Display for ParseErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.formated_error)
    }
}

impl Error for ParseErr {}
