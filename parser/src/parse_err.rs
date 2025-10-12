use lexer::SpannedToken;
use std::error::Error;
use std::fmt;
use std::ops::Range;

#[derive(Debug)]
pub struct ParseErr {
    formated_error: String,
}

struct FormatConfig<'a> {
    message: String,
    file_name: String,
    source_code: &'a str,
    span: Range<usize>,
    line: usize,
    column: usize,
}

impl ParseErr {
    pub fn new(message: String, token: &SpannedToken) -> Self {
        let (line, column) = token.calculate_token_pos().unwrap_or((0, 0));

        let config = FormatConfig {
            message,
            file_name: token.get_file_name(),
            source_code: token.get_source_code(),
            span: token.get_span().clone(),
            line,
            column,
        };

        Self {
            formated_error: Self::format_message(config),
        }
    }

    pub fn expected(expected: impl ToString, found: &SpannedToken) -> Self {
        ParseErr::new(format!("expected: '{}'", expected.to_string()), found)
    }

    fn get_line_text(source_code: &str, offset: usize) -> &str {
        let start = source_code[..offset].rfind('\n').map_or(0, |pos| pos + 1);
        let end = source_code[offset..]
            .find('\n')
            .map_or(source_code.len(), |pos| offset + pos);

        &source_code[start..end]
    }

    fn format_message(config: FormatConfig) -> String {
        let line_text = Self::get_line_text(&config.source_code, config.span.start);

        let mut marker_line = String::new();
        let marker_start = config.column.saturating_sub(1);
        let marker_len = (config.span.end - config.span.start).max(1);

        for i in 0..=line_text.len() {
            if i == marker_start {
                marker_line.push('^');
                for _ in 1..marker_len {
                    marker_line.push('~');
                }
                marker_line.push(' ');
                marker_line.push_str(&config.message);
                break;
            } else if line_text.as_bytes()[i] == b'\t' {
                marker_line.push('\t');
            } else {
                marker_line.push(' ');
            }
        }

        format!(
            "{} --> line {}:{}\n     |\n{:>4} | {}\n     | {}\n",
            config.file_name, config.line, config.column, config.line, line_text, marker_line
        )
    }
}

impl fmt::Display for ParseErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.formated_error)
    }
}

impl Error for ParseErr {}
