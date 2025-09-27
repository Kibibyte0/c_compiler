use logos::Logos;
use token::{LinePosition, Token};

pub mod token;

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
