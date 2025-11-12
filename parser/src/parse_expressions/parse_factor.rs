use crate::ast::{Expression, InnerExpression};
use crate::{ParseErr, Parser};
use lexer::token::Token;
use shared_context::{Const, Span, Type};

impl<'a, 'b> Parser<'a, 'b> {
    /// Parses a "factor" in an expression.
    /// A factor can be:
    /// - an integer constant
    /// - a unary expression
    /// - a parenthesized expression
    /// - a type cast
    /// - an identifier (variable or function call)
    pub(crate) fn parse_factor(&mut self) -> Result<Expression, ParseErr> {
        let token = self.peek()?;

        match token.get_token() {
            tok if tok.is_int_constant() => self.handle_constant_int(),
            tok if tok.is_unary() => self.handle_unary_expression(),
            Token::LeftParenthesis if self.peek_two()?.get_token().is_specifier() => {
                self.handle_type_cast()
            }
            Token::LeftParenthesis => self.handle_parenthesized_expression(),
            Token::Identifier => self.handle_identifier_expression(),
            _ => Err(ParseErr::new(
                "invalid expression",
                token.get_span(),
                &self.source_map,
            )),
        }
    }

    /// Handles parsing of integer constants
    fn handle_constant_int(&mut self) -> Result<Expression, ParseErr> {
        match self.peek()?.get_token() {
            Token::ConstantInt => self.parse_constant_int(),
            Token::ConstantLong => self.parse_constant_long(),
            Token::ConstantUint => self.parse_constant_uint(),
            Token::ConstantUlong => self.parse_constant_ulong(),
            _ => self.parse_constant_int(),
        }
    }

    /// Handles parsing of unary expressions, e.g., `-x` or `!flag`
    fn handle_unary_expression(&mut self) -> Result<Expression, ParseErr> {
        let (start, line) = self.peek()?.get_span().get_start_and_line();

        let op = self.parse_unary_op()?;
        let inner_exp = self.parse_factor()?;
        let end = self.current_token.get_span().end;

        let expr_type = InnerExpression::Unary {
            operator: op,
            operand: Box::new(inner_exp),
        };
        let span = Span::new(start, end, line);
        Ok(Expression::new(expr_type, Type::default(), span))
    }

    /// Handles parenthesized expressions: `(expr)`
    fn handle_parenthesized_expression(&mut self) -> Result<Expression, ParseErr> {
        self.advance()?; // consume '('
        let inner_exp = self.parse_expression(0)?;
        self.expect_token(Token::RightParenthesis)?;
        Ok(inner_exp)
    }

    /// handle type casting: (<type>) <factor>
    fn handle_type_cast(&mut self) -> Result<Expression, ParseErr> {
        let (start, line) = self.peek()?.get_span().get_start_and_line();

        self.advance()?; // consume '('
        let target_type = self.parse_type_list()?;
        self.expect_token(Token::RightParenthesis)?;
        let expr_type = InnerExpression::Cast {
            target_type,
            expr: Box::new(self.parse_factor()?),
        };

        let end = self.current_token.get_span().end;
        let span = Span::new(start, end, line);
        Ok(Expression::new(expr_type, Type::default(), span))
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
        let (start, line) = self.peek()?.get_span().get_start_and_line();

        let name = self.parse_identifier()?;
        self.expect_token(Token::LeftParenthesis)?;
        let args = self.parse_function_args()?;
        self.expect_token(Token::RightParenthesis)?;
        let end = self.current_token.get_span().end;
        let span = Span::new(start, end, line);

        let expr_type = InnerExpression::FunctionCall { name, args };
        Ok(Expression::new(expr_type, Type::default(), span))
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
        let (start, line) = self.peek()?.get_span().get_start_and_line();

        let id = self.parse_identifier()?;
        let end = self.current_token.get_span().end;
        let span = Span::new(start, end, line);

        let expr_type = InnerExpression::Var(id);
        Ok(Expression::new(expr_type, Type::default(), span))
    }

