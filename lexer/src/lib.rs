use std::ops::Range;

use logos::Logos;
use token::Token;

pub mod token;

pub struct SpannedToken<'a> {
    token: Token,
    source_code: &'a str,
    file_name: &'a str,
    span: Range<usize>,
}

impl<'a> Default for SpannedToken<'a> {
    fn default() -> Self {
        Self {
            token: Token::Skip,
            source_code: "",
            file_name: "",
            span: Range { start: 0, end: 0 },
        }
    }
}

impl<'a> SpannedToken<'a> {
    pub fn new(token: Token, span: Range<usize>, source_code: &'a str, file_name: &'a str) -> Self {
        Self {
            token,
            span,
            source_code,
            file_name,
        }
    }

    pub fn get_token(&self) -> Token {
        self.token
    }

    pub fn get_span(&self) -> &Range<usize> {
        &self.span
    }

    pub fn get_lexeme(&self) -> &'a str {
        let start = self.get_span().start;
        let end = self.get_span().end;
        &self.source_code[start..end]
    }

    pub fn get_source_code(&self) -> &'a str {
        self.source_code
    }

    pub fn get_file_name(&self) -> String {
        self.file_name.to_string()
    }
}

pub struct Lexer<'a> {
    lex: logos::Lexer<'a, Token>,
    source_code: &'a str,
    file_name: &'a str,
}

impl<'a> Lexer<'a> {
    pub fn new(source_code: &'a str, file_name: &'a str) -> Self {
        Self {
            lex: Token::lexer(source_code),
            source_code,
            file_name,
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
                    return Some(SpannedToken::new(
                        token,
                        self.lex.span(),
                        self.source_code,
                        self.file_name,
                    ));
                }
            }
        }
    }
}
