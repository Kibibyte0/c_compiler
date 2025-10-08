use crate::InstructionFix;
use crate::asm;

use crate::asm::{Operand::Reg, Register};

impl InstructionFix {
    // this function fix up the instructions by replacing the old vector with a new one
    pub fn fix_instructions(program: &mut asm::Program) {
        let function = program.get_mut_function();
        let instructions = function.get_mut_instructions();

        let mut new_instructions: Vec<asm::Instruction> = Vec::new();

        for instr in instructions.drain(..) {
            let fix_up = Self::fix_instruction(&instr, &mut new_instructions);
            if !fix_up {
                new_instructions.push(instr);
            }
        }

        *instructions = new_instructions;
    }

    fn fix_instruction(
        instr: &asm::Instruction,
        new_instructions: &mut Vec<asm::Instruction>,
    ) -> bool {
        use asm::Instruction::*;

        match instr {
            Mov { src, dst } => Self::fix_mov(src, dst, new_instructions),
            Cmp(src1, src2) => Self::fix_cmp(src1, src2, new_instructions),
            Binary { op, src, dst } => match op {
                asm::BinaryOP::Add | asm::BinaryOP::Sub => {
                    Self::fix_add_sub(op, src, dst, new_instructions)
                }
                asm::BinaryOP::Mul => Self::fix_mul(src, dst, new_instructions),
            },
            Idiv(src) => Self::fix_div(src, new_instructions),
            _ => false,
        }
    }

    // fix up mov instructions, if the instruction is invalid.
    // some mov instructions might have stack address as both dst and src
    // which is not allowed in x86_64 assembly
    // return true if a fix up happens
    fn fix_mov(
        src: &asm::Operand,
        dst: &asm::Operand,
        new_instructions: &mut Vec<asm::Instruction>,
    ) -> bool {
        if Self::is_stack(dst) && Self::is_stack(src) {
            new_instructions.push(asm::Instruction::Mov {
                src: src.clone(),
                dst: asm::Operand::Reg(asm::Register::R10),
            });
            new_instructions.push(asm::Instruction::Mov {
                src: asm::Operand::Reg(asm::Register::R10),
                dst: dst.clone(),
            });
            true
        } else {
            false
        }
    }

    // fix binary instructions, if the instruction is invalid
    // some binary op instructions might have stack address as both dst and src
    // which is not allowed in x86_64 assembly
    // return true if a fix up happens
    fn fix_add_sub(
        op: &asm::BinaryOP,
        src: &asm::Operand,
        dst: &asm::Operand,
        new_instructions: &mut Vec<asm::Instruction>,
    ) -> bool {
        if Self::is_stack(dst) && Self::is_stack(src) {
            new_instructions.push(asm::Instruction::Mov {
                src: src.clone(),
                dst: Reg(Register::R10),
            });
            new_instructions.push(asm::Instruction::Binary {
                op: op.clone(),
                src: Reg(Register::R10),
                dst: dst.clone(),
            });
            true
        } else {
            false
        }
    }

    // fix div instruction, if the instruction is invalid
    // some div op might have an immediate as a src, which is not allowed.
    // return true if the fix up happens
    fn fix_div(src: &asm::Operand, new_instructions: &mut Vec<asm::Instruction>) -> bool {
        if Self::is_immediate(src) {
            new_instructions.push(asm::Instruction::Mov {
                src: src.clone(),
                dst: Reg(Register::R10),
            });
            new_instructions.push(asm::Instruction::Idiv(Reg(Register::R10)));
            true
        } else {
            false
        }
    }

    // fix mul instruction, if the instruction is invalid
    // some mul op will have a stack address as dst, which is not allowed.
    // return true if the fix up happens
    fn fix_mul(
        src: &asm::Operand,
        dst: &asm::Operand,
        new_instructions: &mut Vec<asm::Instruction>,
    ) -> bool {
        if Self::is_stack(dst) {
            new_instructions.push(asm::Instruction::Mov {
                src: dst.clone(),
                dst: Reg(Register::R11),
            });
            new_instructions.push(asm::Instruction::Binary {
                op: asm::BinaryOP::Mul,
                src: src.clone(),
                dst: Reg(Register::R11),
            });
            new_instructions.push(asm::Instruction::Mov {
                src: Reg(Register::R11),
                dst: dst.clone(),
            });
            true
        } else {
            false
        }
    }

    // fix up cmp instructions, if the instruction is invalid.
    // some cmp instructions might have stack address as both dst and src,
    // and they might have an immediate as the second operand, both of which are invalid
    // return true if a fix up happens
    fn fix_cmp(
        src1: &asm::Operand,
        src2: &asm::Operand,
        new_instructions: &mut Vec<asm::Instruction>,
    ) -> bool {
        if Self::is_stack(src1) && Self::is_stack(src2) {
            new_instructions.push(asm::Instruction::Mov {
                src: src1.clone(),
                dst: asm::Operand::Reg(asm::Register::R10),
            });
            new_instructions.push(asm::Instruction::Cmp(
                asm::Operand::Reg(Register::R10),
                src2.clone(),
            ));
            true
        } else if Self::is_immediate(src2) {
            new_instructions.push(asm::Instruction::Mov {
                src: src2.clone(),
                dst: asm::Operand::Reg(asm::Register::R11),
            });
            new_instructions.push(asm::Instruction::Cmp(
                src1.clone(),
                asm::Operand::Reg(Register::R11),
            ));
            true
        } else {
            false
        }
    }

    // helper to check if an operand is a stack address
    fn is_stack(op: &asm::Operand) -> bool {
        match op {
            asm::Operand::Stack(_) => true,
            _ => false,
        }
    }

    fn is_immediate(op: &asm::Operand) -> bool {
        match op {
            asm::Operand::Immediate(_) => true,
            _ => false,
        }
    }
}
