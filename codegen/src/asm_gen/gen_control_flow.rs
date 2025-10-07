use ir_gen::tacky;

use crate::asm;
use crate::asm::Instruction;
use crate::asm_gen::AsmGen;

// TODO: handle control flow instruction generation
// impl: gen_control_flow()
impl AsmGen {
    pub(super) fn handle_jump(tar: tacky::Identifier) -> Vec<Instruction> {
        let mut new_instructions = Vec::new();
        new_instructions.push(asm::Instruction::Jmp(asm::Identifier(tar.0)));
        new_instructions
    }

    pub(super) fn handle_jump_if_not_zero(
        pred: tacky::Value,
        tar: tacky::Identifier,
    ) -> Vec<Instruction> {
        let mut new_instructions = Vec::new();
        new_instructions.push(asm::Instruction::Cmp(
            asm::Operand::Immediate(0),
            Self::convert_val(&pred),
        ));
        new_instructions.push(asm::Instruction::JmpCC(
            asm::Cond::NE,
            asm::Identifier(tar.0),
        ));
        new_instructions
    }

    pub(super) fn handle_jump_if_zero(
        pred: tacky::Value,
        tar: tacky::Identifier,
    ) -> Vec<Instruction> {
        let mut new_instructions = Vec::new();
        new_instructions.push(asm::Instruction::Cmp(
            asm::Operand::Immediate(0),
            Self::convert_val(&pred),
        ));
        new_instructions.push(asm::Instruction::JmpCC(
            asm::Cond::E,
            asm::Identifier(tar.0),
        ));
        new_instructions
    }

    pub(super) fn handle_label(tar: tacky::Identifier) -> Vec<Instruction> {
        let mut new_instructions = Vec::new();
        new_instructions.push(asm::Instruction::Label(asm::Identifier(tar.0)));
        new_instructions
    }
}
