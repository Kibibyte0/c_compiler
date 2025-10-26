use lexer::{SpannedToken, token::Token};
use parse_err::ParseErr;
use shared_context::{CompilerContext, Identifier, Span, SpannedIdentifier};
use std::error::Error;

use crate::ast::*;

mod parse_err;
mod parse_expressions;
mod parse_statement;

pub mod ast;
pub mod print_ast;

/// Top-level entry point for parsing a program.
///
/// Consumes the lexer and takes a mutable ref to compiler context, producing a parsed Program AST node
/// or an error if the input is invalid.
///
/// The CompilerContext provides access to source
/// maps and the string interner.
pub fn parse<'src, 'ctx>(
    lexer: lexer::Lexer<'src>,
    ctx: &'ctx mut CompilerContext<'src>,
) -> Result<Program, Box<dyn Error>> {
    let mut parser = Parser::new(lexer, ctx)?;
    let program = parser.parse_program()?;
    Ok(program)
}

/// The main parser structure.
///
/// The parser operates on tokens produced by the Lexer, maintaining a limited
/// lookahead buffer (up to three tokens) to enable predictive parsing decisions.
///
/// The parser builds an abstract syntax tree (AST) for the entire source program.
pub struct Parser<'src, 'ctx> {
    lexer: lexer::Lexer<'src>,
    ctx: &'ctx mut CompilerContext<'src>,

    /// The most recently consumed token.
    current_token: SpannedToken<'src>,

    /// Up to three lookahead tokens for predictive parsing.
    first_peeked_token: Option<SpannedToken<'src>>,
    second_peeked_token: Option<SpannedToken<'src>>,
    third_peeked_token: Option<SpannedToken<'src>>,
}

impl<'src, 'ctx> Parser<'src, 'ctx> {
    /// Creates a new parser instance from a lexer and compiler context.
    pub fn new(
        lexer: lexer::Lexer<'src>,
        ctx: &'ctx mut CompilerContext<'src>,
    ) -> Result<Self, ParseErr> {
        Ok(Self {
            lexer,
            ctx,
            current_token: SpannedToken::default(),
            first_peeked_token: None,
            second_peeked_token: None,
            third_peeked_token: None,
        })
    }

