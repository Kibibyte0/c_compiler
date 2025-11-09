use crate::ParseErr;
use crate::Parser;
use crate::ast::{Expression, ForInit, Statement, StatementType};
use lexer::token::Token;
use shared_context::{Identifier, Span};

impl<'src, 'ctx> Parser<'src, 'ctx> {
    /// Parses a statement and returns a `Statement` AST node.
    /// Handles all types of statements: return, if, loops, break/continue, compound blocks, or expressions.
    pub(crate) fn parse_statement(&mut self) -> Result<Statement, ParseErr> {
        // Get the current token's line and start position for the statement's span.
        let (start, line) = self.peek()?.get_span().get_start_and_line();

        // Peek at the next token to decide which type of statement to parse
        let next_token = self.peek()?.get_token();
        let stmt_type = match next_token {
            Token::Return => self.parse_return_statement()?,
            Token::LeftCurlyBracket => self.parse_compound_statement()?,
            Token::If => self.parse_if_statement()?,
            Token::Do => self.parse_do_while_statement()?,
            Token::While => self.parse_while_statement()?,
            Token::For => self.parse_for_statement()?,
            Token::Continue => self.parse_continue_statement()?,
            Token::Break => self.parse_break_statement()?,
            Token::Semicolon => {
                self.advance()?; // consume the ';' token for empty statement
                StatementType::Null
            }
            _ => {
                // Expression statement: parse an expression followed by a semicolon
                let exp = self.parse_expression(0)?;
                self.expect_token(Token::Semicolon)?;
                StatementType::ExprStatement(exp)
            }
        };

        // Compute the span of the statement from start to current token
        let end = self.current_token.get_span().end;
        let span = Span::new(start, end, line);

        Ok(Statement::new(stmt_type, span))
    }

    /// Parses a `return` statement
    fn parse_return_statement(&mut self) -> Result<StatementType, ParseErr> {
        self.advance()?; // consume the 'return' token
        let exp = self.parse_expression(0)?;
        self.expect_token(Token::Semicolon)?;
        Ok(StatementType::Return(exp))
    }

    /// Parses a compound block `{ ... }`
    fn parse_compound_statement(&mut self) -> Result<StatementType, ParseErr> {
        Ok(StatementType::Compound(self.parse_block()?))
    }

    /// Parses an `if` statement with optional `else`
    fn parse_if_statement(&mut self) -> Result<StatementType, ParseErr> {
        self.advance()?; // consume the 'if' token

        // Parse the condition inside parentheses
        self.expect_token(Token::LeftParenthesis)?;
        let condition = self.parse_expression(0)?;
        self.expect_token(Token::RightParenthesis)?;

        // Parse the statement to execute if condition is true
        let if_clause = Box::new(self.parse_statement()?);

        // Optional else clause
        let else_clause = match self.peek()?.get_token() {
            Token::Else => {
                self.advance()?; // consume 'else'
                Some(Box::new(self.parse_statement()?))
            }
            _ => None,
        };

        Ok(StatementType::IfStatement {
            condition,
            if_clause,
            else_clause,
        })
    }

    /// Parses a `continue` statement
    fn parse_continue_statement(&mut self) -> Result<StatementType, ParseErr> {
        self.advance()?; // consume 'continue'
        let stmt_type = StatementType::Continue(Identifier::default());
        self.expect_token(Token::Semicolon)?;
        Ok(stmt_type)
    }

    /// Parses a `break` statement
    fn parse_break_statement(&mut self) -> Result<StatementType, ParseErr> {
        self.advance()?; // consume 'ctxreak'
        let stmt_type = StatementType::Break(Identifier::default());
        self.expect_token(Token::Semicolon)?;
        Ok(stmt_type)
    }

    /// Parses a `while` loop: `while (cond) stmt`
    fn parse_while_statement(&mut self) -> Result<StatementType, ParseErr> {
        self.advance()?; // consume 'while'
        self.expect_token(Token::LeftParenthesis)?;
        let condition = self.parse_expression(0)?;
        self.expect_token(Token::RightParenthesis)?;
        let body = Box::new(self.parse_statement()?);
        Ok(StatementType::While {
            condition,
            body,
            label: Identifier::default(),
        })
    }

    /// Parses a `do-while` loop: `do stmt while (cond);`
    fn parse_do_while_statement(&mut self) -> Result<StatementType, ParseErr> {
        self.advance()?; // consume 'do'
        let body = Box::new(self.parse_statement()?);
        self.expect_token(Token::While)?;
        self.expect_token(Token::LeftParenthesis)?;
        let condition = self.parse_expression(0)?;
        self.expect_token(Token::RightParenthesis)?;
        self.expect_token(Token::Semicolon)?;
        Ok(StatementType::DoWhile {
            condition,
            body,
            label: Identifier::default(),
        })
    }

    /// Parses a `for` loop: `for (init; condition; post) stmt`
    fn parse_for_statement(&mut self) -> Result<StatementType, ParseErr> {
        self.advance()?; // consume 'for'
        self.expect_token(Token::LeftParenthesis)?;

        let init = self.parse_for_init()?;
        let condition = self.parse_optional_for_statement_expr()?;
        self.expect_token(Token::Semicolon)?;
        let post = self.parse_optional_for_statement_expr()?;
        self.expect_token(Token::RightParenthesis)?;

        let body = Box::new(self.parse_statement()?);

        Ok(StatementType::For {
            init,
            condition,
            post,
            body,
            label: Identifier::default(),
        })
    }

    /// Parses the initialization part of a `for` loop
    fn parse_for_init(&mut self) -> Result<ForInit, ParseErr> {
        let next_token = self.peek()?.get_token();
        if next_token.is_specifier() {
            let (start, line) = self.peek()?.get_span().get_start_and_line();
            let (var_type, storage_class) = self.parse_type_and_storage_class_list()?;
            Ok(ForInit::D(self.parse_variable_declaration(
                var_type,
                storage_class,
                start,
                line,
            )?))
        } else {
            let for_init = ForInit::E(self.parse_optional_for_statement_expr()?);
            self.expect_token(Token::Semicolon)?;
            Ok(for_init)
        }
    }

    /// Parses an optional expression in `for` loops (condition or post-expression)
    pub fn parse_optional_for_statement_expr(&mut self) -> Result<Option<Expression>, ParseErr> {
        let next_token = self.peek()?.get_token();
        match next_token {
            Token::RightParenthesis | Token::Semicolon => Ok(None),
            _ => Ok(Some(self.parse_expression(0)?)),
        }
    }
}
