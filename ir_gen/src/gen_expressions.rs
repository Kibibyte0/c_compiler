use crate::IRgen;
use crate::tacky;
use parser::ast::Expression;
use parser::ast::{self};

mod gen_logical_expressions;

impl<'a, 'b> IRgen<'a, 'b> {
    pub(crate) fn gen_expression(
        &mut self,
        expr: ast::Expression,
        instructions: &mut Vec<tacky::Instruction>,
    ) -> tacky::Value {
        let (expr_type, _) = expr.into_parts();
        match expr_type {
            ast::ExpressionType::Constant(int) => tacky::Value::Constant(int),

            ast::ExpressionType::Unary { operator, operand } => {
                self.gen_unary_expr(operator, *operand, instructions)
            }

            ast::ExpressionType::Binary {
                operator,
                operand1,
                operand2,
            } => self.gen_binary_expr(operator, *operand1, *operand2, instructions),
            ast::ExpressionType::Var(name) => tacky::Value::Var(name),
            ast::ExpressionType::Assignment { lvalue, rvalue } => {
                self.gen_assignment(*lvalue, *rvalue, instructions)
            }
            ast::ExpressionType::Conditional { cond, cons, alt } => {
                self.gen_conditional(*cond, *cons, *alt, instructions)
            }
        }
    }

    // generate tacky for binary expressions
    fn gen_binary_expr(
        &mut self,
        operator: ast::BinaryOP,
        operand1: ast::Expression,
        operand2: ast::Expression,
        instructions: &mut Vec<tacky::Instruction>,
    ) -> tacky::Value {
        use ast::BinaryOP;

        match operator {
            BinaryOP::LogicalAnd | BinaryOP::LogicalOr => {
                self.gen_logical_expr(operator, operand1, operand2, instructions)
            }
            _ => {
                let src1 = self.gen_expression(operand1, instructions);
                let src2 = self.gen_expression(operand2, instructions);
                let dst = self.make_temp_var();
                let tacky_op = IRgen::convert_binary_op(operator);
                instructions.push(tacky::Instruction::Binary {
                    op: tacky_op,
                    src1,
                    src2,
                    dst: dst,
                });
                dst
            }
        }
    }

    // generate takcy for unary expressions
    fn gen_unary_expr(
        &mut self,
        operator: ast::UnaryOP,
        operand: ast::Expression,
        instructions: &mut Vec<tacky::Instruction>,
    ) -> tacky::Value {
        let src = self.gen_expression(operand, instructions);
        let dst = self.make_temp_var();
        let tacky_op = IRgen::convert_unary_op(operator);
        instructions.push(tacky::Instruction::Unary {
            op: tacky_op,
            src,
            dst: dst,
        });
        dst
    }

    fn gen_assignment(
        &mut self,
        lvalue: ast::Expression,
        rvalue: ast::Expression,
        instructions: &mut Vec<tacky::Instruction>,
    ) -> tacky::Value {
        let lval = self.gen_expression(lvalue, instructions);
        let rval = self.gen_expression(rvalue, instructions);
        let instr = tacky::Instruction::Copy {
            src: rval,
            dst: lval,
        };
        instructions.push(instr);
        rval
    }

    // instruction for generating ternary expressions
    fn gen_conditional(
        &mut self,
        cond: Expression,
        cons: Expression,
        alt: Expression,
        instructions: &mut Vec<tacky::Instruction>,
    ) -> tacky::Value {
        let result_var = self.make_temp_var();
        let e2_label = self.make_label();
        let end_label = self.make_label();

        let cond_result = self.gen_expression(cond, instructions);
        instructions.push(tacky::Instruction::JumpIfZero(cond_result, e2_label));

        let value1 = self.gen_expression(cons, instructions);
        instructions.push(tacky::Instruction::Copy {
            src: value1,
            dst: result_var,
        });

        instructions.push(tacky::Instruction::Jump(end_label));

        instructions.push(tacky::Instruction::Label(e2_label));

        let value2 = self.gen_expression(alt, instructions);
        instructions.push(tacky::Instruction::Copy {
            src: value2,
            dst: result_var,
        });

        instructions.push(tacky::Instruction::Label(end_label));
        result_var
    }

    fn convert_binary_op(op: ast::BinaryOP) -> tacky::BinaryOP {
        match op {
            // arithmatic operators
            ast::BinaryOP::Add => tacky::BinaryOP::Add,
            ast::BinaryOP::Sub => tacky::BinaryOP::Sub,
            ast::BinaryOP::Mul => tacky::BinaryOP::Mul,
            ast::BinaryOP::Div => tacky::BinaryOP::Div,
            ast::BinaryOP::Mod => tacky::BinaryOP::Mod,
            // comparison operators
            ast::BinaryOP::Equal => tacky::BinaryOP::Equal,
            ast::BinaryOP::NotEqual => tacky::BinaryOP::NotEqual,
            ast::BinaryOP::GreaterThan => tacky::BinaryOP::GreaterThan,
            ast::BinaryOP::GreaterThanOrEq => tacky::BinaryOP::GreaterThanOrEq,
            ast::BinaryOP::LessThan => tacky::BinaryOP::LessThan,
            ast::BinaryOP::LessThanOrEq => tacky::BinaryOP::LessThanOrEq,
            // defualt
            _ => tacky::BinaryOP::Add,
        }
    }

    fn convert_unary_op(op: ast::UnaryOP) -> tacky::UnaryOP {
        match op {
            ast::UnaryOP::Not => tacky::UnaryOP::Not,
            ast::UnaryOP::Neg => tacky::UnaryOP::Neg,
            ast::UnaryOP::LogicalNot => tacky::UnaryOP::LogicalNot,
        }
    }
}
