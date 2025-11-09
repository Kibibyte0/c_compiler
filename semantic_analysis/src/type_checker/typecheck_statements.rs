use crate::{TypeChecker, semantic_error::ErrorType};
use parser::ast::*;
use shared_context::{Identifier, type_interner::FuncTypeId};

impl<'src, 'c> TypeChecker<'src, 'c> {
    /// Type check a statement.
    pub(crate) fn typecheck_statement(
        &mut self,
        stmt: Statement,
        curr_fun: FuncTypeId,
    ) -> Result<Statement, ErrorType> {
        let (stmt_type, span) = stmt.into_parts();

        let checked_stmt_type = match stmt_type {
            StatementType::Return(expr) => self.typecheck_return_statement(expr, curr_fun)?,
            StatementType::ExprStatement(expr) => self.typecheck_expr_statement(expr)?,
            StatementType::IfStatement {
                condition,
                if_clause,
                else_clause,
            } => self.typecheck_if_statement(condition, *if_clause, else_clause, curr_fun)?,
            StatementType::Break(_) => stmt_type, // break and continue are trivially valid
            StatementType::Continue(_) => stmt_type,
            StatementType::While {
                condition,
                body,
                label,
            } => self.typecheck_while_statement(condition, *body, label, curr_fun)?,
            StatementType::DoWhile {
                condition,
                body,
                label,
            } => self.typecheck_do_while_statement(condition, *body, label, curr_fun)?,
            StatementType::For {
                init,
                condition,
                post,
                body,
                label,
            } => self.typecheck_for_statement(init, condition, post, *body, label, curr_fun)?,
            StatementType::Compound(block) => self.typecheck_compound_statement(block, curr_fun)?,
            StatementType::Null => StatementType::Null,
        };

        Ok(Statement::new(checked_stmt_type, span))
    }

    /// Type check a return statement.
    fn typecheck_return_statement(
        &mut self,
        expr: Expression,
        curr_fun: FuncTypeId,
    ) -> Result<StatementType, ErrorType> {
        // convert the expression to the type of the enclosing function return type.
        let ret_type = self.ty_interner.get(curr_fun).ret;
        let checked_expr = self.typecheck_expression(expr)?;
        let con_expr = Self::convert_to(checked_expr, ret_type);
        Ok(StatementType::Return(con_expr))
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
        curr_fun: FuncTypeId,
    ) -> Result<StatementType, ErrorType> {
        let checked_cond = self.typecheck_expression(condition)?;
        let checked_if = Box::new(self.typecheck_statement(if_clause, curr_fun)?);
        let checked_else = match else_clause {
            Some(stmt) => Some(Box::new(self.typecheck_statement(*stmt, curr_fun)?)),
            None => None,
        };

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
        curr_fun: FuncTypeId,
    ) -> Result<StatementType, ErrorType> {
        let checked_cond = self.typecheck_expression(condition)?;
        let checked_body = Box::new(self.typecheck_statement(body, curr_fun)?);
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
        curr_fun: FuncTypeId,
    ) -> Result<StatementType, ErrorType> {
        let checked_cond = self.typecheck_expression(condition)?;
        let checked_body = Box::new(self.typecheck_statement(body, curr_fun)?);
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
        curr_fun: FuncTypeId,
    ) -> Result<StatementType, ErrorType> {
        let checked_init = self.typecheck_for_init(init)?;
        let checked_condition = match condition {
            Some(expr) => Some(self.typecheck_expression(expr)?),
            None => None,
        };
        let checked_post = match post {
            Some(expr) => Some(self.typecheck_expression(expr)?),
            None => None,
        };
        let checked_body = Box::new(self.typecheck_statement(body, curr_fun)?);

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
            ForInit::D(var_decl) => Ok(ForInit::D(
                self.typecheck_local_variable_declaration(var_decl)?,
            )),
            ForInit::E(option_expr) => Ok(ForInit::E(match option_expr {
                Some(expr) => Some(self.typecheck_expression(expr)?),
                None => None,
            })),
        }
    }

    /// Type check a compound statement (block).
    fn typecheck_compound_statement(
        &mut self,
        block: Block,
        curr_fun: FuncTypeId,
    ) -> Result<StatementType, ErrorType> {
        let checked_block = self.typecheck_block(block, curr_fun)?;
        Ok(StatementType::Compound(checked_block))
    }
}
