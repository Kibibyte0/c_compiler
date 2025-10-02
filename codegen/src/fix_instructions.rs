use crate::ASMgen;
use crate::asm;

use crate::asm::{Operand::Reg, Register};

impl ASMgen {
    // this function fix up the instructions by replacing the old vector with a new one
    pub fn fix_instructions(&self, program: &mut asm::Program) {
        let function = program.get_mut_function();
        let instructions = function.get_mut_instructions();

        let mut new_instructions: Vec<asm::Instruction> = Vec::new();

        for instr in instructions.drain(..) {
            let fix_up = match &instr {
                asm::Instruction::Mov { src, dst } => {
                    ASMgen::fix_mov_instruction(dst, src, &mut new_instructions)
                }
                asm::Instruction::Binary { op, src, dst } => match op {
                    asm::BinaryOP::Add | asm::BinaryOP::Sub => {
                        ASMgen::fix_add_sub_instruction(op, src, dst, &mut new_instructions)
                    }
                    asm::BinaryOP::Mul => {
                        ASMgen::fix_mul_instruction(src, dst, &mut new_instructions)
                    }
                },
                asm::Instruction::Idiv(src) => {
                    ASMgen::fix_div_instruction(src, &mut new_instructions)
                }
                _ => false,
            };

            if !fix_up {
                new_instructions.push(instr);
            }
        }

        *instructions = new_instructions;
    }

    // fix up mov instructions, if the instruction is invalid.
    // some mov instructions will have stack address as both dst and src
    // which is not allowed in x86_64 assembly
    // return true if a fix up happens
    fn fix_mov_instruction(
        src: &asm::Operand,
        dst: &asm::Operand,
        new_instructions: &mut Vec<asm::Instruction>,
    ) -> bool {
        if ASMgen::is_stack(dst) && ASMgen::is_stack(src) {
            new_instructions.push(asm::Instruction::Mov {
                dst: asm::Operand::Reg(asm::Register::R10),
                src: src.clone(),
            });
            new_instructions.push(asm::Instruction::Mov {
                dst: dst.clone(),
                src: asm::Operand::Reg(asm::Register::R10),
            });
            true
        } else {
            false
        }
    }

    // fix binary instructions, if the instruction is invalid
    // some binary op instructions will have stack address as both dst and src
    // which is not allowed in x86_64 assembly
    // return true if a fix up happens
    fn fix_add_sub_instruction(
        op: &asm::BinaryOP,
        src: &asm::Operand,
        dst: &asm::Operand,
        new_instructions: &mut Vec<asm::Instruction>,
    ) -> bool {
        if ASMgen::is_stack(dst) && ASMgen::is_stack(src) {
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
    // some div op will have a stack address as a src, which is not allowed.
    // return true if the fix up happens
    fn fix_div_instruction(
        src: &asm::Operand,
        new_instructions: &mut Vec<asm::Instruction>,
    ) -> bool {
        if ASMgen::is_stack(src) {
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
    fn fix_mul_instruction(
        src: &asm::Operand,
        dst: &asm::Operand,
        new_instructions: &mut Vec<asm::Instruction>,
    ) -> bool {
        if ASMgen::is_stack(dst) {
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

    // helper to check if an operand is a stack address
    fn is_stack(op: &asm::Operand) -> bool {
        match op {
            asm::Operand::Stack(_) => true,
            _ => false,
        }
    }
}
