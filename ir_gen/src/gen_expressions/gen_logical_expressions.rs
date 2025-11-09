// Generation of short-circuiting logical expressions (`&&` and `||`).
//
// This module defines how logical AND and OR operations are lowered into Tacky
// IR while preserving short-circuit semantics:
//
// - For `a && b`, the right-hand side `b` is only evaluated if `a` is true (non-zero).
// - For `a || b`, the right-hand side `b` is only evaluated if `a` is false (zero).
//
// Each operation produces an explicit control flow with labels and conditional
// jumps, ensuring precise runtime behavior identical to C-like semantics.

use crate::IRgen;
use crate::tacky;
use parser::ast::{self};
use shared_context::{Const, Type};

impl<'a, 'b> IRgen<'a, 'b> {
    /// Generates short-circuiting logic for a logical AND expression.
    pub(super) fn gen_logical_and(
        &mut self,
        operand1: ast::Expression,
        operand2: ast::Expression,
        instructions: &mut Vec<tacky::Instruction>,
    ) -> tacky::Value {
        let result_var = self.make_temp_var(Type::Int);
        let false_label = self.make_label();
        let end_label = self.make_label();

        // Evaluate first operand (`a` in `a && b`)
        let val1 = self.gen_expression(operand1, instructions);

        // If first operand is false (zero), skip evaluating the second
        instructions.push(tacky::Instruction::JumpIfZero(val1, false_label));

        // Evaluate second operand only if first was true
        let val2 = self.gen_expression(operand2, instructions);

        // If second operand is also false, jump to false branch
        instructions.push(tacky::Instruction::JumpIfZero(val2, false_label));

        // Both operands non-zero -> result = 1 (true)
        instructions.push(tacky::Instruction::Copy {
            src: tacky::Value::Constant(Const::ConstInt(1)),
            dst: result_var,
        });

        // Skip false branch
        instructions.push(tacky::Instruction::Jump(end_label));

        // False branch -> result = 0 (false)
        instructions.push(tacky::Instruction::Label(false_label));
        instructions.push(tacky::Instruction::Copy {
            src: tacky::Value::Constant(Const::ConstInt(0)),
            dst: result_var,
        });

        // End of AND evaluation
        instructions.push(tacky::Instruction::Label(end_label));

        result_var
    }

    /// Generates short-circuiting logic for a logical OR expression.
    pub(super) fn gen_logical_or(
        &mut self,
        operand1: ast::Expression,
        operand2: ast::Expression,
        instructions: &mut Vec<tacky::Instruction>,
    ) -> tacky::Value {
        let result_var = self.make_temp_var(Type::Int);
        let true_label = self.make_label();
        let end_label = self.make_label();

        // Evaluate first operand (`a` in `a || b`)
        let val1 = self.gen_expression(operand1, instructions);

        // If first operand is true (non-zero), short-circuit to true branch
        instructions.push(tacky::Instruction::JumpIfNotZero(val1, true_label));

        // Otherwise evaluate the second operand
        let val2 = self.gen_expression(operand2, instructions);

        // If second operand is true, jump to true branch
        instructions.push(tacky::Instruction::JumpIfNotZero(val2, true_label));

        // Both operands false -> result = 0
        instructions.push(tacky::Instruction::Copy {
            src: tacky::Value::Constant(Const::ConstInt(0)),
            dst: result_var,
        });

        // Skip true branch
        instructions.push(tacky::Instruction::Jump(end_label));

        // True branch -> result = 1
        instructions.push(tacky::Instruction::Label(true_label));
        instructions.push(tacky::Instruction::Copy {
            src: tacky::Value::Constant(Const::ConstInt(1)),
            dst: result_var,
        });

        // End of OR evaluation
        instructions.push(tacky::Instruction::Label(end_label));

        result_var
    }
}
