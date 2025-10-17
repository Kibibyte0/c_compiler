use crate::ast::BinaryOP;
use crate::ast::Expression;
use crate::ast::ExpressionType;
use crate::ast::UnaryOP;
use crate::parser::ParseErr;
use crate::parser::Parser;
use lexer::token::Token;
use shared_context::Span;

impl<'a, 'c> Parser<'a, 'c> {
    pub(crate) fn parse_expression(&mut self, min_prec: usize) -> Result<Expression, ParseErr> {
        let mut span_start = self.peek()?.get_span().start;
        let mut left = self.parse_factor()?;

        let mut next_token = self.peek()?.get_token();
        while next_token.is_binary() && next_token.precedence() >= min_prec {
            if next_token == Token::Assignment {
                left = self.parse_assignment(left, next_token.precedence(), span_start)?;
                span_start = self.peek()?.get_span().start;
            } else if next_token == Token::QuestionMark {
                left = self.parse_conditional(left, next_token.precedence(), span_start)?;
                span_start = self.peek()?.get_span().start;
            } else {
                left = self.parse_binary(left, next_token.precedence(), span_start)?;
                span_start = self.peek()?.get_span().start;
            }
            next_token = self.peek()?.get_token();
        }

        Ok(left)
    }

    fn parse_factor(&mut self) -> Result<Expression, ParseErr> {
        let start = self.peek()?.get_span().start;
        let next_token = self.peek()?;

        if next_token.get_token() == Token::ConstantInt {
            self.parse_constant_int()
        } else if next_token.get_token().is_unary() {
            let op = self.parse_unary_op()?;
            let inner_exp = self.parse_factor()?;
            let end = self.current_token.get_span().end;

            let expr_type = ExpressionType::Unary {
                operator: op,
                operand: Box::new(inner_exp),
            };
            let span = Span::new(start, end);
            Ok(Expression::new(expr_type, span))
        } else if next_token.get_token() == Token::LeftParenthesis {
            self.advance()?; // consume the '('token
            let inner_exp = self.parse_expression(0);
            self.expect_token(")")?;
            inner_exp
        } else if next_token.get_token() == Token::Identifier {
            let id = self.parse_identifier()?;
            let end = self.current_token.get_span().end;
            let span = Span::new(start, end);
            let expr_type = ExpressionType::Var(id);
            Ok(Expression::new(expr_type, span))
        } else {
            Err(ParseErr::new(
                String::from("invald expression"),
                &next_token,
                &self.ctx.source_map,
            ))
        }
    }

    fn parse_assignment(
        &mut self,
        left: Expression,
        token_precedence: usize,
        start: usize,
    ) -> Result<Expression, ParseErr> {
        self.advance()?; // consume the '=' token
        let right = self.parse_expression(token_precedence)?;
        let expr_type = ExpressionType::Assignment {
            lvalue: Box::new(left),
            rvalue: Box::new(right),
        };
        let end = self.current_token.get_span().end;
        let span = Span::new(start, end);
        Ok(Expression::new(expr_type, span))
    }

    fn parse_conditional(
        &mut self,
        left: Expression,
        token_precedence: usize,
        start: usize,
    ) -> Result<Expression, ParseErr> {
        let middle = self.parse_conditional_middle()?;
        let right = self.parse_expression(token_precedence)?;
        let expr_type = ExpressionType::Conditional {
            cond: Box::new(left),
            cons: Box::new(middle),
            alt: Box::new(right),
        };
        let end = self.current_token.get_span().end;
        let span = Span::new(start, end);
        Ok(Expression::new(expr_type, span))
    }

    // parse the middle expression of the ternary operator
    fn parse_conditional_middle(&mut self) -> Result<Expression, ParseErr> {
        self.advance()?; // consume the '?' token
        let exp = self.parse_expression(0)?;
        self.expect_token(":")?;
        Ok(exp)
    }

    fn parse_binary(
        &mut self,
        left: Expression,
        token_precedence: usize,
        start: usize,
    ) -> Result<Expression, ParseErr> {
        let op = self.parse_binary_op()?;
        let right = self.parse_expression(token_precedence + 1)?;
        let expr_type = ExpressionType::Binary {
            operator: op,
            operand1: Box::new(left),
            operand2: Box::new(right),
        };
        let end = self.current_token.get_span().end;
        let span = Span::new(start, end);
        Ok(Expression::new(expr_type, span))
    }

    // parse a binary operator
    fn parse_binary_op(&mut self) -> Result<BinaryOP, ParseErr> {
        let token = self.advance()?;

        match token.get_token() {
            Token::Add => Ok(BinaryOP::Add),
            Token::Neg => Ok(BinaryOP::Sub), // Negative token for both Subtraction and Negation
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
            _ => Err(ParseErr::expected(
                "binary operator",
                &token,
                &self.ctx.source_map,
            )),
        }
    }

    // parse a unary operator
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

    // parse integer literals
    fn parse_constant_int(&mut self) -> Result<Expression, ParseErr> {
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
            let span = Span::new(start, end);
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
