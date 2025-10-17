use super::ResolverContext;
use crate::VariableResolver;
use crate::semantic_error::ErrorType;
use parser::ast::*;
use shared_context::Identifier;

impl<'a, 'c> VariableResolver<'a, 'c> {
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
        };

        Ok(Expression::new(resolved_expr_type, span))
    }

    fn resolve_assignment(
        &mut self,
        lvalue: Expression,
        rvalue: Expression,
        resolver_ctx: &mut ResolverContext,
    ) -> Result<ExpressionType, ErrorType> {
        let lexpr_type = lvalue.get_expr_type_ref();
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

    fn resolve_variable(
        &mut self,
        name: Identifier,
        resolver_ctx: &ResolverContext,
    ) -> Result<ExpressionType, ErrorType> {
        let symbol = name.get_symbol();
        if let Some(id) = resolver_ctx.search_scope(&symbol) {
            Ok(ExpressionType::Var(id))
        } else {
            let (_, _, span) = name.into_parts();
            Err(ErrorType::UseOfUndeclared(span))
        }
    }

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
}
