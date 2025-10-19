use core::fmt;
use parser::ast::Span;

use shared_context::source_map::SourceMap;

#[derive(Debug)]
pub enum ErrorType {
    DeclaredTwice { first: Span, second: Span },
    UseOfUndeclared(Span),
    InvalidLeftValue(Span),
    BreakErr(Span),
    ContinueErr(Span),
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
            ErrorType::BreakErr(span) => Self::format_break_error(source_map, &span),
            ErrorType::ContinueErr(span) => Self::format_continue_error(source_map, &span),
        };

        Self { formated_error }
    }

    fn format_declared_twice_err(source_map: &SourceMap, first: &Span, second: &Span) -> String {
        format!(
            "variable declared twice\n\
             first declaration:\n{}\
             second declaration:\n{}",
            source_map.format_message("".to_string(), first.get_range(), first.get_line()),
            source_map.format_message("".to_string(), second.get_range(), second.get_line()),
        )
    }

    fn format_use_of_undeclared_err(source_map: &SourceMap, span: &Span) -> String {
        format!(
            "use of undeclared variable\n{}",
            source_map.format_message("".to_string(), span.get_range(), span.get_line())
        )
    }

    fn format_invalid_left_value_err(source_map: &SourceMap, span: &Span) -> String {
        format!(
            "invalid left-hand side of assignment\n{}",
            source_map.format_message("".to_string(), span.get_range(), span.get_line())
        )
    }

    fn format_break_error(source_map: &SourceMap, span: &Span) -> String {
        format!(
            "break statement can't exists outside of loop\n{}",
            source_map.format_message("".to_string(), span.get_range(), span.get_line())
        )
    }

    fn format_continue_error(source_map: &SourceMap, span: &Span) -> String {
        format!(
            "continue statement can't exists outside of loop\n{}",
            source_map.format_message("".to_string(), span.get_range(), span.get_line())
        )
    }
}

impl fmt::Display for SemanticErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.formated_error)
    }
}

impl std::error::Error for SemanticErr {}
