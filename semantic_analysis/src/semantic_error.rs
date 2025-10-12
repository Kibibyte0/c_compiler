use core::fmt;
use std::ops::Range;

#[derive(Debug)]
pub enum ErrorType {
    DeclaredTwice {
        first: Range<usize>,
        second: Range<usize>,
    },
    UseOfUndeclared(Range<usize>),
    InvalidLeftValue(Range<usize>),
}

#[derive(Debug)]
pub struct SemanticErr {
    formated_error: String,
}

impl SemanticErr {
    pub fn new(err: ErrorType, file_name: &str, source_code: &str) -> Self {
        let formated_error = match err {
            ErrorType::DeclaredTwice { first, second } => {
                Self::format_declared_twice_err(file_name, source_code, &first, &second)
            }
            ErrorType::UseOfUndeclared(span) => {
                Self::format_use_of_undeclared_err(file_name, source_code, &span)
            }
            ErrorType::InvalidLeftValue(span) => {
                Self::format_invalid_left_value_err(file_name, source_code, &span)
            }
        };

        Self { formated_error }
    }

    fn get_line_and_column(source_code: &str, offset: usize) -> (usize, usize) {
        let mut line = 1;
        let mut col = 1;

        for (i, c) in source_code.chars().enumerate() {
            if i == offset {
                break;
            }
            if c == '\n' {
                line += 1;
                col = 1;
            } else {
                col += 1;
            }
        }

        (line, col)
    }

    fn get_line_text(source_code: &str, offset: usize) -> &str {
        let start = source_code[..offset].rfind('\n').map_or(0, |pos| pos + 1);
        let end = source_code[offset..]
            .find('\n')
            .map_or(source_code.len(), |pos| offset + pos);

        &source_code[start..end]
    }

    fn format_message(file_name: &str, source_code: &str, span: &Range<usize>) -> String {
        let (line, col) = Self::get_line_and_column(source_code, span.start);
        let line_text = Self::get_line_text(source_code, span.start);

        let mut marker_line = String::new();
        let marker_start = col.saturating_sub(1);
        let marker_len = (span.end - span.start).max(1);

        for i in 0..=line_text.len() {
            if i == marker_start {
                marker_line.push('^');
                for _ in 1..marker_len {
                    marker_line.push('~');
                }
                break;
            } else if line_text.as_bytes()[i] == b'\t' {
                marker_line.push('\t');
            } else {
                marker_line.push(' ');
            }
        }

        format!(
            "{} --> line {}:{}\n     |\n{:>4} | {}\n     | {}\n",
            file_name, line, col, line, line_text, marker_line
        )
    }

    fn format_declared_twice_err(
        file_name: &str,
        source_code: &str,
        first: &Range<usize>,
        second: &Range<usize>,
    ) -> String {
        format!(
            "variable declared twice\n\
             first declaration:\n{}\
             second declaration:\n{}",
            Self::format_message(file_name, source_code, first),
            Self::format_message(file_name, source_code, second),
        )
    }

    fn format_use_of_undeclared_err(
        file_name: &str,
        source_code: &str,
        span: &Range<usize>,
    ) -> String {
        format!(
            "use of undeclared variable\n{}",
            Self::format_message(file_name, source_code, span)
        )
    }

    fn format_invalid_left_value_err(
        file_name: &str,
        source_code: &str,
        span: &Range<usize>,
    ) -> String {
        format!(
            "invalid left-hand side of assignment\n{}",
            Self::format_message(file_name, source_code, span)
        )
    }
}

impl fmt::Display for SemanticErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.formated_error)
    }
}

impl std::error::Error for SemanticErr {}
