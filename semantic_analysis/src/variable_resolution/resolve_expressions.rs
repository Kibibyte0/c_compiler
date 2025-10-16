use super::ResolverContext;
use crate::VariableResolver;
use crate::semantic_error::ErrorType;
use parser::ast::*;
use std::ops::Range;

impl<'a> VariableResolver<'a> {
    pub(super) fn resolve_expression(
        &mut self,
        sp_exp: Spanned<Expression>,
        ctx: &mut ResolverContext,
    ) -> Result<Spanned<Expression>, ErrorType> {
        let (exp, span) = sp_exp.into_parts();
        match exp {
            Expression::Assignment { lvalue, rvalue } => {
                self.resolve_assignment(*lvalue, *rvalue, span, ctx)
            }
            Expression::Var(sp_name) => self.resolve_variable(sp_name, span, ctx),
            Expression::Binary {
                operator,
                operand1,
                operand2,
            } => self.resolve_binary(operator, *operand1, *operand2, span, ctx),
            Expression::Unary { operator, operand } => {
                self.resolve_unary(operator, *operand, span, ctx)
            }
            Expression::Constant(int) => Ok(Spanned::new(Expression::Constant(int), span)),
            Expression::Conditional { cond, cons, alt } => {
                self.resolve_condtional(*cond, *cons, *alt, span, ctx)
            }
        }
    }

    fn resolve_assignment(
        &mut self,
        lvalue: Spanned<Expression>,
        rvalue: Spanned<Expression>,
        span: Range<usize>,
        ctx: &mut ResolverContext,
    ) -> Result<Spanned<Expression>, ErrorType> {
        let lexp = lvalue.get_node_ref();
        match lexp {
            Expression::Var(_) => Ok(Spanned::new(
                Expression::Assignment {
                    lvalue: Box::new(self.resolve_expression(lvalue, ctx)?),
                    rvalue: Box::new(self.resolve_expression(rvalue, ctx)?),
                },
                span,
            )),
            _ => {
                let lspan = lvalue.get_span_ref().clone();
                Err(ErrorType::InvalidLeftValue(lspan))
            }
        }
    }

    fn resolve_variable(
        &mut self,
        sp_name: Spanned<Identifier>,
        span: Range<usize>,
        ctx: &ResolverContext,
    ) -> Result<Spanned<Expression>, ErrorType> {
        let (name, name_span) = sp_name.into_parts();
        if let Some(id) = ctx.search_scope(&name) {
            Ok(Spanned::new(
                Expression::Var(Spanned::new(
                    Identifier::new(id.get_node_ref().get_name_copy()),
                    name_span,
                )),
                span,
            ))
        } else {
            Err(ErrorType::UseOfUndeclared(name_span))
        }
    }

    fn resolve_binary(
        &mut self,
        operator: BinaryOP,
        operand1: Spanned<Expression>,
        operand2: Spanned<Expression>,
        span: Range<usize>,
        ctx: &mut ResolverContext,
    ) -> Result<Spanned<Expression>, ErrorType> {
        Ok(Spanned::new(
            Expression::Binary {
                operator,
                operand1: Box::new(self.resolve_expression(operand1, ctx)?),
                operand2: Box::new(self.resolve_expression(operand2, ctx)?),
            },
            span,
        ))
    }

    fn resolve_unary(
        &mut self,
        operator: UnaryOP,
        operand: Spanned<Expression>,
        span: Range<usize>,
        ctx: &mut ResolverContext,
    ) -> Result<Spanned<Expression>, ErrorType> {
        Ok(Spanned::new(
            Expression::Unary {
                operator,
                operand: Box::new(self.resolve_expression(operand, ctx)?),
            },
            span,
        ))
    }

    fn resolve_condtional(
        &mut self,
        cond: Spanned<Expression>,
        cons: Spanned<Expression>,
        alt: Spanned<Expression>,
        span: Range<usize>,
        ctx: &mut ResolverContext,
    ) -> Result<Spanned<Expression>, ErrorType> {
        let cond = Box::new(self.resolve_expression(cond, ctx)?);
        let cons = Box::new(self.resolve_expression(cons, ctx)?);
        let alt = Box::new(self.resolve_expression(alt, ctx)?);

        Ok(Spanned::new(
            Expression::Conditional { cond, cons, alt },
            span,
        ))
    }
}
