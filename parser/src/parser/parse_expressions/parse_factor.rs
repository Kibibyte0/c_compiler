use crate::ast::{Expression, ExpressionType, Span};
use crate::parser::{ParseErr, Parser};
use lexer::token::Token;

impl<'a, 'b> Parser<'a, 'b> {
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

    fn handle_constant_int(&mut self) -> Result<Expression, ParseErr> {
        self.parse_constant_int()
    }

    fn handle_unary_expression(&mut self) -> Result<Expression, ParseErr> {
        let line = self.peek()?.get_line();
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

    fn handle_parenthesized_expression(&mut self) -> Result<Expression, ParseErr> {
        self.advance()?; // consume '('
        let inner_exp = self.parse_expression(0)?;
        self.expect_token(Token::RightParenthesis)?;
        Ok(inner_exp)
    }

    fn handle_identifier_expression(&mut self) -> Result<Expression, ParseErr> {
        let line = self.peek()?.get_line();
        let start = self.peek()?.get_span().start;

        let id = self.parse_identifier()?;
        let end = self.current_token.get_span().end;
        let span = Span::new(start, end, line);

        let expr_type = ExpressionType::Var(id);
        Ok(Expression::new(expr_type, span))
    }

    fn parse_constant_int(&mut self) -> Result<Expression, ParseErr> {
        let line = self.peek()?.get_line();
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
