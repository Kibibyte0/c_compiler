use crate::{TypeChecker, semantic_error::ErrorType};
use parser::ast::*;
use shared_context::{Span, SpannedIdentifier, Type, symbol_table::EntryType};

impl<'src, 'ctx> TypeChecker<'src, 'ctx> {
    /// uses C conversion rules to get the common type between two types
    /// the common type is the type that an expression having type1 and type2 as operands should have
    fn get_common_type(type1: Type, type2: Type) -> Type {
        if type1 == type2 { type1 } else { Type::Long }
    }

    /// convert an Expression by wrapping it in a cast
    pub(crate) fn convert_to(expr: Expression, target_type: Type) -> Expression {
        let expr_type = expr.get_type();
        if target_type == expr_type {
            expr
        } else {
            let span = expr.get_span();
            let inner = InnerExpression::Cast {
                target_type: target_type,
                expr: Box::new(expr),
            };
            Expression::new(inner, target_type, span)
        }
    }

    /// equalize the types of two operands according to C common type rules
    ///
    /// return the tuple (converted_op1, converted_op2, common_type)
    fn equalize_operands(
        operand1: Expression,
        operand2: Expression,
    ) -> (Expression, Expression, Type) {
        let op1_type = operand1.get_type();
        let op2_type = operand2.get_type();
        let common_type = Self::get_common_type(op1_type, op2_type);
        let converted_op1 = Self::convert_to(operand1, common_type);
        let converted_op2 = Self::convert_to(operand2, common_type);
        (converted_op1, converted_op2, common_type)
    }

    /// Type checks an expression recursively.
    pub(crate) fn typecheck_expression(&self, expr: Expression) -> Result<Expression, ErrorType> {
        let (inner, expr_type, span) = expr.into_parts();

        match inner {
            InnerExpression::Constant(_) => Ok(Expression::new(inner, expr_type, span)),
            InnerExpression::Unary { operator, operand } => {
                self.typecheck_unary_expression(operator, *operand, span)
            }
            InnerExpression::Binary {
                operator,
                operand1,
                operand2,
            } => self.typecheck_binary_expression(operator, *operand1, *operand2, span),
            InnerExpression::Conditional { cond, cons, alt } => {
                self.typecheck_conditional_expression(*cond, *cons, *alt, span)
            }
            InnerExpression::Var(ident) => self.typecheck_var_expression(ident, span),
            InnerExpression::Assignment { lvalue, rvalue } => {
                self.typecheck_assignment_expression(*lvalue, *rvalue, span)
            }
            InnerExpression::FunctionCall { name, args } => {
                self.typecheck_function_call_expression(name, args, span)
            }
            InnerExpression::Cast { target_type, expr } => {
                self.typecheck_cast_expression(*expr, target_type, span)
            }
        }
    }

    /// type check cast expressions
    fn typecheck_cast_expression(
        &self,
        expr: Expression,
        target_type: Type,
        span: Span,
    ) -> Result<Expression, ErrorType> {
        let checked_expr = self.typecheck_expression(expr)?;
        if target_type != checked_expr.get_type() {
            let inner = InnerExpression::Cast {
                target_type,
                expr: Box::new(checked_expr),
            };
            Ok(Expression::new(inner, target_type, span))
        } else {
            Ok(checked_expr)
        }
    }

    /// Type check unary expressions
    fn typecheck_unary_expression(
        &self,
        operator: UnaryOP,
        operand: Expression,
        span: Span,
    ) -> Result<Expression, ErrorType> {
        let checked_operand = self.typecheck_expression(operand)?;

        let new_expr_type = match operator {
            UnaryOP::LogicalNot => Type::Int,
            _ => checked_operand.get_type(),
        };

        let inner = InnerExpression::Unary {
            operator,
            operand: Box::new(checked_operand),
        };

        Ok(Expression::new(inner, new_expr_type, span))
    }

