
pub struct ParseErr {
    message: String,
    line_num: usize,
    col_num: usize,
}

impl ParseErr {
    pub fn new(message: String, line_num: usize, col_num: usize) -> ParseErr {
        ParseErr {
            message,
            line_num,
            col_num,
        }
    }

    pub fn report(&self, source_code: &str) {
        eprintln!(
            "error: {}, line: {}, column {}",
            self.message, self.line_num, self.col_num
        );

        if let Some(line) = source_code.lines().nth(self.line_num - 1) {
            eprintln!("{}", line);
            eprintln!("{:>width$}^", "", width = self.col_num);
            panic!();
        }
    }
}