use std::ops::Range;

pub struct SourceMap<'a> {
    source_code: &'a str,
    file_name: &'a str,
}

impl<'a> SourceMap<'a> {
    pub fn new(file_name: &'a str, source_code: &'a str) -> Self {
        Self {
            source_code,
            file_name,
        }
    }

    pub fn format_message(&self, message: String, span: Range<usize>, line: usize) -> String {
        let line_text = self.get_line_text(span.start);
        let column = self.get_col_number(span.start);

        let mut marker_line = String::new();
        let marker_start = column.saturating_sub(1);
        let marker_len = (span.end - span.start).max(1);

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
                marker_line.push('\t');
            } else {
                marker_line.push(' ');
            }
        }

        format!(
            "{} --> line {}:{}\n     |\n{:>4} | {}\n     | {}\n",
            self.file_name, line, column, line, line_text, marker_line
        )
    }

    fn get_line_text(&self, offset: usize) -> &str {
        let start = self.source_code[..offset]
            .rfind('\n')
            .map_or(0, |pos| pos + 1);
        let end = self.source_code[offset..]
            .find('\n')
            .map_or(self.source_code.len(), |pos| offset + pos);

        &self.source_code[start..end]
    }

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

    pub fn get_file_name(&self) -> &'a str {
        self.file_name
    }

    pub fn get_source_code(&self) -> &'a str {
        self.source_code
    }
}
