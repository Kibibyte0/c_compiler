use crate::IRgen;
use crate::tacky;
use parser::ast;

mod gen_logical_expressions;

impl IRgen {
    pub(crate) fn gen_expression(
        &mut self,
        exp: ast::Expression,
        instructions: &mut Vec<tacky::Instruction>,
    ) -> tacky::Value {
        match exp {
            ast::Expression::Constant(int) => tacky::Value::Constant(int),

            ast::Expression::Unary { operator, operand } => {
                self.gen_unary_expr(operator, *operand, instructions)
            }

            ast::Expression::Binary {
                operator,
                operand1,
                operand2,
            } => self.gen_binary_expr(operator, *operand1, *operand2, instructions),
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
                let dst_name = self.make_temp_var();
                let dst = tacky::Value::Var(tacky::Identifier(dst_name.clone()));
                let tacky_op = IRgen::convert_binary_op(operator);
                instructions.push(tacky::Instruction::Binary {
                    op: tacky_op,
                    src1,
                    src2,
                    dst,
                });
                tacky::Value::Var(tacky::Identifier(dst_name))
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
        let dst_name = self.make_temp_var();
        let dst = tacky::Value::Var(tacky::Identifier(dst_name.clone()));
        let tacky_op = IRgen::convert_unary_op(operator);
        instructions.push(tacky::Instruction::Unary {
            op: tacky_op,
            src,
            dst,
        });
        tacky::Value::Var(tacky::Identifier(dst_name))
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
