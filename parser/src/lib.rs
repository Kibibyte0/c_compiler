use lexer::{Lexer, SpannedToken, Token};
use ast::{Program, FunctionDefinition, Statement, Expression, Identifier};

mod parse_err;

mod ast;

// get next token, exit with error if the there is no tokens left
fn get_next_token<'source>(lexer: &'source mut Lexer) -> SpannedToken<'source> {
    if let Some(tok) = lexer.next() {
        tok
    } else {
        let message = String::from("unexpected end of input stream");
        let err = parse_err::ParseErr::new(
            message, lexer.get_line_num(), lexer.get_col_num()
        );
        err.report(lexer.get_source_code());
        unreachable!();
    }
}

// check if the expected token match the current token
// exit with error if they don't match
fn assert_token(expected: &str, lexer: &mut Lexer) {
    let line_num = lexer.get_line_num();
    let col_num = lexer.get_col_num();

    let token = get_next_token(lexer);

    if expected != token.lexeme {
        let message = format!("expected '{}'", expected);
        let err = parse_err::ParseErr::new(
            message, line_num, col_num
        );

        err.report(lexer.get_source_code());
    }
}

// entry point for the parser 
pub fn parse_program(mut lexer: Lexer) -> Program {
    Program {
        function: parse_function(&mut lexer),
    }
}

// parsing functions
fn parse_function(lexer: &mut Lexer) -> FunctionDefinition {
    assert_token("int", lexer);

    let name = parse_identifier(lexer); 

    assert_token("(", lexer);
    assert_token("void", lexer);
    assert_token(")", lexer);

    assert_token("{", lexer);

    let body = parse_statement(lexer);

    assert_token("}", lexer);

    FunctionDefinition {
        name,
        body,
    }
}

fn parse_identifier(lexer: &mut Lexer) -> Identifier  {
    let token = get_next_token(lexer);
    if token.token_type == Token::Identifier {
        Identifier(token.lexeme.to_string())
    } else {
        let message = String::from("expected an identifier");
        let err = parse_err::ParseErr::new(
            message, lexer.get_line_num(), lexer.get_col_num()
        );
        err.report(lexer.get_source_code());
        unreachable!();
    }
}

fn parse_statement(lexer: &mut Lexer) -> Statement {
    assert_token("return", lexer);
    let exp = parse_expression(lexer);
    assert_token(";", lexer);
    Statement::Return(exp)
}

fn parse_expression(lexer: &mut Lexer) -> Expression {
    let token = get_next_token(lexer);
    if token.token_type == Token::ConstantInt {
        Expression::Constant(token.lexeme.parse().unwrap())
    } else {
        let message = String::from("expected an integer constant");
        let err = parse_err::ParseErr::new(
            message, lexer.get_line_num(), lexer.get_col_num()
        );
        err.report(lexer.get_source_code());
        unreachable!();
    }
}
