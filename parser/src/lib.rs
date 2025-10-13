use lexer::{SpannedToken, token::Token};
use parse_err::ParseErr;

use crate::ast::{BlockItem, Declaration, FunctionDef, Identifier, Program, Spanned, Statement};

mod parse_err;
mod parse_expressions;
pub mod print_ast;

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
    /// return error If they don't match
    fn expect_token(&mut self, expected: &'static str) -> Result<(), ParseErr> {
        let token = self.peek()?;
        if token.get_lexeme() != expected {
            Err(ParseErr::expected(expected, &self.current_token))
        } else {
            self.advance()?; // consume token
            Ok(())
        }
    }

    /// warp parser node in spanned struct
    fn spanned<T>(
        &mut self,
        parse_fn: impl FnOnce(&mut Self) -> Result<T, ParseErr>,
    ) -> Result<Spanned<T>, ParseErr> {
        let start = self.peek()?.get_span().start;
        let result = parse_fn(self)?;
        let end = self.current_token.get_span().end;
        Ok(Spanned::new(result, start..end))
    }

    pub fn parse(&mut self) -> Result<Spanned<Program>, ParseErr> {
        self.spanned(|this| this.parse_program())
    }

    // entry point for the parser
    fn parse_program(&mut self) -> Result<ast::Program, ParseErr> {
        let function = self.spanned(|this| this.parse_function())?;
        let program = ast::Program::new(function);

        if let Ok(tok) = self.advance() {
            Err(ParseErr::expected("end of input", tok))
        } else {
            Ok(program)
        }
    }

    fn parse_function(&mut self) -> Result<ast::FunctionDef, ParseErr> {
        self.expect_token("int")?;

        let name = self.spanned(|this| this.parse_identifier())?;

        self.expect_token("(")?;
        self.expect_token("void")?;
        self.expect_token(")")?;

        self.expect_token("{")?;

        let mut function_body = Vec::new();
        while self.peek()?.get_token() != Token::RightCurlyBracket {
            let block_item = self.spanned(|this| this.parse_block_item())?;
            function_body.push(block_item);
        }
        self.advance()?; // consume the '}' token
        Ok(FunctionDef::new(name, function_body))
    }

    fn parse_block_item(&mut self) -> Result<BlockItem, ParseErr> {
        let next_token = self.peek()?;
        match next_token.get_token() {
            Token::Int => Ok(BlockItem::D(self.spanned(|this| this.parse_declaration())?)),
            _ => Ok(BlockItem::S(self.spanned(|this| this.parse_statement())?)),
        }
    }

    fn parse_statement(&mut self) -> Result<Statement, ParseErr> {
        let next_token = self.peek()?.get_token();
        match next_token {
            Token::Return => self.parse_return_statement(),
            Token::Semicolon => {
                self.advance()?; // consume the ';' token
                Ok(Statement::Null)
            }
            Token::If => self.parse_if_statement(),
            _ => {
                let exp = self.parse_expression(0)?;
                self.expect_token(";")?;
                Ok(Statement::ExprStatement(exp))
            }
        }
    }

    fn parse_declaration(&mut self) -> Result<Declaration, ParseErr> {
        self.expect_token("int")?;
        let name = self.spanned(|this| this.parse_identifier())?;

        let init = if self.peek()?.get_token() == Token::Assignment {
            self.advance()?; // consume the '=' token
            Some(self.parse_expression(0)?)
        } else {
            None
        };

        self.expect_token(";")?;

        Ok(Declaration::new(name, init))
    }

    fn parse_return_statement(&mut self) -> Result<Statement, ParseErr> {
        self.advance()?; // consume the 'return' token
        let exp = self.parse_expression(0)?;
        self.expect_token(";")?;
        Ok(Statement::Return(exp))
    }

    fn parse_if_statement(&mut self) -> Result<Statement, ParseErr> {
        self.advance()?; // consume the 'if' token

        self.expect_token("(")?;
        let condition = self.parse_expression(0)?;
        self.expect_token(")")?;

        let if_clause = Box::new(self.spanned(|this| this.parse_statement())?);

        let else_clause = match self.peek()?.get_token() {
            Token::Else => {
                self.advance()?; // consume the 'else' token
                Some(Box::new(self.spanned(|this| this.parse_statement())?))
            }
            _ => None,
        };

        Ok(Statement::IfStatement {
            condition,
            if_clause,
            else_clause,
        })
    }

    fn parse_identifier(&mut self) -> Result<Identifier, ParseErr> {
        let token = self.advance()?;

        if token.get_token() == Token::Identifier {
            Ok(Identifier::new(token.get_lexeme().to_string()))
        } else {
            Err(ParseErr::expected("identifier", &token))
        }
    }
}
