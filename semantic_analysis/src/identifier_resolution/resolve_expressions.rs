use super::ResolverContext;
use crate::IdentifierResolver;
use crate::semantic_error::ErrorType;
use parser::ast::*;
use shared_context::SpannedIdentifier;

impl<'src, 'ctx> IdentifierResolver<'src, 'ctx> {
    /// Resolves an expression by recursively resolving all sub-expressions.
    ///
    /// This includes variables, assignments, unary/binary operations,
    /// conditional expressions, constants, and function calls.
    /// Ensures that all identifiers are declared in accessible scopes.
    pub(super) fn resolve_expression(
        &mut self,
        expr: Expression,
        resolver_ctx: &mut ResolverContext,
    ) -> Result<Expression, ErrorType> {
        let (expr_type, span) = expr.into_parts();

        let resolved_expr_type = match expr_type {
            ExpressionType::Assignment { lvalue, rvalue } => {
                self.resolve_assignment(*lvalue, *rvalue, resolver_ctx)?
            }
            ExpressionType::Var(name) => self.resolve_variable(name, resolver_ctx)?,
            ExpressionType::Binary {
                operator,
                operand1,
                operand2,
            } => self.resolve_binary(operator, *operand1, *operand2, resolver_ctx)?,
            ExpressionType::Unary { operator, operand } => {
                self.resolve_unary(operator, *operand, resolver_ctx)?
            }
            ExpressionType::Constant(int) => ExpressionType::Constant(int),
            ExpressionType::Conditional { cond, cons, alt } => {
                self.resolve_condtional(*cond, *cons, *alt, resolver_ctx)?
            }
            ExpressionType::FunctionCall { name, args } => {
                self.resolve_function_call(name, args, resolver_ctx)?
            }
        };

        Ok(Expression::new(resolved_expr_type, span))
    }

    /// Resolves an optional expression (may be `None`), returning a resolved `Option`.
    pub(crate) fn resolve_optional_expr(
        &mut self,
        optional_expr: Option<Expression>,
        resolver_ctx: &mut ResolverContext,
    ) -> Result<Option<Expression>, ErrorType> {
        match optional_expr {
            Some(expr) => Ok(Some(self.resolve_expression(expr, resolver_ctx)?)),
            None => Ok(None),
        }
    }

    /// Resolves an assignment expression.
    ///
    /// Ensures that the left-hand side is a valid l-value (currently only variables).
    fn resolve_assignment(
        &mut self,
        lvalue: Expression,
        rvalue: Expression,
        resolver_ctx: &mut ResolverContext,
    ) -> Result<ExpressionType, ErrorType> {
        let lexpr_type = lvalue.get_expr_type_ref();

        // Only variables can be assigned to
        match lexpr_type {
            ExpressionType::Var(_) => Ok(ExpressionType::Assignment {
                lvalue: Box::new(self.resolve_expression(lvalue, resolver_ctx)?),
                rvalue: Box::new(self.resolve_expression(rvalue, resolver_ctx)?),
            }),
            _ => {
                let (_, lspan) = lvalue.into_parts();
                Err(ErrorType::InvalidLeftValue(lspan))
            }
        }
    }

    /// Resolves a variable by checking if it exists in any accessible scope.
    ///
    /// Returns an error if the variable is undeclared.
    fn resolve_variable(
        &mut self,
        name: SpannedIdentifier,
        resolver_ctx: &ResolverContext,
    ) -> Result<ExpressionType, ErrorType> {
        let (identifier, span) = name.into_parts();
        let symbol = identifier.get_symbol();

        if let Some(prev_entry) = resolver_ctx.search_scope(&symbol) {
            Ok(ExpressionType::Var(prev_entry.get_sp_identifier()))
        } else {
            Err(ErrorType::UseOfUndeclared(span))
        }
    }

    /// Resolves a binary operation expression.
    ///
    /// Recursively resolves both operands.
    fn resolve_binary(
        &mut self,
        operator: BinaryOP,
        operand1: Expression,
        operand2: Expression,
        resolver_ctx: &mut ResolverContext,
    ) -> Result<ExpressionType, ErrorType> {
        Ok(ExpressionType::Binary {
            operator,
            operand1: Box::new(self.resolve_expression(operand1, resolver_ctx)?),
            operand2: Box::new(self.resolve_expression(operand2, resolver_ctx)?),
        })
    }

    /// Resolves a unary operation expression.
    ///
    /// Recursively resolves the operand.
    fn resolve_unary(
        &mut self,
        operator: UnaryOP,
        operand: Expression,
        resolver_ctx: &mut ResolverContext,
    ) -> Result<ExpressionType, ErrorType> {
        Ok(ExpressionType::Unary {
            operator,
            operand: Box::new(self.resolve_expression(operand, resolver_ctx)?),
        })
    }

    /// Resolves a conditional expression (`cond ? cons : alt`).
    ///
    /// All three sub-expressions are recursively resolved.
    fn resolve_condtional(
        &mut self,
        cond: Expression,
        cons: Expression,
        alt: Expression,
        resolver_ctx: &mut ResolverContext,
    ) -> Result<ExpressionType, ErrorType> {
        let cond = Box::new(self.resolve_expression(cond, resolver_ctx)?);
        let cons = Box::new(self.resolve_expression(cons, resolver_ctx)?);
        let alt = Box::new(self.resolve_expression(alt, resolver_ctx)?);

        Ok(ExpressionType::Conditional { cond, cons, alt })
    }

    /// Resolves a function call.
    ///
    /// Checks that the function is declared, then recursively resolves all argument expressions.
    fn resolve_function_call(
        &mut self,
        name: SpannedIdentifier,
        args: Vec<Box<Expression>>,
        resolver_ctx: &mut ResolverContext,
    ) -> Result<ExpressionType, ErrorType> {
        let symbol = name.get_identifier().get_symbol();

        if let Some(prev_entry) = resolver_ctx.search_scope(&symbol) {
            let mut resolved_args = Vec::new();
            for arg in args {
                resolved_args.push(Box::new(self.resolve_expression(*arg, resolver_ctx)?));
            }

            Ok(ExpressionType::FunctionCall {
                // use the name of the previous entry, this is a delibertate design choice
                // this will help the typechecker catch errors like using a variable as a function
                name: prev_entry.get_sp_identifier(),
                args: resolved_args,
            })
        } else {
            Err(ErrorType::UseOfUndeclared(name.get_span()))
        }
    }
}
