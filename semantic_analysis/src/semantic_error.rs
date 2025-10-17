use core::fmt;
use shared_context::Span;

use shared_context::source_map::SourceMap;

#[derive(Debug)]
pub enum ErrorType {
    DeclaredTwice { first: Span, second: Span },
    UseOfUndeclared(Span),
    InvalidLeftValue(Span),
}

#[derive(Debug)]
pub struct SemanticErr {
    formated_error: String,
}

impl SemanticErr {
    pub fn new(err: ErrorType, source_map: &SourceMap) -> Self {
        let formated_error = match err {
            ErrorType::DeclaredTwice { first, second } => {
                Self::format_declared_twice_err(source_map, &first, &second)
            }
            ErrorType::UseOfUndeclared(span) => {
                Self::format_use_of_undeclared_err(source_map, &span)
            }
            ErrorType::InvalidLeftValue(span) => {
                Self::format_invalid_left_value_err(source_map, &span)
            }
        };

        Self { formated_error }
    }

    fn format_declared_twice_err(source_map: &SourceMap, first: &Span, second: &Span) -> String {
        format!(
            "variable declared twice\n\
             first declaration:\n{}\
             second declaration:\n{}",
            source_map.format_message("".to_string(), first.get_range()),
            source_map.format_message("".to_string(), second.get_range()),
        )
    }

    fn format_use_of_undeclared_err(source_map: &SourceMap, span: &Span) -> String {
        format!(
            "use of undeclared variable\n{}",
            source_map.format_message("".to_string(), span.get_range())
        )
    }

    fn format_invalid_left_value_err(source_map: &SourceMap, span: &Span) -> String {
        format!(
            "invalid left-hand side of assignment\n{}",
            source_map.format_message("".to_string(), span.get_range())
        )
    }
}

impl fmt::Display for SemanticErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.formated_error)
    }
}

impl std::error::Error for SemanticErr {}