    /// Parses an integer constant
    ///
    /// Turn it into a long integer constant if the value does not fit
    fn parse_constant_int(&mut self) -> Result<Expression, ParseErr> {
        let (start, line) = self.peek()?.get_span().get_start_and_line();
        let token = self.advance()?;

        // parse the number literal into 128 bit signed integer
        let value = token.get_lexeme().parse::<i128>().map_err(|_| {
            ParseErr::new(
                "failed to parse integer constant",
                token.get_span(),
                &self.source_map,
            )
        })?;

        let end = self.current_token.get_span().end;
        let span = Span::new(start, end, line);

        // check if the number literal can fit to any of the supported types
        if let Ok(int) = i32::try_from(value) {
            let constant = Const::ConstInt(int);
            let expr_type = InnerExpression::Constant(constant);
            Ok(Expression::new(expr_type, Type::Int, span))
        } else if let Ok(long) = i64::try_from(value) {
            let contant = Const::ConstLong(long);
            let expr_type = InnerExpression::Constant(contant);
            Ok(Expression::new(expr_type, Type::Long, span))
        } else {
            return Err(ParseErr::new(
                "integer value too large to represent",
                token.get_span(),
                &self.source_map,
            ));
        }
    }

    /// parse a long integer constant (e.g., `123l` or `123L`)
    fn parse_constant_long(&mut self) -> Result<Expression, ParseErr> {
        let (start, line) = self.peek()?.get_span().get_start_and_line();
        let token = self.advance()?;

        // remove the suffix
        let lexeme = &token.get_lexeme()[..token.get_lexeme().len() - 1];

        let value = lexeme.parse::<i128>().map_err(|_| {
            ParseErr::new(
                "failed to parse integer constant",
                token.get_span(),
                &self.source_map,
            )
        })?;

        let end = self.current_token.get_span().end;
        let span = Span::new(start, end, line);

        if let Ok(long) = i64::try_from(value) {
            let contant = Const::ConstLong(long);
            let expr_type = InnerExpression::Constant(contant);
            Ok(Expression::new(expr_type, Type::Long, span))
        } else {
            return Err(ParseErr::new(
                "integer value too large to represent",
                token.get_span(),
                &self.source_map,
            ));
        }
    }

    /// Parses an unsigned integer constant
    ///
    /// Turn it into an unsignd long integer constant if the value does not fit
    fn parse_constant_uint(&mut self) -> Result<Expression, ParseErr> {
        let (start, line) = self.peek()?.get_span().get_start_and_line();
        let token = self.advance()?;

        // remove the suffix
        let lexeme = &token.get_lexeme()[..token.get_lexeme().len() - 1];

        // parse the number literal into 128 bit unsigned integer
        let value = lexeme.parse::<u128>().map_err(|_| {
            ParseErr::new(
                "failed to parse integer constant",
                token.get_span(),
                &self.source_map,
            )
        })?;

        let end = self.current_token.get_span().end;
        let span = Span::new(start, end, line);

        // check if the number literal can fit to any of the supported types
        if let Ok(uint) = u32::try_from(value) {
            let constant = Const::ConstUint(uint);
            let expr_type = InnerExpression::Constant(constant);
            Ok(Expression::new(expr_type, Type::Uint, span))
        } else if let Ok(ulong) = u64::try_from(value) {
            let contant = Const::ConstUlong(ulong);
            let expr_type = InnerExpression::Constant(contant);
            Ok(Expression::new(expr_type, Type::Ulong, span))
        } else {
            return Err(ParseErr::new(
                "integer value too large to represent",
                token.get_span(),
                &self.source_map,
            ));
        }
    }

    /// parse an unsigned long integer constant (e.g., `123lU` or `123uL`)
    fn parse_constant_ulong(&mut self) -> Result<Expression, ParseErr> {
        let (start, line) = self.peek()?.get_span().get_start_and_line();
        let token = self.advance()?;

        // remove the suffix
        let lexeme = &token.get_lexeme()[..token.get_lexeme().len() - 2];

        let value = lexeme.parse::<u128>().map_err(|_| {
            ParseErr::new(
                "failed to parse integer constant",
                token.get_span(),
                &self.source_map,
            )
        })?;

        let end = self.current_token.get_span().end;
        let span = Span::new(start, end, line);

        if let Ok(ulong) = u64::try_from(value) {
            let contant = Const::ConstUlong(ulong);
            let expr_type = InnerExpression::Constant(contant);
            Ok(Expression::new(expr_type, Type::Ulong, span))
        } else {
            return Err(ParseErr::new(
                "integer value too large to represent",
                token.get_span(),
                &self.source_map,
            ));
        }
    }
}
