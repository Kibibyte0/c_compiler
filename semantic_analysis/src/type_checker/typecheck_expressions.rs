use crate::{TypeChecker, semantic_error::ErrorType};
use parser::ast::*;
use shared_context::{Span, SpannedIdentifier, symbol_table::Type};

impl<'src, 'ctx> TypeChecker<'src, 'ctx> {
    /// Type checks an expression recursively.
    pub(crate) fn typecheck_expression(
        &mut self,
        expr: Expression,
    ) -> Result<Expression, ErrorType> {
        let (expr_type, span) = expr.into_parts();

        let checked_expr_type = match expr_type {
            ExpressionType::Constant(_) => expr_type, // constants are trivially valid
            ExpressionType::Unary { operator, operand } => {
                self.typecheck_unary(operator, *operand)?
            }
            ExpressionType::Binary {
                operator,
                operand1,
                operand2,
            } => self.typecheck_binary(operator, *operand1, *operand2)?,
            ExpressionType::Conditional { cond, cons, alt } => {
                self.typecheck_conditional(*cond, *cons, *alt)?
            }
            ExpressionType::Var(ident) => self.typecheck_var(ident, span)?,
            ExpressionType::Assignment { lvalue, rvalue } => {
                self.typecheck_assignment(*lvalue, *rvalue)?
            }
            ExpressionType::FunctionCall { name, args } => {
                self.typecheck_function_call(name, args, span)?
            }
        };

        Ok(Expression::new(checked_expr_type, span))
    }

    /// Type check unary expressions
    fn typecheck_unary(
        &mut self,
        operator: UnaryOP,
        operand: Expression,
    ) -> Result<ExpressionType, ErrorType> {
        let checked_operand = self.typecheck_expression(operand)?;
        Ok(ExpressionType::Unary {
            operator,
            operand: Box::new(checked_operand),
        })
    }

    /// Type check binary expressions
    fn typecheck_binary(
        &mut self,
        operator: BinaryOP,
        operand1: Expression,
        operand2: Expression,
    ) -> Result<ExpressionType, ErrorType> {
        let checked_op1 = self.typecheck_expression(operand1)?;
        let checked_op2 = self.typecheck_expression(operand2)?;
        Ok(ExpressionType::Binary {
            operator,
            operand1: Box::new(checked_op1),
            operand2: Box::new(checked_op2),
        })
    }

    /// Type check conditional expressions (ternary operator)
    fn typecheck_conditional(
        &mut self,
        cond: Expression,
        cons: Expression,
        alt: Expression,
    ) -> Result<ExpressionType, ErrorType> {
        let checked_cond = self.typecheck_expression(cond)?;
        let checked_cons = self.typecheck_expression(cons)?;
        let checked_alt = self.typecheck_expression(alt)?;
        Ok(ExpressionType::Conditional {
            cond: Box::new(checked_cond),
            cons: Box::new(checked_cons),
            alt: Box::new(checked_alt),
        })
    }

    /// Type check variable usage
    fn typecheck_var(
        &mut self,
        sp_ident: SpannedIdentifier,
        span: Span,
    ) -> Result<ExpressionType, ErrorType> {
        let entry = self
            .compiler_ctx
            .symbol_table
            .get(sp_ident.get_identifier())
            .unwrap();

        if entry.entry_type != Type::Int {
            // Attempting to use a function or incompatible type as a variable
            Err(ErrorType::FunctionAsVariable(span))
        } else {
            Ok(ExpressionType::Var(sp_ident))
        }
    }

    /// Type check assignments
    fn typecheck_assignment(
        &mut self,
        lvalue: Expression,
        rvalue: Expression,
    ) -> Result<ExpressionType, ErrorType> {
        let checked_lvalue = self.typecheck_expression(lvalue)?;
        let checked_rvalue = self.typecheck_expression(rvalue)?;
        Ok(ExpressionType::Assignment {
            lvalue: Box::new(checked_lvalue),
            rvalue: Box::new(checked_rvalue),
        })
    }

    /// Type check function calls
    fn typecheck_function_call(
        &mut self,
        sp_iden: SpannedIdentifier,
        args: Vec<Box<Expression>>,
        span: Span,
    ) -> Result<ExpressionType, ErrorType> {
        let entry = self
            .compiler_ctx
            .symbol_table
            .get(sp_iden.get_identifier())
            .unwrap();

        // Ensure the identifier refers to a function
        if let Type::FunType(expected_args) = entry.entry_type {
            if args.len() != expected_args {
                return Err(ErrorType::WrongNumberOfArgs {
                    span,
                    expected: expected_args,
                    got: args.len(),
                });
            }
        } else {
            return Err(ErrorType::VariableAsFunction(span));
        }

        let mut checked_args = Vec::new();
        for arg in args {
            checked_args.push(Box::new(self.typecheck_expression(*arg)?));
        }

        // TODO: Extend to support functions returning different types
        Ok(ExpressionType::FunctionCall {
            name: sp_iden,
            args: checked_args,
        })
    }
}
