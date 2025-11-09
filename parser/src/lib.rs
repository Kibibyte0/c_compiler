use lexer::{SpannedToken, token::Token};
use parse_err::ParseErr;
use shared_context::{
    Span, source_map::SourceMap, symbol_interner::SymbolInterner, type_interner::TypeInterner,
};
use std::error::Error;

use crate::ast::*;

mod parse_declarations;
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
    ty_interner: &'ctx mut TypeInterner<'src>,
    interner: &'ctx mut SymbolInterner<'src>,
    source_map: &'ctx SourceMap<'src>,
) -> Result<Program, Box<dyn Error>> {
    let mut parser = Parser::new(lexer, ty_interner, interner, source_map)?;
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
    ty_interner: &'ctx mut TypeInterner<'src>,
    sy_interner: &'ctx mut SymbolInterner<'src>,
    source_map: &'ctx SourceMap<'src>,

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
        ty_interner: &'ctx mut TypeInterner<'src>,
        sy_interner: &'ctx mut SymbolInterner<'src>,
        source_map: &'ctx SourceMap<'src>,
    ) -> Result<Self, ParseErr> {
        Ok(Self {
            lexer,
            ty_interner,
            sy_interner,
            source_map,
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
                        "unexpected end of input",
                        self.current_token.get_span(),
                        &self.source_map,
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
                        "end of input",
                        self.current_token.get_span(),
                        &self.source_map,
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
                        "end of input (peek_two)",
                        self.current_token.get_span(),
                        &self.source_map,
                    )
                })?)
            }
        }
    }

    /// Peeks three tokens ahead without consuming any.
    ///
    /// currently peek three is no longer in use due to a change in the parser
    /// but it's left here in case it's needed in the future
    /// if it is of no use after finishing the parser, it will be discarded
    fn _peek_three(&mut self) -> Result<SpannedToken<'src>, ParseErr> {
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
                        "end of input (peek_three)",
                        self.current_token.get_span(),
                        &self.source_map,
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
            Err(ParseErr::expected(expected, &token, &self.source_map))
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
        let mut declarations = Vec::new();
        while self.peek().is_ok() {
            declarations.push(self.parse_declaration()?);
        }
        Ok(Program::new(declarations))
    }

    /// Parses a block of statements and/or declarations:
    fn parse_block(&mut self) -> Result<Block, ParseErr> {
        let (start, line) = self.peek()?.get_span().get_start_and_line();

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
        if self.peek()?.get_token().is_specifier() {
            Ok(BlockItem::D(self.parse_declaration()?))
        } else {
            Ok(BlockItem::S(self.parse_statement()?))
        }
    }
}
