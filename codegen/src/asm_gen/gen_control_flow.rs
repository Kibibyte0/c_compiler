use ir_gen::tacky;
use shared_context::Identifier;

use crate::asm;
use crate::asm_gen::AsmGen;

// TODO: handle control flow instruction generation
// impl: gen_control_flow()
impl AsmGen {
    pub(super) fn handle_jump(tar: Identifier, asm_instructions: &mut Vec<asm::Instruction>) {
        asm_instructions.push(asm::Instruction::Jmp(tar));
    }

    pub(super) fn handle_jump_if_not_zero(
        pred: tacky::Value,
        tar: Identifier,
        asm_instructions: &mut Vec<asm::Instruction>,
    ) {
        asm_instructions.push(asm::Instruction::Cmp {
            src: asm::Operand::Immediate(0),
            dst: Self::convert_val(pred),
        });
        asm_instructions.push(asm::Instruction::JmpCC(asm::Cond::NE, tar));
    }

    pub(super) fn handle_jump_if_zero(
        pred: tacky::Value,
        tar: Identifier,
        asm_instructions: &mut Vec<asm::Instruction>,
    ) {
        asm_instructions.push(asm::Instruction::Cmp {
            src: asm::Operand::Immediate(0),
            dst: Self::convert_val(pred),
        });
        asm_instructions.push(asm::Instruction::JmpCC(asm::Cond::E, tar));
    }

    pub(super) fn handle_label(tar: Identifier, asm_instructions: &mut Vec<asm::Instruction>) {
        asm_instructions.push(asm::Instruction::Label(tar));
    }
}
