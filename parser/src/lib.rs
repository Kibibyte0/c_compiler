use lexer::{Lexer, SpannedToken, token::Token};
use parse_err::ParseErr;

use crate::ast::Identifier;

mod parse_err;
mod parse_expressions;

pub mod ast;

pub struct Parser<'source> {
    lexer: Lexer<'source>,
    peeked_token: Option<SpannedToken<'source>>,
}

impl<'source> Parser<'source> {
    // create a new parser instance, return an 'input is empty' error if there is no tokens
    pub fn build(mut lexer: Lexer<'source>) -> Result<Self, ParseErr> {
        let peeked_token = lexer
            .next()
            .ok_or_else(|| ParseErr::new(String::from("input is empty"), 0, 0))?;

        Ok(Self {
            lexer,
            peeked_token: Some(peeked_token),
        })
    }

    // report end on input errors
    fn unexpected_eof(&self) -> ParseErr {
        ParseErr::new(
            "unexpected end of input".to_string(),
            self.lexer.get_line_num(),
            self.lexer.get_col_num(),
        )
    }

    // advance the parser to the next token and return the current peeked token,
    fn advance(&mut self) -> Result<SpannedToken<'source>, ParseErr> {
        let token = self
            .peeked_token
            .take()
            .ok_or_else(|| self.unexpected_eof())?;

        if token.token_type == Token::Error {
            return Err(ParseErr::new(
                format!("invalid token: {}", token.lexeme),
                token.line_num,
                token.col_start,
            ));
        }

        self.peeked_token = self.lexer.next();
        Ok(token)
    }

    // return the peeked token, return end of input stream error if there is no token
    fn peek(&self) -> Result<&SpannedToken<'source>, ParseErr> {
        self.peeked_token
            .as_ref()
            .ok_or_else(|| self.unexpected_eof())
    }

    // compare the current token lexmme to an expected string and advance
    // report error if the do not match
    fn _expect_lexeme(&mut self, expected: &str) -> Result<(), ParseErr> {
        let token = self.advance()?;

        if token.lexeme != expected {
            Err(ParseErr::expected_found(expected, &token))
        } else {
            Ok(())
        }
    }

    // same as expect_lexeme() but compare token type instead
    // used when possible for more performance
    fn expect_token_type(&mut self, expected: Token) -> Result<(), ParseErr> {
        match self.advance() {
            Ok(token) => {
                if token.token_type != expected {
                    Err(ParseErr::expected_found(format!("{:?}", expected), &token))
                } else {
                    Ok(())
                }
            }
            Err(_) => {
                // When advance fails, return the expected-token error
                // as the expexted token might be the last token, and eof error would be incorrect
                Err(ParseErr::expected(
                    format!("{:?}", expected),
                    self.lexer.get_line_num(),
                    self.lexer.get_col_num(),
                ))
            }
        }
    }

    // entry point for the parser
    pub fn parse_program(&mut self) -> Result<ast::Program, ParseErr> {
        let program = ast::Program::new(self.parse_function()?);

        if let Ok(tok) = self.peek() {
            Err(ParseErr::expected_found("end of input", tok))
        } else {
            Ok(program)
        }
    }

    fn parse_function(&mut self) -> Result<ast::FunctionDef, ParseErr> {
        self.expect_token_type(Token::Int)?;

        let name = self.parse_identifier()?;

        self.expect_token_type(Token::LeftParenthesis)?;
        self.expect_token_type(Token::Void)?;
        self.expect_token_type(Token::RightParenthesis)?;

        self.expect_token_type(Token::LeftCurlyBracket)?;

        let body = self.parse_statement()?;

        self.expect_token_type(Token::RightCurlyBracket)?;

        Ok(ast::FunctionDef::new(name, body))
    }

    fn parse_identifier(&mut self) -> Result<Identifier, ParseErr> {
        let token = self.advance()?;

        if token.token_type == Token::Identifier {
            Ok(ast::Identifier(token.lexeme.to_string()))
        } else {
            Err(ParseErr::expected_found("identifier", &token))
        }
    }

    fn parse_statement(&mut self) -> Result<ast::Statement, ParseErr> {
        self.expect_token_type(Token::Return)?;
        let exp = self.parse_expression(0)?;
        self.expect_token_type(Token::Semicolon)?;
        Ok(ast::Statement::Return(exp))
    }
}
