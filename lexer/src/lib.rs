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
    // keeping tack of the end pos of the previous token relative to the input
    // String helps with calculating the col number relative to the current line
    prev_token_end_pos: usize,
}

impl<'source> Lexer<'source> {
    pub fn new(input: &'source str) -> Self {
        let extras = LinePosition {
            line_num: 1,
            col_num: 1,
        };

        Self {
            lex: Token::lexer_with_extras(input, extras),
            prev_token_end_pos: 0,
        }
    }

    pub fn next(&mut self) -> Option<SpannedToken<'source>> {
        loop {
            let token = match self.lex.next()? {
                Ok(tok) => tok,
                _ => Token::Error,
            };

            // advance the col number, and update the end pos of the previous col.
            // Note that col_num will always point to the end of last token fetched by the lexer
            self.lex.extras.col_num += self.lex.span().end - self.prev_token_end_pos;
            self.prev_token_end_pos = self.lex.span().end;

            match token {
                Token::Skip => continue,
                _ => {
                    return Some(SpannedToken {
                        token_type: token,
                        lexeme: self.lex.slice(),
                        line_num: self.lex.extras.line_num,
                        // to calculate the start of the token, we subtract col_num,
                        // which points to the end of token, by the token length
                        col_start: self.lex.extras.col_num - self.lex.slice().len(),
                        col_end: self.lex.extras.col_num,
                    });
                }
            }
        }
    }

    pub fn get_line_num(&self) -> usize {
        self.lex.extras.line_num
    }

    pub fn get_col_num(&self) -> usize {
        self.lex.extras.col_num
    }
}
