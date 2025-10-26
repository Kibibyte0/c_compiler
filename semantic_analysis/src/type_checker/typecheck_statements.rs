use crate::{TypeChecker, semantic_error::ErrorType};
use parser::ast::*;
use shared_context::Identifier;

impl<'src, 'c> TypeChecker<'src, 'c> {
    /// Type check a statement.
    pub(crate) fn typecheck_statement(&mut self, stmt: Statement) -> Result<Statement, ErrorType> {
        let (stmt_type, span) = stmt.into_parts();

        let checked_stmt_type = match stmt_type {
            StatementType::Return(expr) => self.typecheck_return_statement(expr)?,
            StatementType::ExprStatement(expr) => self.typecheck_expr_statement(expr)?,
            StatementType::IfStatement {
                condition,
                if_clause,
                else_clause,
            } => self.typecheck_if_statement(condition, *if_clause, else_clause)?,
            StatementType::Break(_) => stmt_type, // break and continue are trivially valid
            StatementType::Continue(_) => stmt_type,
            StatementType::While {
                condition,
                body,
                label,
            } => self.typecheck_while_statement(condition, *body, label)?,
            StatementType::DoWhile {
                condition,
                body,
                label,
            } => self.typecheck_do_while_statement(condition, *body, label)?,
            StatementType::For {
                init,
                condition,
                post,
                body,
                label,
            } => self.typecheck_for_statement(init, condition, post, *body, label)?,
            StatementType::Compound(block) => self.typecheck_compound_statement(block)?,
            StatementType::Null => StatementType::Null,
        };

        Ok(Statement::new(checked_stmt_type, span))
    }

    /// Type check a return statement.
    fn typecheck_return_statement(&mut self, expr: Expression) -> Result<StatementType, ErrorType> {
        let checked_expr = self.typecheck_expression(expr)?;
        Ok(StatementType::Return(checked_expr))
    }

    /// Type check an expression statement.
    fn typecheck_expr_statement(&mut self, expr: Expression) -> Result<StatementType, ErrorType> {
        let checked_expr = self.typecheck_expression(expr)?;
        Ok(StatementType::ExprStatement(checked_expr))
    }

    /// Type check an if statement with optional else clause.
    fn typecheck_if_statement(
        &mut self,
        condition: Expression,
        if_clause: Statement,
        else_clause: Option<Box<Statement>>,
    ) -> Result<StatementType, ErrorType> {
        let checked_cond = self.typecheck_expression(condition)?;
        let checked_if = Box::new(self.typecheck_statement(if_clause)?);
        let checked_else =
            else_clause.map(|stmt| Box::new(self.typecheck_statement(*stmt).unwrap()));

        Ok(StatementType::IfStatement {
            condition: checked_cond,
            if_clause: checked_if,
            else_clause: checked_else,
        })
    }

    /// Type check a while loop.
    fn typecheck_while_statement(
        &mut self,
        condition: Expression,
        body: Statement,
        label: Identifier,
    ) -> Result<StatementType, ErrorType> {
        let checked_cond = self.typecheck_expression(condition)?;
        let checked_body = Box::new(self.typecheck_statement(body)?);
        Ok(StatementType::While {
            condition: checked_cond,
            body: checked_body,
            label,
        })
    }

    /// Type check a do-while loop.
    fn typecheck_do_while_statement(
        &mut self,
        condition: Expression,
        body: Statement,
        label: Identifier,
    ) -> Result<StatementType, ErrorType> {
        let checked_cond = self.typecheck_expression(condition)?;
        let checked_body = Box::new(self.typecheck_statement(body)?);
        Ok(StatementType::DoWhile {
            condition: checked_cond,
            body: checked_body,
            label,
        })
    }

    /// Type check a for loop, including init, condition, post, and body.
    fn typecheck_for_statement(
        &mut self,
        init: ForInit,
        condition: Option<Expression>,
        post: Option<Expression>,
        body: Statement,
        label: Identifier,
    ) -> Result<StatementType, ErrorType> {
        let checked_init = self.typecheck_for_init(init)?;
        let checked_condition = condition.map(|expr| self.typecheck_expression(expr).unwrap());
        let checked_post = post.map(|expr| self.typecheck_expression(expr).unwrap());
        let checked_body = Box::new(self.typecheck_statement(body)?);

        Ok(StatementType::For {
            init: checked_init,
            condition: checked_condition,
            post: checked_post,
            body: checked_body,
            label,
        })
    }

    /// Type check the initialization part of a for loop.
    fn typecheck_for_init(&mut self, for_init: ForInit) -> Result<ForInit, ErrorType> {
        match for_init {
            ForInit::D(var_decl) => Ok(ForInit::D(self.typecheck_variable_declaration(var_decl)?)),
            ForInit::E(option_expr) => Ok(ForInit::E(
                option_expr.map(|expr| self.typecheck_expression(expr).unwrap()),
            )),
        }
    }

    /// Type check a compound statement (block).
    fn typecheck_compound_statement(&mut self, block: Block) -> Result<StatementType, ErrorType> {
        let checked_block = self.typecheck_block(block)?;
        Ok(StatementType::Compound(checked_block))
    }
}
