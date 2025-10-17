use std::error::Error;

use crate::ast::Program;
use parser::Parser;
use shared_context::CompilerContext;

pub mod ast;
mod parser;
pub mod print_ast;

pub fn parse<'a, 'c>(
    lexer: lexer::Lexer<'a>,
    ctx: &'c mut CompilerContext<'a>,
) -> Result<Program, Box<dyn Error>> {
    let mut parser = Parser::new(lexer, ctx)?;
    let program = parser.parse_program()?;
    Ok(program)
}
