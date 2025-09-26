use lexer::{Lexer, SpannedToken, Token};
use ast::{Program, FunctionDefinition, Statement, Expression, Identifier};

mod parse_err;
use parse_err::ParseErr;

pub mod ast;

pub struct Parser<'source> {
    lexer: Lexer<'source>,
    peeked_token: Option<SpannedToken<'source>>,
}

impl<'source> Parser<'source> {
    // create a new parser instance, return an 'input is empty' error if there is no tokens
    pub fn build(mut lexer: Lexer<'source>) -> Result<Self, ParseErr> {
        let current_token = lexer.next().ok_or_else(|| {
            ParseErr::new(
                String::from("input is empty"),
                0,
                0,
            )
        })?;

        Ok(Self {
            lexer,
            peeked_token: Some(current_token),
        })
    }

    // advance the parser to the next token and return the current peeked token,
    fn advance(&mut self) -> Result<SpannedToken<'source>, ParseErr> {
        let token = self.peeked_token.take().ok_or_else(|| {
            ParseErr::new(
                String::from("end of input stream"),
                self.lexer.get_line_num(),
                self.lexer.get_span().end,
            )
        })?;

        if token.token_type == Token::Error {
            return Err(ParseErr {
                message: format!("invalid token: {}", token.lexeme),
                line: token.line_num,
                column: token.col_start,
            });
        }

        self.peeked_token = self.lexer.next();
        Ok(token)
    }

    // compare the current token lexmme to an expected string and advance
    // report error if the do not match
    fn _expect_lexeme(&mut self, expected: &str) -> Result<(), ParseErr> {
        let token = self.advance()?;

        if token.lexeme != expected {
            Err(ParseErr::new(
                format!("expected {}, found '{}'", expected, token.lexeme),
                token.line_num,
                token.col_start,
            ))
        } else {
            Ok(())
        }
    }

    // same as expect_lexeme() but compare token type instead
    // used when possible for more performance 
    fn expect_token_type(&mut self, expected: Token) -> Result<(), ParseErr> {
        let token = self.advance()?;

        if token.token_type != expected {
            Err(ParseErr::new(
                format!("expected {:?}, found '{}'", expected, token.lexeme),
                token.line_num,
                token.col_start,
            ))
        } else {
            Ok(())
        }
    }

    // entry point for the parser 
    pub fn parse_program(&mut self) -> Result<Program<'source>, ParseErr> {
        let program = Program {
            function: self.parse_function()?,
        };

        if let Ok(tok) = self.advance() {
            Err(ParseErr::new(
                String::from("expected end of input stream"),
                tok.line_num,
                tok.col_start,
            ))
        } else {
            Ok(program)
        }
    }

    fn parse_function(&mut self) -> Result<FunctionDefinition<'source>, ParseErr> {
        self.expect_token_type(Token::Int)?;

        let name = self.parse_identifier()?; 

        self.expect_token_type(Token::LeftParenthesis)?;
        self.expect_token_type(Token::Void)?;
        self.expect_token_type(Token::RightParenthesis)?;

        self.expect_token_type(Token::LeftCurlyBracket)?;
        
        let body = self.parse_statement()?;
        
        self.expect_token_type(Token::RightCurlyBracket)?;

        Ok(FunctionDefinition { name, body })
    }

    fn parse_identifier(&mut self) -> Result<Identifier<'source>, ParseErr>  {
        let token = self.advance()?;

        if token.token_type == Token::Identifier {
            Ok(Identifier(token.lexeme))
        } else {
            Err(ParseErr::new(
                String::from("expected an identifier"), 
                token.line_num,
                token.col_start,
            ))
        }
    }

    fn parse_statement(&mut self) -> Result<Statement, ParseErr> {
        self.expect_token_type(Token::Return)?;
        let exp = self.parse_expression()?;
        self.expect_token_type(Token::Semicolon)?;
        Ok(Statement::Return(exp))
    }

    fn parse_expression(&mut self) -> Result<Expression, ParseErr> {
        let token = self.advance()?;

        if token.token_type == Token::ConstantInt {
            let value = token.lexeme.parse::<i32>().map_err(|_| {
                ParseErr::new(
                    "failed to parse integer constant".to_string(),
                    token.line_num,
                    token.col_start,
                )
            })?;
            Ok(Expression::Constant(value))
        } else {
            Err(ParseErr::new(
                String::from("expected an integer constant"),
                token.line_num,
                token.col_start,
            ))
        }
    }
}

