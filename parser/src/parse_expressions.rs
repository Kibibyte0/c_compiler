use crate::ParseErr;
use crate::Parser;
use crate::ast::{BinaryOP, Expression, ExpressionType, UnaryOP};
use lexer::token::Token;
use shared_context::Span;

mod parse_factor;

impl<'src, 'ctx> Parser<'src, 'ctx> {
    /// Parses an expression with a minimum precedence, supporting
    /// binary, unary, assignment, and conditional (`?:`) operators.
    pub(crate) fn parse_expression(&mut self, min_prec: usize) -> Result<Expression, ParseErr> {
        // Capture the start span for this expression
        let line = self.peek()?.get_span().line;
        let mut span_start = self.peek()?.get_span().start;

        // Parse the left-hand side: a factor (literal, variable, or parenthesis)
        let mut left = self.parse_factor()?;

        // Continue parsing while the next token is a binary operator of sufficient precedence
        while self.peek()?.get_token().is_binary() {
            let next_token = self.peek()?.get_token();
            if next_token.precedence() < min_prec {
                break;
            }

            left = match next_token {
                Token::Assignment => {
                    self.handle_assignment(left, next_token.precedence(), span_start, line)?
                }
                Token::QuestionMark => {
                    self.handle_conditional(left, next_token.precedence(), span_start, line)?
                }
                _ => self.handle_binary(left, next_token.precedence(), span_start, line)?,
            };

            // Update the start position for the next iteration
            span_start = self.peek()?.get_span().start;
        }

        Ok(left)
    }

    /// Handles assignment expressions (`lhs = rhs`)
    fn handle_assignment(
        &mut self,
        left: Expression,
        token_precedence: usize,
        start: usize,
        line: usize,
    ) -> Result<Expression, ParseErr> {
        self.advance()?; // consume '='
        let right = self.parse_expression(token_precedence)?;
        let expr_type = ExpressionType::Assignment {
            lvalue: Box::new(left),
            rvalue: Box::new(right),
        };
        let end = self.current_token.get_span().end;
        Ok(Expression::new(expr_type, Span::new(start, end, line)))
    }

    /// Handles ternary conditional expressions (`cond ? cons : alt`)
    fn handle_conditional(
        &mut self,
        left: Expression,
        token_precedence: usize,
        start: usize,
        line: usize,
    ) -> Result<Expression, ParseErr> {
        let cons_expr = self.parse_conditional_middle()?;
        let alt_expr = self.parse_expression(token_precedence)?;
        let expr_type = ExpressionType::Conditional {
            cond: Box::new(left),
            cons: Box::new(cons_expr),
            alt: Box::new(alt_expr),
        };
        let end = self.current_token.get_span().end;
        Ok(Expression::new(expr_type, Span::new(start, end, line)))
    }

    /// Parses the middle of a ternary expression (between `?` and `:`)
    fn parse_conditional_middle(&mut self) -> Result<Expression, ParseErr> {
        self.advance()?; // consume '?'
        let expr = self.parse_expression(0)?;
        self.expect_token(Token::Colon)?;
        Ok(expr)
    }

    /// Handles binary operators (`a + b`, `x == y`, etc.)
    fn handle_binary(
        &mut self,
        left: Expression,
        token_precedence: usize,
        start: usize,
        line: usize,
    ) -> Result<Expression, ParseErr> {
        let op = self.parse_binary_op()?;
        let right = self.parse_expression(token_precedence + 1)?;
        let expr_type = ExpressionType::Binary {
            operator: op,
            operand1: Box::new(left),
            operand2: Box::new(right),
        };
        let end = self.current_token.get_span().end;
        Ok(Expression::new(expr_type, Span::new(start, end, line)))
    }

    /// Converts the current token into a [`BinaryOP`] or returns an error
    fn parse_binary_op(&mut self) -> Result<BinaryOP, ParseErr> {
        let token = self.advance()?;

        match token.get_token() {
            Token::Add => Ok(BinaryOP::Add),
            Token::Neg => Ok(BinaryOP::Sub), // Negation and Subtraction share the same token
            Token::Mul => Ok(BinaryOP::Mul),
            Token::Div => Ok(BinaryOP::Div),
            Token::Mod => Ok(BinaryOP::Mod),

            Token::LogicalAnd => Ok(BinaryOP::LogicalAnd),
            Token::LogicalOr => Ok(BinaryOP::LogicalOr),

            Token::Equal => Ok(BinaryOP::Equal),
            Token::NotEqual => Ok(BinaryOP::NotEqual),
            Token::LessThan => Ok(BinaryOP::LessThan),
            Token::GreaterThan => Ok(BinaryOP::GreaterThan),
            Token::LessThanOrEq => Ok(BinaryOP::LessThanOrEq),
            Token::GreaterThanOrEq => Ok(BinaryOP::GreaterThanOrEq),

            _ => Err(ParseErr::expected(
                "binary operator",
                &token,
                &self.ctx.source_map,
            )),
        }
    }

    /// Converts the current token into a [`UnaryOP`] or returns an error
    fn parse_unary_op(&mut self) -> Result<UnaryOP, ParseErr> {
        let token = self.advance()?;

        match token.get_token() {
            Token::Neg => Ok(UnaryOP::Neg),
            Token::Not => Ok(UnaryOP::Not),
            Token::LogicalNot => Ok(UnaryOP::LogicalNot),
            _ => Err(ParseErr::expected(
                "unary operator",
                &token,
                &self.ctx.source_map,
            )),
        }
    }
}
