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

pub struct SpannedToken<'a> {
    pub token: Token,
    pub lexeme: &'a str,
    pub line: usize,
    pub column: usize,
}

impl<'source> Lexer<'source> {
    // initiate the lexer
    pub fn new(input: &'source str) -> Self {
        Self {
            iter: Token::lexer(input),
            input,
            line: 0,
            column: 0,
            position: 0,
        }
    }

    pub fn next(&mut self) -> Option<SpannedToken<'source>> {
        // return None when the iter is empty
        let token = match self.iter.next()? {
            Ok(mat) => mat,
            Err(_) => panic!("invalid token: {}", self.iter.slice()),
        };

        // catch invalid tokens
        if let Token::Invalid = token {
            panic!("Inavlid Identifier {}", self.iter.slice());
        }

        // update the line and column number
        let start = self.iter.span().start;
        self.count_lines(start);

        // update the current position
        self.position = self.iter.span().end;

        let token_column = self.column;
        self.column += self.position - start;

        let spanned_token = SpannedToken {
            token,
            lexeme: self.iter.slice(),
            line: self.line,
            column: token_column,
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
}
