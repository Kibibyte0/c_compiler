use crate::ast::UnaryOP;

use super::ParseErr;
use super::Parser;
use super::ast::Expression;
use lexer::token::Token;

// impl block for C expressions
impl<'source> Parser<'source> {
    pub(crate) fn parse_expression(&mut self) -> Result<Expression, ParseErr> {
        let next_token = self.peek()?;

        if next_token.token_type == Token::ConstantInt {
            self.parse_constant_int()
        } else if next_token.token_type.is_unary() {
            let op = self.parse_unary_op()?;
            let inner_exp = self.parse_expression()?;
            Ok(Expression::Unary(op, Box::new(inner_exp)))
        } else if next_token.token_type == Token::LeftParenthesis {
            self.advance()?;
            let inner_exp = self.parse_expression();
            self.expect_token_type(Token::RightParenthesis)?;
            inner_exp
        } else {
            Err(ParseErr::new(
                String::from("invald expression"),
                next_token.line_num,
                next_token.col_start,
            ))
        }
    }

    // parse a unary operator
    fn parse_unary_op(&mut self) -> Result<UnaryOP, ParseErr> {
        let token = self.advance()?;

        match token.lexeme {
            "-" => Ok(UnaryOP::Negation),
            "~" => Ok(UnaryOP::BitwiseComplement),
            _ => Err(ParseErr::expected("unary operator", &token)),
        }
    }

    // parse integer literals
    fn parse_constant_int(&mut self) -> Result<Expression, ParseErr> {
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
            Err(ParseErr::expected("integer constant", &token))
        }
    }
}
