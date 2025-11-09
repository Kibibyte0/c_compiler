use ir_gen::tacky;
use shared_context::Identifier;

use crate::asm;
use crate::asm_gen::AsmGen;

impl<'ctx, 'src> AsmGen<'ctx, 'src> {
    /// Emit an unconditional jump to the given target label.
    ///
    /// Corresponds directly to a single `jmp` instruction in x86-64.
    pub(super) fn handle_jump(tar: Identifier, asm_instructions: &mut Vec<asm::Instruction>) {
        asm_instructions.push(asm::Instruction::Jmp(tar));
    }

    /// Emit a conditional jump if the predicate is non-zero.
    ///
    /// Logic:
    /// 1. Compare the value with 0.
    /// 2. Jump to the target if the value is not equal to zero.
    pub(super) fn handle_jump_if_not_zero(
        &self,
        pred: tacky::Value,
        tar: Identifier,
        asm_instructions: &mut Vec<asm::Instruction>,
    ) {
        asm_instructions.push(asm::Instruction::Cmp {
            size: self.get_val_size(pred),
            src: asm::Operand::Immediate(0),
            dst: Self::convert_val(pred),
        });
        asm_instructions.push(asm::Instruction::JmpCC(asm::Cond::NE, tar));
    }

    /// Emit a conditional jump if the predicate is zero.
    ///
    /// Logic:
    /// 1. Compare the value with 0.
    /// 2. Jump to the target if the value is equal to zero.
    pub(super) fn handle_jump_if_zero(
        &self,
        pred: tacky::Value,
        tar: Identifier,
        asm_instructions: &mut Vec<asm::Instruction>,
    ) {
        asm_instructions.push(asm::Instruction::Cmp {
            size: self.get_val_size(pred),
            src: asm::Operand::Immediate(0),
            dst: Self::convert_val(pred),
        });
        asm_instructions.push(asm::Instruction::JmpCC(asm::Cond::E, tar));
    }

    /// Emit a label in the assembly instructions.
    ///
    /// Labels are used as jump targets in control flow.
    pub(super) fn handle_label(tar: Identifier, asm_instructions: &mut Vec<asm::Instruction>) {
        asm_instructions.push(asm::Instruction::Label(tar));
    }
}