    /// Advances to the next token and returns it.
    ///
    /// If a peeked token exists, it is consumed first; otherwise, the lexer
    /// is queried for the next token. Returns an error on unexpected EOF.
    fn advance(&mut self) -> Result<SpannedToken<'src>, ParseErr> {
        match self.first_peeked_token.take() {
            Some(token) => {
                // Shift lookahead tokens left (peek2 -> peek1, etc.)
                self.current_token = token;
                self.first_peeked_token = self.second_peeked_token.take();
                self.second_peeked_token = self.third_peeked_token.take();
                Ok(token)
            }
            None => {
                // No lookahead available, fetch from lexer
                let token = self.lexer.next().ok_or_else(|| {
                    ParseErr::new(
                        "unexpected end of input".to_string(),
                        &self.current_token,
                        &self.ctx.source_map,
                    )
                })?;
                self.current_token = token;
                Ok(token)
            }
        }
    }

    /// Returns the next token without consuming it.
    fn peek(&mut self) -> Result<SpannedToken<'src>, ParseErr> {
        match self.first_peeked_token {
            Some(token) => Ok(token),
            None => {
                // Fetch a new token from the lexer and cache it
                self.first_peeked_token = self.lexer.next();
                Ok(self.first_peeked_token.ok_or_else(|| {
                    ParseErr::new(
                        "end of input".to_string(),
                        &self.current_token,
                        &self.ctx.source_map,
                    )
                })?)
            }
        }
    }

    /// Peeks two tokens ahead without consuming any.
    fn peek_two(&mut self) -> Result<SpannedToken<'src>, ParseErr> {
        // Ensure first peeked token exists
        if self.first_peeked_token.is_none() {
            self.peek()?;
        }

        match self.second_peeked_token {
            Some(token) => Ok(token),
            None => {
                self.second_peeked_token = self.lexer.next();
                Ok(self.second_peeked_token.ok_or_else(|| {
                    ParseErr::new(
                        "end of input (peek_two)".to_string(),
                        &self.current_token,
                        &self.ctx.source_map,
                    )
                })?)
            }
        }
    }

    /// Peeks three tokens ahead without consuming any.
    fn peek_three(&mut self) -> Result<SpannedToken<'src>, ParseErr> {
        // Ensure first and second peeked tokens exist
        if self.second_peeked_token.is_none() {
            self.peek_two()?;
        }

        match self.third_peeked_token {
            Some(token) => Ok(token),
            None => {
                self.third_peeked_token = self.lexer.next();
                Ok(self.third_peeked_token.ok_or_else(|| {
                    ParseErr::new(
                        "end of input (peek_three)".to_string(),
                        &self.current_token,
                        &self.ctx.source_map,
                    )
                })?)
            }
        }
    }

    /// Ensures the next token matches the expected kind.
    ///
    /// If it matches, the token is consumed. Otherwise, a descriptive
    /// ParseError is returned.
    fn expect_token(&mut self, expected: Token) -> Result<(), ParseErr> {
        let token = self.peek()?;
        if token.get_token() != expected {
            Err(ParseErr::expected(expected, &token, &self.ctx.source_map))
        } else {
            self.advance()?; // consume the expected token
            Ok(())
        }
    }

    /// Parses the root of the program.
    ///
    /// A program is a list of function declarations. This method loops
    /// until EOF, repeatedly parsing top-level functions.
    pub fn parse_program(&mut self) -> Result<Program, ParseErr> {
        let mut functions = Vec::new();
        while self.peek().is_ok() {
            functions.push(self.parse_function_decl()?);
        }
        Ok(Program::new(functions))
    }

    /// Parses a function declaration:
    ///
    /// ```text
    /// int <identifier> ( <param-list> ) <block-or-semicolon>
    /// ```
    fn parse_function_decl(&mut self) -> Result<FunctionDecl, ParseErr> {
        let line = self.peek()?.get_span().line;
        let start = self.peek()?.get_span().start;

        self.expect_token(Token::Int)?;
        let name = self.parse_identifier()?;

        self.expect_token(Token::LeftParenthesis)?;
        let params = self.parse_params_list()?;
        self.expect_token(Token::RightParenthesis)?;

        let body = self.parse_optional_block()?;

        let end = self.current_token.get_span().end;
        let span = Span::new(start, end, line);

        Ok(FunctionDecl::new(name, params, body, span))
    }

    /// Parses an optional function body block.
    ///
    /// Either a `{ ... }` block or a terminating semicolon (`;`) for
    /// declarations without a body.
    fn parse_optional_block(&mut self) -> Result<Option<Block>, ParseErr> {
        match self.peek()?.get_token() {
            Token::LeftCurlyBracket => Ok(Some(self.parse_block()?)),
            _ => {
                self.expect_token(Token::Semicolon)?;
                Ok(None)
            }
        }
    }

    /// Parses a function parameter list.
    ///
    /// Accepts either:
    /// - `void` (no parameters), or
    /// - one or more `int <identifier>` pairs separated by commas.
    fn parse_params_list(&mut self) -> Result<Vec<SpannedIdentifier>, ParseErr> {
        let mut params = Vec::new();
        if self.peek()?.get_token() == Token::Void {
            self.advance()?; // consume 'void'
            return Ok(params);
        }

        self.expect_token(Token::Int)?;
        params.push(self.parse_identifier()?);

        while self.peek()?.get_token() != Token::RightParenthesis {
            self.expect_token(Token::Comma)?;
            self.expect_token(Token::Int)?;
            params.push(self.parse_identifier()?);
        }
        Ok(params)
    }

    /// Parses a block of statements and/or declarations:
    ///
    /// ```text
    /// { <block-item>* }
    /// ```
    fn parse_block(&mut self) -> Result<Block, ParseErr> {
        let line = self.peek()?.get_span().line;
        let start = self.peek()?.get_span().start;

        self.expect_token(Token::LeftCurlyBracket)?;

        let mut block_items = Vec::new();
        while self.peek()?.get_token() != Token::RightCurlyBracket {
            let block_item = self.parse_block_item()?;
            block_items.push(block_item);
        }
        self.advance()?; // consume '}'

        let end = self.current_token.get_span().end;
        let span = Span::new(start, end, line);

        Ok(Block::new(block_items, span))
    }

    /// Parses a block item, which may be either a declaration or a statement.
    fn parse_block_item(&mut self) -> Result<BlockItem, ParseErr> {
        let next_token = self.peek()?;
        match next_token.get_token() {
            Token::Int => Ok(BlockItem::D(self.parse_declaration()?)),
            _ => Ok(BlockItem::S(self.parse_statement()?)),
        }
    }

    /// Parses a declaration, determining whether it is a function or variable declaration.
    fn parse_declaration(&mut self) -> Result<Declaration, ParseErr> {
        let token = self.peek_three()?.get_token();
        match token {
            Token::LeftParenthesis => Ok(Declaration::FunDecl(self.parse_function_decl()?)),
            _ => Ok(Declaration::VarDecl(self.parse_variable_declaration()?)),
        }
    }

    /// Parses a variable declaration:
    ///
    /// ```text
    /// int <identifier> [= <expr>] ;
    /// ```
    fn parse_variable_declaration(&mut self) -> Result<VariableDecl, ParseErr> {
        let line = self.peek()?.get_span().line;
        let start = self.peek()?.get_span().start;

        self.expect_token(Token::Int)?;
        let name = self.parse_identifier()?;
        let init = match self.peek()?.get_token() {
            Token::Assignment => {
                self.advance()?; // consume '='
                Some(self.parse_expression(0)?)
            }
            _ => None,
        };

        self.expect_token(Token::Semicolon)?;

        let end = self.current_token.get_span().end;
        let span = Span::new(start, end, line);
        Ok(VariableDecl::new(name, init, span))
    }

    /// Parses an identifier and returns it as a SpannedToken.
    ///
    /// Converts the lexeme into an interned identifier and attaches
    /// span information for error reporting.
    fn parse_identifier(&mut self) -> Result<SpannedIdentifier, ParseErr> {
        let line = self.peek()?.get_span().line;
        let start = self.peek()?.get_span().start;
        let token = self.advance()?;
        let end = self.current_token.get_span().end;
        let span = Span::new(start, end, line);

        if token.get_token() == Token::Identifier {
            let identifier = Identifier::new(self.ctx.interner.intern(token.get_lexeme()), 0);
            Ok(SpannedIdentifier::new(identifier, span))
        } else {
            Err(ParseErr::expected(
                "identifier",
                &token,
                &self.ctx.source_map,
            ))
        }
    }
}
