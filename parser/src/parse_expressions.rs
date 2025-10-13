use super::ParseErr;
use super::Parser;
use super::ast::Expression;
use crate::ast::BinaryOP;
use crate::ast::Spanned;
use crate::ast::UnaryOP;
use lexer::token::Token;

impl<'source> Parser<'source> {
    pub(crate) fn parse_expression(
        &mut self,
        min_prec: usize,
    ) -> Result<Spanned<Expression>, ParseErr> {
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

    fn parse_factor(&mut self) -> Result<Spanned<Expression>, ParseErr> {
        let span_start = self.peek()?.get_span().start;
        let next_token = self.peek()?;

        if next_token.get_token() == Token::ConstantInt {
            self.spanned(|this| this.parse_constant_int())
        } else if next_token.get_token().is_unary() {
            let op = self.parse_unary_op()?;
            let inner_exp = self.parse_factor()?;
            let span_end = self.current_token.get_span().end;
            Ok(Spanned::new(
                Expression::Unary {
                    operator: op,
                    operand: Box::new(inner_exp),
                },
                span_start..span_end,
            ))
        } else if next_token.get_token() == Token::LeftParenthesis {
            self.advance()?; // consume the '('token
            let inner_exp = self.parse_expression(0);
            self.expect_token(")")?;
            inner_exp
        } else if next_token.get_token() == Token::Identifier {
            let id = self.spanned(|this| this.parse_identifier())?;
            let span_end = self.current_token.get_span().end;
            Ok(Spanned::new(Expression::Var(id), span_start..span_end))
        } else {
            Err(ParseErr::new(String::from("invald expression"), next_token))
        }
    }

    fn parse_assignment(
        &mut self,
        left: Spanned<Expression>,
        token_precedence: usize,
        span_start: usize,
    ) -> Result<Spanned<Expression>, ParseErr> {
        self.advance()?; // consume the '=' token
        let right = self.parse_expression(token_precedence)?;
        let span_end = self.current_token.get_span().end;
        let expr = Spanned::new(
            Expression::Assignment {
                lvalue: Box::new(left),
                rvalue: Box::new(right),
            },
            span_start..span_end,
        );
        Ok(expr)
    }

    fn parse_conditional(
        &mut self,
        left: Spanned<Expression>,
        token_precedence: usize,
        span_start: usize,
    ) -> Result<Spanned<Expression>, ParseErr> {
        let middle = self.parse_conditional_middle()?;
        let right = self.parse_expression(token_precedence)?;
        let span_end = self.current_token.get_span().end;
        let expr = Spanned::new(
            Expression::Conditional {
                cond: Box::new(left),
                cons: Box::new(middle),
                alt: Box::new(right),
            },
            span_start..span_end,
        );
        Ok(expr)
    }

    // parse the middle expression of the ternary operator
    fn parse_conditional_middle(&mut self) -> Result<Spanned<Expression>, ParseErr> {
        self.advance()?; // consume the '?' token
        let exp = self.parse_expression(0)?;
        self.expect_token(":")?;
        Ok(exp)
    }

    fn parse_binary(
        &mut self,
        left: Spanned<Expression>,
        token_precedence: usize,
        span_start: usize,
    ) -> Result<Spanned<Expression>, ParseErr> {
        let op = self.parse_binary_op()?;
        let right = self.parse_expression(token_precedence + 1)?;
        let span_end = self.current_token.get_span().end;
        let expr = Spanned::new(
            Expression::Binary {
                operator: op,
                operand1: Box::new(left),
                operand2: Box::new(right),
            },
            span_start..span_end,
        );
        Ok(expr)
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
            _ => Err(ParseErr::expected("binary operator", &token)),
        }
    }

    // parse a unary operator
    fn parse_unary_op(&mut self) -> Result<UnaryOP, ParseErr> {
        let token = self.advance()?;

        match token.get_token() {
            Token::Neg => Ok(UnaryOP::Neg),
            Token::Not => Ok(UnaryOP::Not),
            Token::LogicalNot => Ok(UnaryOP::LogicalNot),
            _ => Err(ParseErr::expected("unary operator", &token)),
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
            Err(ParseErr::expected("integer constant", &token))
        }
    }
}
