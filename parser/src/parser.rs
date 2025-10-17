use lexer::{SpannedToken, token::Token};
use parse_err::ParseErr;
use shared_context::{CompilerContext, Identifier, Span};

use crate::ast::*;

mod parse_err;
mod parse_expressions;

pub struct Parser<'a, 'c> {
    lexer: lexer::Lexer<'a>,
    ctx: &'c mut CompilerContext<'a>,
    current_token: SpannedToken<'a>,
    peeked_token: Option<SpannedToken<'a>>,
}

impl<'a, 'c> Parser<'a, 'c> {
    /// create a new instance of parser
    pub fn new(
        lexer: lexer::Lexer<'a>,
        ctx: &'c mut CompilerContext<'a>,
    ) -> Result<Self, ParseErr> {
        Ok(Self {
            lexer,
            ctx,
            current_token: SpannedToken::default(),
            peeked_token: None,
        })
    }

    /// advance the parser to the next token and return the current token,
    fn advance(&mut self) -> Result<SpannedToken<'a>, ParseErr> {
        match self.peeked_token.take() {
            Some(token) => {
                self.peeked_token = None;
                self.current_token = token;
                Ok(self.current_token.clone())
            }
            None => {
                let token = self.lexer.next().ok_or_else(|| {
                    ParseErr::new(
                        "unexpected end of input".to_string(),
                        &self.current_token,
                        &self.ctx.source_map,
                    )
                })?;
                self.current_token = token;
                Ok(self.current_token.clone())
            }
        }
    }

    /// return the peeked token
    fn peek(&mut self) -> Result<SpannedToken<'a>, ParseErr> {
        // if there is something in peeked token, return it and leave it unchanged
        if self.peeked_token.is_some() {
            Ok(self.peeked_token.clone().unwrap())
        } else {
            // if it's empty, get the next token and return reference to it
            self.peeked_token = self.lexer.next();
            Ok(self.peeked_token.clone().ok_or_else(|| {
                ParseErr::new(
                    "end of input".to_string(),
                    &self.current_token,
                    &self.ctx.source_map,
                )
            })?)
        }
    }

    /// compare the next token with the expected token type,
    /// return error If they don't match
    fn expect_token(&mut self, expected: &'static str) -> Result<(), ParseErr> {
        let token = self.peek()?;
        if token.get_lexeme() != expected {
            Err(ParseErr::expected(
                expected,
                &self.current_token,
                &self.ctx.source_map,
            ))
        } else {
            self.advance()?; // consume token
            Ok(())
        }
    }

    // entry point for the parser
    pub fn parse_program(&mut self) -> Result<Program, ParseErr> {
        let function = self.parse_function()?;
        let program = Program::new(function);

        if let Ok(tok) = self.advance() {
            Err(ParseErr::expected(
                "end of input",
                &tok,
                &self.ctx.source_map,
            ))
        } else {
            Ok(program)
        }
    }

    fn parse_function(&mut self) -> Result<FunctionDef, ParseErr> {
        let start = self.peek()?.get_span().start;
        self.expect_token("int")?;

        let name = self.parse_identifier()?;

        self.expect_token("(")?;
        self.expect_token("void")?;
        self.expect_token(")")?;

        let body = self.parse_block()?;

        let end = self.current_token.get_span().end;
        let span = Span::new(start, end);

        Ok(FunctionDef::new(name, body, span))
    }

    fn parse_block(&mut self) -> Result<Block, ParseErr> {
        let start = self.peek()?.get_span().start;

        self.expect_token("{")?;

        let mut block_items = Vec::new();
        while self.peek()?.get_token() != Token::RightCurlyBracket {
            let block_item = self.parse_block_item()?;
            block_items.push(block_item);
        }
        self.advance()?; // consume the '}' token

        let end = self.current_token.get_span().end;
        let span = Span::new(start, end);

        Ok(Block::new(block_items, span))
    }

    fn parse_block_item(&mut self) -> Result<BlockItem, ParseErr> {
        let next_token = self.peek()?;
        match next_token.get_token() {
            Token::Int => Ok(BlockItem::D(self.parse_declaration()?)),
            _ => Ok(BlockItem::S(self.parse_statement()?)),
        }
    }

    fn parse_statement(&mut self) -> Result<Statement, ParseErr> {
        let start = self.peek()?.get_span().start;

        let next_token = self.peek()?.get_token();
        let stmt_type = match next_token {
            Token::Return => self.parse_return_statement()?,
            Token::Semicolon => {
                self.advance()?; // consume the ';' token
                StatementType::Null
            }
            Token::LeftCurlyBracket => self.parse_compound_statement()?,
            Token::If => self.parse_if_statement()?,
            _ => {
                let exp = self.parse_expression(0)?;
                self.expect_token(";")?;
                StatementType::ExprStatement(exp)
            }
        };

        let end = self.current_token.get_span().end;
        let span = Span::new(start, end);

        Ok(Statement::new(stmt_type, span))
    }

    fn parse_declaration(&mut self) -> Result<Declaration, ParseErr> {
        let start = self.peek()?.get_span().start;

        self.expect_token("int")?;
        let name = self.parse_identifier()?;

        let init = if self.peek()?.get_token() == Token::Assignment {
            self.advance()?; // consume the '=' token
            Some(self.parse_expression(0)?)
        } else {
            None
        };

        self.expect_token(";")?;

        let end = self.current_token.get_span().end;
        let span = Span::new(start, end);

        Ok(Declaration::new(name, init, span))
    }

    fn parse_return_statement(&mut self) -> Result<StatementType, ParseErr> {
        self.advance()?; // consume the 'return' token
        let exp = self.parse_expression(0)?;
        self.expect_token(";")?;
        Ok(StatementType::Return(exp))
    }

    fn parse_compound_statement(&mut self) -> Result<StatementType, ParseErr> {
        Ok(StatementType::Compound(self.parse_block()?))
    }

    fn parse_if_statement(&mut self) -> Result<StatementType, ParseErr> {
        self.advance()?; // consume the 'if' token

        self.expect_token("(")?;
        let condition = self.parse_expression(0)?;
        self.expect_token(")")?;

        let if_clause = Box::new(self.parse_statement()?);

        let else_clause = match self.peek()?.get_token() {
            Token::Else => {
                self.advance()?; // consume the 'else' token
                Some(Box::new(self.parse_statement()?))
            }
            _ => None,
        };

        Ok(StatementType::IfStatement {
            condition,
            if_clause,
            else_clause,
        })
    }

    fn parse_identifier(&mut self) -> Result<Identifier, ParseErr> {
        let start = self.peek()?.get_span().start;

        let token = self.advance()?;

        let end = self.current_token.get_span().end;
        let span = Span::new(start, end);

        if token.get_token() == Token::Identifier {
            Ok(Identifier::new(
                self.ctx.interner.intern(token.get_lexeme()),
                0,
                span,
            ))
        } else {
            Err(ParseErr::expected(
                "identifier",
                &token,
                &self.ctx.source_map,
            ))
        }
    }
}
