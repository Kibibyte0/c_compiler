use lexer::{Lexer, SpannedToken, Token};
use ast::{Program, FunctionDefinition, Statement, Expression, Identifier};
use std::mem;

mod parse_err;

pub mod ast;

pub struct Parser<'source> {
    lexer: Lexer<'source>,
    current_token: SpannedToken<'source>,
    previous_token: SpannedToken<'source>,
}

impl<'source> Parser<'source> {
    pub fn new(lexer: Lexer<'source>) -> Self {
        Self {
            lexer: lexer,
            current_token: SpannedToken::default(),
            previous_token: SpannedToken::default(),
        }
    }

    // advance the parser one token, return none if the there is no tokens left
    fn advance(&mut self) -> Option<&SpannedToken<'source>> {
        self.previous_token = mem::take(&mut self.current_token);
        self.current_token = self.lexer.next()?;
        Some(&self.current_token)
    }

    // call advance and report an 'end of input stream error' if there is no tokens left
    fn get_next_token(&mut self) -> &SpannedToken<'source> {
        if let Some(_) = self.advance() {
            &self.current_token
        } else {
            let message = String::from("unexpected end of input stream");
            let err = parse_err::ParseErr::new(
                message,
                self.previous_token.span.line_num,
                self.previous_token.span.col_end,
            );
            err.report(self.lexer.get_source_code());
            unreachable!();
        }
    }

    fn assert_token(&mut self, expected: &str) {
        let token = self.get_next_token();

        if token.lexeme != expected {
            let message = format!("expected '{}'", expected);
            let err = parse_err::ParseErr::new(
                message,
                self.previous_token.span.line_num,
                self.previous_token.span.col_end,
            );

            err.report(self.lexer.get_source_code());
        }
    }

    // entry point for the parser 
    pub fn parse_program(&mut self) -> Program<'source> {
        let program = Program {
            function: self.parse_function(),
        };

        match self.advance() {
            Some(_) => {
                let message = format!("unexpexted token at the end of the input stream");
                let err = parse_err::ParseErr::new(message,
                    self.current_token.span.line_num,
                    self.current_token.span.col_start
                );
                err.report(self.lexer.get_source_code());
                unreachable!();
            }
            None => program,
        }
    }

    fn parse_function(&mut self) -> FunctionDefinition<'source> {
        self.assert_token("int");

        let name = self.parse_identifier(); 

        self.assert_token("(");
        self.assert_token("void");
        self.assert_token(")");

        self.assert_token("{");

        let body = self.parse_statement();

        self.assert_token("}");

        FunctionDefinition {
            name,
            body,
        }
    }

    fn parse_identifier(&mut self) -> Identifier<'source>  {
        let token = self.get_next_token();

        if token.token_type == Token::Identifier {
            Identifier(token.lexeme)
        } else {
            let message = String::from("expected an identifier");
            let err = parse_err::ParseErr::new(
                message, 
                self.previous_token.span.line_num,
                self.previous_token.span.col_end,
            );
            err.report(self.lexer.get_source_code());
            unreachable!();
        }
    }

    fn parse_statement(&mut self) -> Statement {
        self.assert_token("return");
        let exp = self.parse_expression();
        self.assert_token(";");
        Statement::Return(exp)
    }

    fn parse_expression(&mut self) -> Expression {
        let token = self.get_next_token();

        if token.token_type == Token::ConstantInt {
            // it passed the token type test, so it will always be a numerical digit
            Expression::Constant(token.lexeme.parse().unwrap())
        } else {
            let message = String::from("expected an integer constant");
            let err = parse_err::ParseErr::new(
                message,
                self.previous_token.span.line_num,
                self.previous_token.span.col_end,
            );
            err.report(self.lexer.get_source_code());
            unreachable!();
        }
    }
}

