// Expression lowering for the IR generator.
//
// This module defines how high-level AST expressions are translated into
// low-level Tacky IR operations. Each expression (binary, unary, function call,
// conditional, etc.) is flattened into explicit instructions with temporaries
// representing intermediate values.

use crate::IRgen;
use crate::tacky;
use parser::ast::{self, Expression};
use shared_context::SpannedIdentifier;
use shared_context::Type;

mod gen_logical_expressions;

impl<'src, 'ctx> IRgen<'src, 'ctx> {
    /// Lowers a single AST Expression into a tacky::Value.
    ///
    /// Each AST expression type is matched and translated into the appropriate
    /// sequence of Tacky instructions. Sub-expressions are recursively lowered.
    ///
    /// Returns a tacky::Value representing the result of the expression.
    pub(crate) fn gen_expression(
        &mut self,
        expr: ast::Expression,
        instructions: &mut Vec<tacky::Instruction>,
    ) -> tacky::Value {
        let (inner, expr_type, _) = expr.into_parts();

        match inner {
            // Integer literal constant.
            ast::InnerExpression::Constant(int) => tacky::Value::Constant(int),

            // Unary operation: e.g. `-x` or `!x`
            ast::InnerExpression::Unary { operator, operand } => {
                self.gen_unary_expr(operator, *operand, expr_type, instructions)
            }

            // Binary operation: e.g. `x + y`, `x * y`, etc.
            ast::InnerExpression::Binary {
                operator,
                operand1,
                operand2,
            } => self.gen_binary_expr(operator, *operand1, *operand2, expr_type, instructions),

            // Variable reference: returns a value referring to the identifier.
            ast::InnerExpression::Var(name) => tacky::Value::Var(name.get_identifier()),

            // Assignment expression: `a = b`
            ast::InnerExpression::Assignment { lvalue, rvalue } => {
                self.gen_assignment(*lvalue, *rvalue, instructions)
            }

            // Ternary conditional expression: `cond ? cons : alt`
            ast::InnerExpression::Conditional { cond, cons, alt } => {
                self.gen_conditional(*cond, *cons, *alt, expr_type, instructions)
            }

            // Function call expression: `fun(x, y, ...)`
            ast::InnerExpression::FunctionCall { name, args } => {
                self.gen_function_call(name, args, expr_type, instructions)
            }

            // Casting expression
            ast::InnerExpression::Cast { target_type, expr } => {
                self.gen_cast_expression(*expr, target_type, instructions)
            }
        }
    }

    /// Generates Tacky instructions for a binary expression.
    ///
    /// Handles arithmetic, comparison, and logical binary operators.
    /// Short-circuit logical operators (`&&`, `||`) are delegated to
    /// [`Self::gen_logical_expr`] for special handling.
    fn gen_binary_expr(
        &mut self,
        operator: ast::BinaryOP,
        operand1: ast::Expression,
        operand2: ast::Expression,
        expr_type: Type,
        instructions: &mut Vec<tacky::Instruction>,
    ) -> tacky::Value {
        use ast::BinaryOP;

        match operator {
            // Logical short-circuiting handled in a dedicated module.
            BinaryOP::LogicalAnd => self.gen_logical_and(operand1, operand2, instructions),
            BinaryOP::LogicalOr => self.gen_logical_or(operand1, operand2, instructions),

            // All other binary operators are evaluated left-to-right.
            _ => {
                let src1 = self.gen_expression(operand1, instructions);
                let src2 = self.gen_expression(operand2, instructions);
                let dst = self.make_temp_var(expr_type);

                let tacky_op = IRgen::convert_binary_op(operator);

                instructions.push(tacky::Instruction::Binary {
                    op: tacky_op,
                    src1,
                    src2,
                    dst,
                });

                dst
            }
        }
    }

    /// Generates Tacky instructions for a unary expression.
    ///
    /// Evaluates the operand, applies the unary operation, and stores
    /// the result in a temporary variable.
    fn gen_unary_expr(
        &mut self,
        operator: ast::UnaryOP,
        operand: ast::Expression,
        expr_type: Type,
        instructions: &mut Vec<tacky::Instruction>,
    ) -> tacky::Value {
        let src = self.gen_expression(operand, instructions);
        let dst = self.make_temp_var(expr_type);

        let tacky_op = IRgen::convert_unary_op(operator);

        instructions.push(tacky::Instruction::Unary {
            op: tacky_op,
            src,
            dst,
        });

        dst
    }

    /// Generates Tacky instructions for an assignment expression.
    fn gen_assignment(
        &mut self,
        lvalue: ast::Expression,
        rvalue: ast::Expression,
        instructions: &mut Vec<tacky::Instruction>,
    ) -> tacky::Value {
        let lval = self.gen_expression(lvalue, instructions);
        let rval = self.gen_expression(rvalue, instructions);

        instructions.push(tacky::Instruction::Copy {
            src: rval,
            dst: lval,
        });

        rval
    }

