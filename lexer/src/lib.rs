use logos::Logos;

#[derive(Debug, PartialEq, Logos)]
pub enum Token {
    // Keywords
    #[token("return")]
    Return,

    #[token("int")]
    Int,

    #[token("void")]
    Void,

    // Symbols
    #[token("(")]
    LeftParenthesis,

    #[token(")")]
    RightParenthesis,

    #[token("{")]
    LeftCurlyBracket,

    #[token("}")]
    RightCurlyBracket,

    #[token(";")]
    Semicolon,

    // Catch invalid tokens like "9main"
    #[regex(r"\d+[a-zA-Z_][a-zA-Z0-9_]*")]
    Invalid,

    // Integer constants (decimal only)
    #[regex(r"\d+")]
    ConstantInt,

    // Identifiers: starts with a letter or underscore, followed by letters, digits, or underscores
    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*", priority = 0)]
    Identifier,

    // skipped patterns
    #[regex(r"[ \t\n\f]+", logos::skip)]
    #[regex(r"//[^\n]*", logos::skip)]
    #[regex(r"/\*([^*]|\*+[^*/])*\*+/", logos::skip)]
    Error,
}


pub struct Lexer<'source> {
    iter: logos::Lexer<'source, Token>,
    input: &'source str,
    line: usize,
    column: usize,
    position: usize,
}

pub struct Span {
    pub line_num: usize,
    pub col_start: usize,
    pub col_end: usize,
}

pub struct SpannedToken<'source> {
    pub token_type: Token,
    pub lexeme: &'source str,
    pub span: Span,
}

impl<'source> Default for SpannedToken<'source> {
    fn default() -> Self {
        Self {
            token_type: Token::Invalid,
            lexeme: "exmpty",
            span: Span { line_num: 0, col_start: 0, col_end: 0 }
        }
    }
}

impl<'source> Lexer<'source> {
    // initiate the lexer
    pub fn new(input: &'source str) -> Self {
        Self {
            iter: Token::lexer(input),
            input,
            line: 1,
            column: 0,
            position: 0,
        }
    }

    pub fn next(&mut self) -> Option<SpannedToken<'source>> {
        // return None when the iter is empty
        let token_type = match self.iter.next()? {
            Ok(mat) => mat,
            Err(_) => panic!("invalid token: {}", self.iter.slice()),
        };

        // catch invalid tokens
        if let Token::Invalid = token_type {
            panic!("Inavlid Identifier {}", self.iter.slice());
        }

        // update the line and column number
        let start = self.iter.span().start;
        self.count_lines(start);

        // update the current position
        self.position = self.iter.span().end;

        // set the start of the token and end of the token relative to the line 
        let token_column_start = self.column;
        self.column += self.position - start;
        let token_column_end = self.column;

        let spanned_token = SpannedToken {
            token_type,
            lexeme: self.iter.slice(),
            span: Span {
                line_num: self.line,
                col_start: token_column_start,
                col_end: token_column_end
            },
        };

        Some(spanned_token)
    }

    // count all the lines and columns from the last position
    fn count_lines(&mut self, start: usize) {
        for char in self.input[self.position..start].chars() {
            if char == '\n' {
                self.line += 1;
                self.column = 0;
            } else {
                self.column += 1;
            }
        }
    }

    // getters for priveate fields
    pub fn get_source_code(&self) -> &'source str {
        self.input
    }

    pub fn get_line_num(&self) -> usize {
        self.line
    }

    pub fn get_col_num(&self) -> usize {
        self.column
    }
}
