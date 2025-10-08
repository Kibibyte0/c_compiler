use lexer::{SpannedToken, token::Token};
use parse_err::ParseErr;

use crate::ast::Identifier;

mod parse_err;
mod parse_expressions;
mod print_ast;

pub mod ast;

pub struct Parser<'a> {
    lexer: lexer::Lexer<'a>,
    current_token: SpannedToken<'a>,
    peeked_token: Option<SpannedToken<'a>>,
}

impl<'a> Parser<'a> {
    /// create a new instance of parser
    pub fn new(lexer: lexer::Lexer<'a>) -> Result<Self, ParseErr> {
        Ok(Self {
            lexer,
            current_token: SpannedToken::default(),
            peeked_token: None,
        })
    }

    /// advance the parser to the next token and return the current token,
    fn advance(&mut self) -> Result<&SpannedToken<'a>, ParseErr> {
        match self.peeked_token.take() {
            Some(token) => {
                self.peeked_token = None;
                self.current_token = token;
                Ok(&self.current_token)
            }
            None => {
                let token = self.lexer.next().ok_or_else(|| {
                    ParseErr::new("unexpected end of input".to_string(), &self.current_token)
                })?;
                self.current_token = token;
                Ok(&self.current_token)
            }
        }
    }

    /// return the peeked token
    fn peek(&mut self) -> Result<&SpannedToken<'a>, ParseErr> {
        // if there is something in peeked token, return it and leave it unchanged
        if self.peeked_token.is_some() {
            Ok(self.peeked_token.as_ref().unwrap())
        } else {
            // if it's empty, get the next token and return reference to it
            self.peeked_token = self.lexer.next();
            Ok(self
                .peeked_token
                .as_ref()
                .ok_or_else(|| ParseErr::new("end of input".to_string(), &self.current_token))?)
        }
    }

    /// compare the next token with the expected token type,
    /// return error If they don't match, the lexeme parameter is used for error logging
    fn expect_token_type(&mut self, expected: Token, lexeme: &'static str) -> Result<(), ParseErr> {
        let token = self.advance()?;
        if token.get_token() != expected {
            Err(ParseErr::expected_found(lexeme, token))
        } else {
            Ok(())
        }
    }

    // entry point for the parser
    pub fn parse_program(&mut self) -> Result<ast::Program, ParseErr> {
        let program = ast::Program::new(self.parse_function()?);

        if let Ok(tok) = self.advance() {
            Err(ParseErr::expected_found("end of input", tok))
        } else {
            Ok(program)
        }
    }

    fn parse_function(&mut self) -> Result<ast::FunctionDef, ParseErr> {
        self.expect_token_type(Token::Int, "int")?;

        let name = self.parse_identifier()?;

        self.expect_token_type(Token::LeftParenthesis, "(")?;
        self.expect_token_type(Token::Void, "void")?;
        self.expect_token_type(Token::RightParenthesis, ")")?;

        self.expect_token_type(Token::LeftCurlyBracket, "{")?;

        let body = self.parse_statement()?;

        self.expect_token_type(Token::RightCurlyBracket, "}")?;

        Ok(ast::FunctionDef::new(name, body))
    }

    fn parse_identifier(&mut self) -> Result<Identifier, ParseErr> {
        let token = self.advance()?;

        if token.get_token() == Token::Identifier {
            Ok(ast::Identifier(token.get_lexeme().to_string()))
        } else {
            Err(ParseErr::expected_found("identifier", &token))
        }
    }

    fn parse_statement(&mut self) -> Result<ast::Statement, ParseErr> {
        self.expect_token_type(Token::Return, "return")?;
        let exp = self.parse_expression(0)?;
        self.expect_token_type(Token::Semicolon, ";")?;
        Ok(ast::Statement::Return(exp))
    }
}
