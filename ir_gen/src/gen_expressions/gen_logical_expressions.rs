use crate::IRgen;
use crate::tacky;
use parser::ast;

impl IRgen {
    pub(super) fn gen_logical_expr(
        &mut self,
        operator: ast::BinaryOP,
        operand1: ast::Expression,
        operand2: ast::Expression,
        instructions: &mut Vec<tacky::Instruction>,
    ) -> tacky::Value {
        match operator {
            ast::BinaryOP::LogicalAnd => self.gen_logical_and(operand1, operand2, instructions),
            ast::BinaryOP::LogicalOr => self.gen_logical_or(operand1, operand2, instructions),
            _ => unreachable!("Only LogicalAnd and LogicalOr should reach gen_logical_expr"),
        }
    }

    fn gen_logical_and(
        &mut self,
        operand1: ast::Expression,
        operand2: ast::Expression,
        instructions: &mut Vec<tacky::Instruction>,
    ) -> tacky::Value {
        let result_var = tacky::Value::Var(tacky::Identifier(self.make_temp_var()));
        let false_label = self.make_label();
        let end_label = self.make_label();

        // evaluate first expression, val1
        let val1 = self.gen_expression(operand1, instructions);

        // jump to false lable if val1 is zero
        instructions.push(tacky::Instruction::JumpIfZero(
            val1,
            tacky::Identifier(false_label.clone()),
        ));

        // evaluate second expression, val2
        let val2 = self.gen_expression(operand2, instructions);

        // jump to false label if val2 is zero
        instructions.push(tacky::Instruction::JumpIfZero(
            val2,
            tacky::Identifier(false_label.clone()),
        ));

        // both are non-zero => result = 1
        instructions.push(tacky::Instruction::Copy {
            src: tacky::Value::Constant(1),
            dst: result_var.clone(),
        });

        // jump to end
        instructions.push(tacky::Instruction::Jump(tacky::Identifier(
            end_label.clone(),
        )));

        // false label
        instructions.push(tacky::Instruction::Label(tacky::Identifier(false_label)));

        // result = 0
        instructions.push(tacky::Instruction::Copy {
            src: tacky::Value::Constant(0),
            dst: result_var.clone(),
        });

        // end label
        instructions.push(tacky::Instruction::Label(tacky::Identifier(end_label)));

        result_var
    }

    fn gen_logical_or(
        &mut self,
        operand1: ast::Expression,
        operand2: ast::Expression,
        instructions: &mut Vec<tacky::Instruction>,
    ) -> tacky::Value {
        let result_var = tacky::Value::Var(tacky::Identifier(self.make_temp_var()));
        let true_label = self.make_label();
        let end_label = self.make_label();

        // evaluate first expression, val1
        let val1 = self.gen_expression(operand1, instructions);

        // jump to true lable if val1 is not zero
        instructions.push(tacky::Instruction::JumpIfNotZero(
            val1,
            tacky::Identifier(true_label.clone()),
        ));

        // evaluate second expression, val2
        let val2 = self.gen_expression(operand2, instructions);

        // jump to true label if val2 is not zero
        instructions.push(tacky::Instruction::JumpIfNotZero(
            val2,
            tacky::Identifier(true_label.clone()),
        ));

        // both are zero => result = 0
        instructions.push(tacky::Instruction::Copy {
            src: tacky::Value::Constant(0),
            dst: result_var.clone(),
        });

        // jump to end label
        instructions.push(tacky::Instruction::Jump(tacky::Identifier(
            end_label.clone(),
        )));

        // true label
        instructions.push(tacky::Instruction::Label(tacky::Identifier(true_label)));

        // result = 1
        instructions.push(tacky::Instruction::Copy {
            src: tacky::Value::Constant(1),
            dst: result_var.clone(),
        });

        // end label
        instructions.push(tacky::Instruction::Label(tacky::Identifier(end_label)));

        result_var
    }
}
