use crate::ast::{ForInit, Span, Statement, StatementType};
use crate::parser::ParseErr;
use crate::parser::Parser;
use lexer::token::Token;
use shared_context::Identifier;

impl<'a, 'b> Parser<'a, 'b> {
    pub(crate) fn parse_statement(&mut self) -> Result<Statement, ParseErr> {
        let line = self.peek()?.get_line();
        let start = self.peek()?.get_span().start;

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
                self.advance()?; // consume the ';' token
                StatementType::Null
            }
            _ => {
                let exp = self.parse_expression(0)?;
                self.expect_token(Token::Semicolon)?;
                StatementType::ExprStatement(exp)
            }
        };

        let end = self.current_token.get_span().end;
        let span = Span::new(start, end, line);

        Ok(Statement::new(stmt_type, span))
    }

    fn parse_return_statement(&mut self) -> Result<StatementType, ParseErr> {
        self.advance()?; // consume the 'return' token
        let exp = self.parse_expression(0)?;
        self.expect_token(Token::Semicolon)?;
        Ok(StatementType::Return(exp))
    }

    fn parse_compound_statement(&mut self) -> Result<StatementType, ParseErr> {
        Ok(StatementType::Compound(self.parse_block()?))
    }

    fn parse_if_statement(&mut self) -> Result<StatementType, ParseErr> {
        self.advance()?; // consume the 'if' token

        self.expect_token(Token::LeftParenthesis)?;
        let condition = self.parse_expression(0)?;
        self.expect_token(Token::RightParenthesis)?;

        let if_clause = Box::new(self.parse_statement()?);

        let else_clause = match self.peek()?.get_token() {
            Token::Else => {
                self.advance()?; // consume the 'else' token
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

    fn parse_continue_statement(&mut self) -> Result<StatementType, ParseErr> {
        self.advance()?; // get the 'continue' token
        let stmt_type = StatementType::Continue(Identifier::default());
        self.expect_token(Token::Semicolon)?;
        Ok(stmt_type)
    }

    fn parse_break_statement(&mut self) -> Result<StatementType, ParseErr> {
        self.advance()?; // get the 'break' token
        let stmt_type = StatementType::Break(Identifier::default());
        self.expect_token(Token::Semicolon)?;
        Ok(stmt_type)
    }

    fn parse_while_statement(&mut self) -> Result<StatementType, ParseErr> {
        self.advance()?; // consume the 'while' token
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

    fn parse_do_while_statement(&mut self) -> Result<StatementType, ParseErr> {
        self.advance()?; // consume the 'do' token
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

    fn parse_for_statement(&mut self) -> Result<StatementType, ParseErr> {
        self.advance()?; // consume the 'for' token
        self.expect_token(Token::LeftParenthesis)?;
        let init = self.parse_for_init()?;
        let condition = self.parse_optional_expr()?;
        self.expect_token(Token::Semicolon)?;
        let post = self.parse_optional_expr()?;
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

    fn parse_for_init(&mut self) -> Result<ForInit, ParseErr> {
        let next_token = self.peek()?.get_token();
        match next_token {
            Token::Int => Ok(ForInit::D(self.parse_declaration()?)),
            _ => {
                let for_init = ForInit::E(self.parse_optional_expr()?);
                self.expect_token(Token::Semicolon)?;
                Ok(for_init)
            }
        }
    }
}
