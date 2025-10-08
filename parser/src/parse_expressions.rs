use crate::ast::BinaryOP;
use crate::ast::UnaryOP;

use super::ParseErr;
use super::Parser;
use super::ast::Expression;
use lexer::token::Token;

// impl block for C expressions
impl<'source> Parser<'source> {
    pub(crate) fn parse_expression(&mut self, min_prec: usize) -> Result<Expression, ParseErr> {
        let mut left = self.parse_factor()?;

        let mut next_token = self.peek()?.get_token();
        while next_token.is_binary() && next_token.precedence() >= min_prec {
            let op = self.parse_binary_op()?;
            let right = self.parse_expression(next_token.precedence() + 1)?;
            left = Expression::Binary {
                operator: op,
                operand1: Box::new(left),
                operand2: Box::new(right),
            };
            next_token = self.peek()?.get_token();
        }
        return Ok(left);
    }

    fn parse_factor(&mut self) -> Result<Expression, ParseErr> {
        let next_token = self.peek()?;

        if next_token.get_token() == Token::ConstantInt {
            self.parse_constant_int()
        } else if next_token.get_token().is_unary() {
            let op = self.parse_unary_op()?;
            let inner_exp = self.parse_factor()?;
            Ok(Expression::Unary {
                operator: op,
                operand: Box::new(inner_exp),
            })
        } else if next_token.get_token() == Token::LeftParenthesis {
            self.advance()?;
            let inner_exp = self.parse_expression(0);
            self.expect_token_type(Token::RightParenthesis, ")")?;
            inner_exp
        } else {
            Err(ParseErr::new(String::from("invald expression"), next_token))
        }
    }

    // parse a binary operator
    fn parse_binary_op(&mut self) -> Result<BinaryOP, ParseErr> {
        let token = self.advance()?;

        match token.get_token() {
            Token::Add => Ok(BinaryOP::Add),
            Token::Neg => Ok(BinaryOP::Sub), // Negative for Subtraction
            Token::Mul => Ok(BinaryOP::Mul),
            Token::Div => Ok(BinaryOP::Div),
            Token::Mod => Ok(BinaryOP::Mod),

            // Logical operators
            Token::LogicalAnd => Ok(BinaryOP::LogicalAnd),
            Token::LogicalOr => Ok(BinaryOP::LogicalOr),

            // Comparison operators
            Token::Equal => Ok(BinaryOP::Equal),
            Token::NotEqual => Ok(BinaryOP::NotEqual),
            Token::LessThan => Ok(BinaryOP::LessThan),
            Token::GreaterThan => Ok(BinaryOP::GreaterThan),
            Token::LessThanOrEq => Ok(BinaryOP::LessThanOrEq),
            Token::GreaterThanOrEq => Ok(BinaryOP::GreaterThanOrEq),

            // If the token is not a recognized binary operator, return an error
            _ => Err(ParseErr::expected_found("binary operator", &token)),
        }
    }

    // parse a unary operator
    fn parse_unary_op(&mut self) -> Result<UnaryOP, ParseErr> {
        let token = self.advance()?;

        match token.get_token() {
            Token::Neg => Ok(UnaryOP::Neg),
            Token::Not => Ok(UnaryOP::Not),
            Token::LogicalNot => Ok(UnaryOP::LogicalNot),
            _ => Err(ParseErr::expected_found("unary operator", &token)),
        }
    }

    // parse integer literals
    fn parse_constant_int(&mut self) -> Result<Expression, ParseErr> {
        let token = self.advance()?;

        if token.get_token() == Token::ConstantInt {
            let value = token.get_lexeme().parse::<i32>().map_err(|_| {
                ParseErr::new("failed to parse integer constant".to_string(), token)
            })?;
            Ok(Expression::Constant(value))
        } else {
            Err(ParseErr::expected_found("integer constant", &token))
        }
    }
}