    /// Generates Tacky instructions for a ternary conditional expression:
    fn gen_conditional(
        &mut self,
        cond: Expression,
        cons: Expression,
        alt: Expression,
        expr_type: Type,
        instructions: &mut Vec<tacky::Instruction>,
    ) -> tacky::Value {
        let result_var = self.make_temp_var(expr_type);
        let e2_label = self.make_label();
        let end_label = self.make_label();

        // Evaluate condition
        let cond_result = self.gen_expression(cond, instructions);
        instructions.push(tacky::Instruction::JumpIfZero(cond_result, e2_label));

        // True branch
        let value1 = self.gen_expression(cons, instructions);
        instructions.push(tacky::Instruction::Copy {
            src: value1,
            dst: result_var,
        });
        instructions.push(tacky::Instruction::Jump(end_label));

        // False branch
        instructions.push(tacky::Instruction::Label(e2_label));
        let value2 = self.gen_expression(alt, instructions);
        instructions.push(tacky::Instruction::Copy {
            src: value2,
            dst: result_var,
        });

        // End label
        instructions.push(tacky::Instruction::Label(end_label));
        result_var
    }

    /// Generates Tacky instructions for a function call.
    fn gen_function_call(
        &mut self,
        sp_iden: SpannedIdentifier,
        args: Vec<Box<Expression>>,
        expr_type: Type,
        instructions: &mut Vec<tacky::Instruction>,
    ) -> tacky::Value {
        let result_var = self.make_temp_var(expr_type);

        // Evaluate all arguments in order and collect their values.
        let mut tacky_args = Vec::new();
        for arg in args {
            tacky_args.push(self.gen_expression(*arg, instructions));
        }

        // Emit the function call instruction.
        instructions.push(tacky::Instruction::FunCall {
            name: sp_iden.get_identifier(),
            args: tacky_args,
            dst: result_var,
        });

        result_var
    }

    /// generate tacky instructions for expression casting
    fn gen_cast_expression(
        &mut self,
        inner: Expression,
        target_type: Type,
        instructions: &mut Vec<tacky::Instruction>,
    ) -> tacky::Value {
        let inner_ty = inner.get_type();
        let inner_ty_size = inner_ty.size();
        let target_ty_size = target_type.size();

        let result = self.gen_expression(inner, instructions);
        let dst = self.make_temp_var(target_type);

        if target_ty_size == inner_ty_size {
            instructions.push(tacky::Instruction::Copy { src: result, dst });
        } else if target_ty_size < inner_ty_size {
            instructions.push(tacky::Instruction::Truncate { src: result, dst });
        } else if inner_ty.is_signed() {
            instructions.push(tacky::Instruction::SignExtend { src: result, dst });
        } else {
            instructions.push(tacky::Instruction::ZeroExtend { src: result, dst });
        }
        dst
    }

    /// Converts an AST-level binary operator into its Tacky equivalent.
    ///
    /// Arithmetic, comparison, and logical operators are mapped directly.
    /// Any unrecognized operator defaults to `Add` (should not occur in valid ASTs).
    fn convert_binary_op(op: ast::BinaryOP) -> tacky::BinaryOP {
        match op {
            // Arithmetic operators
            ast::BinaryOP::Add => tacky::BinaryOP::Add,
            ast::BinaryOP::Sub => tacky::BinaryOP::Sub,
            ast::BinaryOP::Mul => tacky::BinaryOP::Mul,
            ast::BinaryOP::Div => tacky::BinaryOP::Div,
            ast::BinaryOP::Mod => tacky::BinaryOP::Mod,

            // Comparison operators
            ast::BinaryOP::Equal => tacky::BinaryOP::Equal,
            ast::BinaryOP::NotEqual => tacky::BinaryOP::NotEqual,
            ast::BinaryOP::GreaterThan => tacky::BinaryOP::GreaterThan,
            ast::BinaryOP::GreaterThanOrEq => tacky::BinaryOP::GreaterThanOrEq,
            ast::BinaryOP::LessThan => tacky::BinaryOP::LessThan,
            ast::BinaryOP::LessThanOrEq => tacky::BinaryOP::LessThanOrEq,

            // Fallback (should not happen)
            _ => tacky::BinaryOP::Add,
        }
    }

    /// Converts an AST-level unary operator into its Tacky equivalent.
    fn convert_unary_op(op: ast::UnaryOP) -> tacky::UnaryOP {
        match op {
            ast::UnaryOP::BitwiseNot => tacky::UnaryOP::Not,
            ast::UnaryOP::Neg => tacky::UnaryOP::Neg,
            ast::UnaryOP::LogicalNot => tacky::UnaryOP::LogicalNot,
        }
    }
}
