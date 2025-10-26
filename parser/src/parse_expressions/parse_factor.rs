use crate::ast::{Expression, ExpressionType};
use crate::{ParseErr, Parser};
use lexer::token::Token;
use shared_context::Span;

impl<'a, 'b> Parser<'a, 'b> {
    /// Parses a "factor" in an expression.
    /// A factor can be:
    /// - an integer constant
    /// - a unary expression
    /// - a parenthesized expression
    /// - an identifier (variable or function call)
    pub(crate) fn parse_factor(&mut self) -> Result<Expression, ParseErr> {
        let token = self.peek()?;

        match token.get_token() {
            Token::ConstantInt => self.handle_constant_int(),
            tok if tok.is_unary() => self.handle_unary_expression(),
            Token::LeftParenthesis => self.handle_parenthesized_expression(),
            Token::Identifier => self.handle_identifier_expression(),
            _ => Err(ParseErr::new(
                String::from("invalid expression"),
                &token,
                &self.ctx.source_map,
            )),
        }
    }

    /// Handles parsing of integer constants
    fn handle_constant_int(&mut self) -> Result<Expression, ParseErr> {
        self.parse_constant_int()
    }

    /// Handles parsing of unary expressions, e.g., `-x` or `!flag`
    fn handle_unary_expression(&mut self) -> Result<Expression, ParseErr> {
        let line = self.peek()?.get_span().line;
        let start = self.peek()?.get_span().start;

        let op = self.parse_unary_op()?;
        let inner_exp = self.parse_factor()?;
        let end = self.current_token.get_span().end;

        let expr_type = ExpressionType::Unary {
            operator: op,
            operand: Box::new(inner_exp),
        };
        let span = Span::new(start, end, line);
        Ok(Expression::new(expr_type, span))
    }

    /// Handles parenthesized expressions: `(expr)`
    fn handle_parenthesized_expression(&mut self) -> Result<Expression, ParseErr> {
        self.advance()?; // consume '('
        let inner_exp = self.parse_expression(0)?;
        self.expect_token(Token::RightParenthesis)?;
        Ok(inner_exp)
    }

    /// Handles identifiers, which can be variables or function calls
    fn handle_identifier_expression(&mut self) -> Result<Expression, ParseErr> {
        let token = self.peek_two()?.get_token(); // look ahead to see if it's a function call
        match token {
            Token::LeftParenthesis => self.parse_function_call(), // function call
            _ => self.parse_variable_expression(),                // variable
        }
    }

    /// Parses a function call: `foo(arg1, arg2, ...)`
    fn parse_function_call(&mut self) -> Result<Expression, ParseErr> {
        let line = self.peek()?.get_span().line;
        let start = self.peek()?.get_span().start;

        let name = self.parse_identifier()?;
        self.expect_token(Token::LeftParenthesis)?;
        let args = self.parse_function_args()?;
        self.expect_token(Token::RightParenthesis)?;
        let end = self.current_token.get_span().end;
        let span = Span::new(start, end, line);

        let expr_type = ExpressionType::FunctionCall { name, args };
        Ok(Expression::new(expr_type, span))
    }

    /// Parses the argument list of a function call
    fn parse_function_args(&mut self) -> Result<Vec<Box<Expression>>, ParseErr> {
        let mut args = Vec::new();
        match self.peek()?.get_token() {
            Token::RightParenthesis => Ok(args), // no arguments
            _ => {
                args.push(Box::new(self.parse_expression(0)?));
                while self.peek()?.get_token() != Token::RightParenthesis {
                    self.expect_token(Token::Comma)?;
                    args.push(Box::new(self.parse_expression(0)?));
                }
                Ok(args)
            }
        }
    }

    /// Parses a variable expression
    fn parse_variable_expression(&mut self) -> Result<Expression, ParseErr> {
        let line = self.peek()?.get_span().line;
        let start = self.peek()?.get_span().start;

        let id = self.parse_identifier()?;
        let end = self.current_token.get_span().end;
        let span = Span::new(start, end, line);

        let expr_type = ExpressionType::Var(id);
        Ok(Expression::new(expr_type, span))
    }

    /// Parses an integer constant
    fn parse_constant_int(&mut self) -> Result<Expression, ParseErr> {
        let line = self.peek()?.get_span().line;
        let start = self.peek()?.get_span().start;
        let token = self.advance()?;

        if token.get_token() == Token::ConstantInt {
            let value = token.get_lexeme().parse::<i32>().map_err(|_| {
                ParseErr::new(
                    "failed to parse integer constant".to_string(),
                    &token,
                    &self.ctx.source_map,
                )
            })?;
            let expr_type = ExpressionType::Constant(value);
            let end = self.current_token.get_span().end;
            let span = Span::new(start, end, line);
            Ok(Expression::new(expr_type, span))
        } else {
            Err(ParseErr::expected(
                "integer constant",
                &token,
                &self.ctx.source_map,
            ))
        }
    }
}
