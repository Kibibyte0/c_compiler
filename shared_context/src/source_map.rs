use crate::Span;

/// Represents the mapping between AST positions and source code positions.
/// Useful for error reporting, so we can highlight exactly where an error occurs.
pub struct SourceMap<'a> {
    source_code: &'a str, // The full source code as a string slice
    file_name: &'a str,   // Name of the file containing the source code
}

impl<'a> SourceMap<'a> {
    /// Creates a new SourceMap
    ///
    /// # Arguments
    /// - `file_name`: name of the source file
    /// - `source_code`: the content of the file
    pub fn new(file_name: &'a str, source_code: &'a str) -> Self {
        Self {
            source_code,
            file_name,
        }
    }

    /// Formats an error message with source context.
    /// Highlights the part of the source code indicated by `span` and appends the `message`.
    ///
    /// Example output:
    /// ```text
    /// file.rs --> line 3:5
    ///      |
    ///  3   | let x = 10;
    ///      |     ^~~ Error message
    /// ```
    pub fn format_message(&self, message: String, span: Span) -> String {
        let line_text = self.get_line_text(span.start); // Get the text of the line containing the error
        let column = self.get_col_number(span.start); // Determine the column number of the error

        let mut marker_line = String::new();
        let marker_start = column.saturating_sub(1); // Column index starts at 0
        let marker_len = (span.end - span.start).max(1); // At least one character should be marked

        // Construct a line with markers (^) and (~) showing the span
        for i in 0..=line_text.len() {
            if i == marker_start {
                marker_line.push('^');
                for _ in 1..marker_len {
                    marker_line.push('~');
                }
                marker_line.push(' ');
                marker_line.push_str(&message);
                break;
            } else if line_text.as_bytes()[i] == b'\t' {
                marker_line.push('\t'); // Preserve tab alignment
            } else {
                marker_line.push(' '); // Otherwise, just add spaces
            }
        }

        // Format the message with file name, line, column, and markers
        format!(
            "{} --> line {}:{}\n     |\n{:>4} | {}\n     | {}\n",
            self.file_name, span.line, column, span.line, line_text, marker_line
        )
    }

    /// Returns the text of the line containing the given offset.
    fn get_line_text(&self, offset: usize) -> &str {
        let start = self.source_code[..offset]
            .rfind('\n')
            .map_or(0, |pos| pos + 1); // Start of line is after previous newline
        let end = self.source_code[offset..]
            .find('\n')
            .map_or(self.source_code.len(), |pos| offset + pos); // End of line is next newline or EOF

        &self.source_code[start..end]
    }

    /// Returns the column number (1-based) corresponding to the given offset in the source
    fn get_col_number(&self, offset: usize) -> usize {
        let source = self.get_source_code();
        let last_newline_index = source[..offset]
            .char_indices()
            .rfind(|&(_, ch)| ch == '\n')
            .map(|(index, _)| index)
            .unwrap_or(0);

        let col = source[last_newline_index..offset].len();
        col
    }

    /// Returns the file name associated with this SourceMap
    pub fn get_file_name(&self) -> &'a str {
        self.file_name
    }

    /// Returns the full source code
    pub fn get_source_code(&self) -> &'a str {
        self.source_code
    }
}