    /// Type check binary expressions
    fn typecheck_binary_expression(
        &self,
        operator: BinaryOP,
        operand1: Expression,
        operand2: Expression,
        span: Span,
    ) -> Result<Expression, ErrorType> {
        let checked_op1 = self.typecheck_expression(operand1)?;
        let checked_op2 = self.typecheck_expression(operand2)?;

        // for logical operators, the type of operands doesn't matter, only their truthness
        // hence they are not converted
        if operator.is_logical() {
            let inner = InnerExpression::Binary {
                operator,
                operand1: Box::new(checked_op1),
                operand2: Box::new(checked_op2),
            };
            return Ok(Expression::new(inner, Type::Int, span));
        }

        let (con_op1, con_op2, common_type) = Self::equalize_operands(checked_op1, checked_op2);
        let inner = InnerExpression::Binary {
            operator,
            operand1: Box::new(con_op1),
            operand2: Box::new(con_op2),
        };

        // make the expression type int for comparison operators
        if operator.is_arithmetic() {
            Ok(Expression::new(inner, common_type, span))
        } else {
            Ok(Expression::new(inner, Type::Int, span))
        }
    }

    /// Type check conditional expressions (ternary operator)
    fn typecheck_conditional_expression(
        &self,
        cond: Expression,
        cons: Expression,
        alt: Expression,
        span: Span,
    ) -> Result<Expression, ErrorType> {
        let checked_cond = self.typecheck_expression(cond)?;
        let checked_cons = self.typecheck_expression(cons)?;
        let checked_alt = self.typecheck_expression(alt)?;
        let (con_cons, con_alt, common_type) = Self::equalize_operands(checked_cons, checked_alt);
        let inner = InnerExpression::Conditional {
            cond: Box::new(checked_cond),
            cons: Box::new(con_cons),
            alt: Box::new(con_alt),
        };
        Ok(Expression::new(inner, common_type, span))
    }

    /// Type check variable usage
    fn typecheck_var_expression(
        &self,
        sp_ident: SpannedIdentifier,
        span: Span,
    ) -> Result<Expression, ErrorType> {
        // after the identifier resolution pass, it's guaranteed that all variables expressions are in the symbol table.
        let entry = self.symbol_table.get(sp_ident.get_identifier()).unwrap();

        if let EntryType::Scalar(var_type) = entry.entry_type {
            let inner = InnerExpression::Var(sp_ident);
            Ok(Expression::new(inner, var_type, span))
        } else {
            // Attempting to use a function or incompatible type as a variable
            Err(ErrorType::FunctionAsVariable(span))
        }
    }

    /// Type check assignments
    fn typecheck_assignment_expression(
        &self,
        lvalue: Expression,
        rvalue: Expression,
        span: Span,
    ) -> Result<Expression, ErrorType> {
        let checked_lvalue = self.typecheck_expression(lvalue)?;
        let checked_rvalue = self.typecheck_expression(rvalue)?;
        let left_type = checked_lvalue.get_type();
        let con_rvalue = Self::convert_to(checked_rvalue, left_type);

        let inner = InnerExpression::Assignment {
            lvalue: Box::new(checked_lvalue),
            rvalue: Box::new(con_rvalue),
        };
        Ok(Expression::new(inner, left_type, span))
    }

    /// Type check function calls
    fn typecheck_function_call_expression(
        &self,
        sp_iden: SpannedIdentifier,
        args: Vec<Box<Expression>>,
        span: Span,
    ) -> Result<Expression, ErrorType> {
        let entry = self.symbol_table.get(sp_iden.get_identifier()).unwrap();

        // Ensure the identifier refers to a function
        if let EntryType::Func(type_id) = entry.entry_type {
            let fun_type = self.ty_interner.get(type_id);

            let params_count = fun_type.params.len();
            if args.len() != params_count {
                return Err(ErrorType::WrongNumberOfArgs {
                    span,
                    expected: params_count,
                    got: args.len(),
                });
            }
            let mut converted_args = Vec::new();
            for (arg, param_type) in args.into_iter().zip(fun_type.params) {
                let checked_arg = self.typecheck_expression(*arg)?;
                converted_args.push(Box::new(Self::convert_to(checked_arg, *param_type)));
            }
            let inner = InnerExpression::FunctionCall {
                name: sp_iden,
                args: converted_args,
            };
            Ok(Expression::new(inner, fun_type.ret, span))
        } else {
            return Err(ErrorType::VariableAsFunction(span));
        }
    }
}
