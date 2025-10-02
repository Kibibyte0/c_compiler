use crate::IRgen;
use crate::tacky;
use parser::ast;

impl IRgen {
    pub(crate) fn gen_expression(
        &mut self,
        exp: ast::Expression,
        instructions: &mut Vec<tacky::Instruction>,
    ) -> tacky::Value {
        match exp {
            ast::Expression::Constant(int) => tacky::Value::Constant(int),

            ast::Expression::Unary(op, inner) => self.gen_unary(op, *inner, instructions),

            // a dummy placeholder, so it can compiler
            ast::Expression::Binary { op, left, right } => {
                self.gen_binary(op, *left, *right, instructions)
            }
        }
    }

    // generate tacky for binary expressions
    fn gen_binary(
        &mut self,
        op: ast::BinaryOP,
        left: ast::Expression,
        right: ast::Expression,
        instructions: &mut Vec<tacky::Instruction>,
    ) -> tacky::Value {
        let src1 = self.gen_expression(left, instructions);
        let src2 = self.gen_expression(right, instructions);
        let dst_name = self.make_temp_var();
        let dst = tacky::Value::Var(tacky::Identifier(dst_name.clone()));
        let tacky_op = IRgen::convert_binary_op(op);
        instructions.push(tacky::Instruction::Binary {
            op: tacky_op,
            src1,
            src2,
            dst,
        });
        tacky::Value::Var(tacky::Identifier(dst_name))
    }

    // generate takcy for unary expressions
    fn gen_unary(
        &mut self,
        op: ast::UnaryOP,
        inner: ast::Expression,
        instructions: &mut Vec<tacky::Instruction>,
    ) -> tacky::Value {
        let src = self.gen_expression(inner, instructions);
        let dst_name = self.make_temp_var();
        let dst = tacky::Value::Var(tacky::Identifier(dst_name.clone()));
        let tacky_op = IRgen::convert_unary_op(op);
        instructions.push(tacky::Instruction::Unary {
            op: tacky_op,
            src,
            dst,
        });
        tacky::Value::Var(tacky::Identifier(dst_name))
    }

    fn convert_binary_op(op: ast::BinaryOP) -> tacky::BinaryOP {
        match op {
            ast::BinaryOP::Add => tacky::BinaryOP::Add,
            ast::BinaryOP::Sub => tacky::BinaryOP::Sub,
            ast::BinaryOP::Mul => tacky::BinaryOP::Mul,
            ast::BinaryOP::Div => tacky::BinaryOP::Div,
            ast::BinaryOP::Mod => tacky::BinaryOP::Mod,
        }
    }

    fn convert_unary_op(op: ast::UnaryOP) -> tacky::UnaryOP {
        match op {
            ast::UnaryOP::Not => tacky::UnaryOP::Not,
            ast::UnaryOP::Neg => tacky::UnaryOP::Neg,
        }
    }
}
