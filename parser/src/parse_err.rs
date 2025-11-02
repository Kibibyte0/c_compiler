use lexer::SpannedToken;
use shared_context::{Span, source_map::SourceMap};
use std::error::Error;
use std::fmt;

/// Represents an error encountered during parsing.
///
/// A ParseEr holds a formatted, human-readable error message that
/// includes source location information.  
/// It integrates with SourceMap to provide context such as line and
/// column numbers for display.
#[derive(Debug)]
pub struct ParseErr {
    formated_error: String,
}

impl ParseErr {
    /// Creates a new ParseEr from a raw error message and token context.
    ///
    /// The message is formatted using the SourceMap to include the
    /// token’s position and surrounding source information.
    pub fn new(message: &str, span: Span, source_map: &SourceMap) -> Self {
        Self {
            formated_error: source_map.format_message(message, span),
        }
    }

    /// Constructs a standardized “expected …” parse error message.
    ///
    /// Typically used when the parser encounters an unexpected token.
    /// For example:
    /// ```
    /// expected: 'identifier'
    /// ```
    pub fn expected(expected: impl ToString, found: &SpannedToken, source_map: &SourceMap) -> Self {
        ParseErr::new(
            &format!("expected: '{}'", expected.to_string()),
            found.get_span(),
            source_map,
        )
    }
}

impl fmt::Display for ParseErr {
    /// Returns the formatted error message for display.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.formated_error)
    }
}

impl Error for ParseErr {}
