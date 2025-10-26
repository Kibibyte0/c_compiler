use crate::IdentifierResolver;
use crate::ResolverContext;
use crate::semantic_error::ErrorType;
use parser::ast::{Expression, ForInit, Statement, StatementType};
use shared_context::Identifier;

impl<'src, 'ctx> IdentifierResolver<'src, 'ctx> {
    /// Resolves a statement by recursively resolving its inner expressions and nested statements.
    ///
    /// This includes return statements, expression statements, compound blocks, loops,
    /// conditionals, and for-loop initializations. All identifiers within the statement
    /// are checked against the current scopes in `resolver_ctx`.
    pub(crate) fn resolve_statement(
        &mut self,
        stmt: Statement,
        resolver_ctx: &mut ResolverContext,
    ) -> Result<Statement, ErrorType> {
        let (stmt_type, span) = stmt.into_parts();

        // Resolve the inner statement type
        let resolved_stmt_type = match stmt_type {
            StatementType::Return(expr) => {
                let expr = self.resolve_expression(expr, resolver_ctx)?;
                StatementType::Return(expr)
            }
            StatementType::ExprStatement(expr) => {
                let expr = self.resolve_expression(expr, resolver_ctx)?;
                StatementType::ExprStatement(expr)
            }
            StatementType::Compound(sp_block) => {
                // Compound statements introduce a new scope
                StatementType::Compound(self.resolve_block(sp_block, resolver_ctx)?)
            }
            StatementType::Continue(label) => StatementType::Continue(label),
            StatementType::Break(label) => StatementType::Break(label),
            StatementType::While {
                condition,
                body,
                label: _,
            } => self.resolve_while_statement(condition, *body, resolver_ctx)?,
            StatementType::DoWhile {
                condition,
                body,
                label: _,
            } => self.resolve_do_while_statement(condition, *body, resolver_ctx)?,
            StatementType::For {
                init,
                condition,
                post,
                body,
                label: _,
            } => self.resolve_for_statement(init, condition, post, *body, resolver_ctx)?,
            StatementType::IfStatement {
                condition,
                if_clause,
                else_clause,
            } => {
                self.resolve_if_statement_type(condition, *if_clause, else_clause, resolver_ctx)?
            }
            StatementType::Null => StatementType::Null,
        };

        Ok(Statement::new(resolved_stmt_type, span))
    }

    /// Resolves an `if` statement, recursively resolving the condition, if-clause, and optional else-clause.
    fn resolve_if_statement_type(
        &mut self,
        condition: Expression,
        if_clause: Statement,
        else_clause: Option<Box<Statement>>,
        resolver_ctx: &mut ResolverContext,
    ) -> Result<StatementType, ErrorType> {
        let condition = self.resolve_expression(condition, resolver_ctx)?;
        let if_clause = Box::new(self.resolve_statement(if_clause, resolver_ctx)?);

        let else_clause = if let Some(clause) = else_clause {
            Some(Box::new(self.resolve_statement(*clause, resolver_ctx)?))
        } else {
            None
        };

        Ok(StatementType::IfStatement {
            condition,
            if_clause,
            else_clause,
        })
    }

    /// Resolves a `while` statement, resolving the condition and body recursively.
    /// Assigns a default label identifier for potential future use.
    fn resolve_while_statement(
        &mut self,
        condition: Expression,
        body: Statement,
        resolver_ctx: &mut ResolverContext,
    ) -> Result<StatementType, ErrorType> {
        let condition = self.resolve_expression(condition, resolver_ctx)?;
        let body = Box::new(self.resolve_statement(body, resolver_ctx)?);
        Ok(StatementType::While {
            condition,
            body,
            label: Identifier::default(),
        })
    }

    /// Resolves a `do-while` statement similarly to `while`.
    fn resolve_do_while_statement(
        &mut self,
        condition: Expression,
        body: Statement,
        resolver_ctx: &mut ResolverContext,
    ) -> Result<StatementType, ErrorType> {
        let condition = self.resolve_expression(condition, resolver_ctx)?;
        let body = Box::new(self.resolve_statement(body, resolver_ctx)?);
        Ok(StatementType::DoWhile {
            condition,
            body,
            label: Identifier::default(),
        })
    }

    /// Resolves a `for` statement.
    ///
    /// Creates a new scope for the loop body and the initialization, ensuring
    /// that loop variables do not leak outside of the loop.
    fn resolve_for_statement(
        &mut self,
        init: ForInit,
        condition: Option<Expression>,
        post: Option<Expression>,
        body: Statement,
        resolver_ctx: &mut ResolverContext,
    ) -> Result<StatementType, ErrorType> {
        resolver_ctx.create_scope();

        let init = self.resolve_for_init(init, resolver_ctx)?;
        let condition = self.resolve_optional_expr(condition, resolver_ctx)?;
        let post = self.resolve_optional_expr(post, resolver_ctx)?;
        let body = Box::new(self.resolve_statement(body, resolver_ctx)?);

        resolver_ctx.delete_scope();

        Ok(StatementType::For {
            init,
            condition,
            post,
            body,
            label: Identifier::default(),
        })
    }

    /// Resolves the initialization of a `for` loop, which may be either a declaration or an expression.
    fn resolve_for_init(
        &mut self,
        init: ForInit,
        resolver_ctx: &mut ResolverContext,
    ) -> Result<ForInit, ErrorType> {
        match init {
            ForInit::D(decl) => Ok(ForInit::D(
                self.resolve_variable_declaration(decl, resolver_ctx)?,
            )),
            ForInit::E(optional_expr) => Ok(ForInit::E(
                self.resolve_optional_expr(optional_expr, resolver_ctx)?,
            )),
        }
    }
}
