use std::ops::Range;

use logos::Logos;
use token::Token;

pub mod token;

#[derive(Clone)]
pub struct SpannedToken<'a> {
    token: Token,
    lexeme: &'a str,
    span: Range<usize>,
}

impl<'a> Default for SpannedToken<'a> {
    fn default() -> Self {
        Self {
            token: Token::Skip,
            lexeme: "",
            span: Range { start: 0, end: 0 },
        }
    }
}

impl<'a> SpannedToken<'a> {
    pub fn new(token: Token, lexeme: &'a str, span: Range<usize>) -> Self {
        Self {
            token,
            lexeme,
            span,
        }
    }

    pub fn get_token(&self) -> Token {
        self.token
    }

    pub fn get_span(&self) -> Range<usize> {
        self.span.clone()
    }

    pub fn get_lexeme(&self) -> &'a str {
        self.lexeme
    }
}

pub struct Lexer<'a> {
    lex: logos::Lexer<'a, Token>,
}

impl<'a> Lexer<'a> {
    pub fn new(source_code: &'a str) -> Self {
        Self {
            lex: Token::lexer(source_code),
        }
    }

    pub fn next(&mut self) -> Option<SpannedToken<'a>> {
        loop {
            let token = match self.lex.next()? {
                Ok(tok) => tok,
                Err(_) => Token::Error,
            };

            match token {
                Token::Skip => continue,
                _ => {
                    return Some(SpannedToken::new(token, self.lex.slice(), self.lex.span()));
                }
            }
        }
    }
}
