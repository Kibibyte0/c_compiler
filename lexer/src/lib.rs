use logos::Logos;

// Extras for logos to keep track of line number
#[derive(Debug, Default)]
pub struct LinePosition {
    line_num: usize,
}

// callback function to be used whenever logos match a new line char
// always return None to skip the newline
fn update_line_num(lex: &mut logos::Lexer<Token>) {
    lex.extras.line_num += 1;
}

#[derive(Debug, PartialEq, Logos)]
#[logos(extras = LinePosition)]
pub enum Token {
    // Identifiers: starts with a letter or underscore, followed by letters, digits, or underscores
    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*", priority = 0)]
    Identifier,

    // Integer constants (decimal only)
    #[regex(r"\d+")]
    ConstantInt,

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

    // skipped patterns
    #[regex(r"\n", callback = update_line_num)]
    #[regex(r"[ \t\f]+")]
    #[regex(r"//[^\n]*")]
    #[regex(r"/\*[^*]*\*+([^/*][^*]*\*+)*/")]
    Skip,

    // invalid patterns
    #[regex(r"\d+[a-zA-Z_][a-zA-Z0-9_]*")]
    Error,
} 


#[derive(Debug)]
pub struct SpannedToken<'source> {
    pub token_type: Token,
    pub lexeme: &'source str,
    pub line_num: usize,
    pub col_start: usize,
    pub col_end: usize,
}

impl<'source> Default for SpannedToken<'source> {
    fn default() -> Self {
        Self {
            token_type: Token::Skip,
            lexeme: "",
            line_num: 0,
            col_start: 0,
            col_end: 0,
        }
    }
}

pub struct Lexer<'source> {
    lex: logos::Lexer<'source, Token>,
}

impl<'source> Lexer<'source> {
    pub fn new(input: &'source str) -> Self {
        let extras = LinePosition { line_num: 1 };

        Self {
            lex: Token::lexer_with_extras(input, extras),
        }
    }

pub fn next(&mut self) -> Option<SpannedToken<'source>> {
        loop {
            let token = match self.lex.next()? {
                Ok(tok) => tok,
                _ => Token::Error,
            };

            match token {
                Token::Skip => continue,
                _ => {
                    return Some(SpannedToken {
                        token_type: token,
                        lexeme: self.lex.slice(),
                        line_num: self.lex.extras.line_num,
                        col_start: self.lex.span().start,
                        col_end: self.lex.span().end,
                    });
                }
            }
        }
    }

    pub fn get_line_num(&self) -> usize {
        self.lex.extras.line_num
    }

    pub fn get_span(&self) -> std::ops::Range<usize> {
        self.lex.span()
    }
}