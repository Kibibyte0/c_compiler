use crate::InstructionFix;
use crate::asm;
use crate::asm::{Operand::Reg, Register};

impl InstructionFix {
    /// Fix up all instructions in a program by replacing illegal operand combinations
    /// with valid ones using temporary registers.
    pub fn fix_instructions(program: &mut asm::Program) {
        let asm_items = program.get_mut_functions();
        for item in asm_items {
            match item {
                asm::TopLevel::F(fun_def) => Self::handle_function(fun_def),
                // since all static variable defintion are at the end
                // we return when we see the first static variable defintion
                asm::TopLevel::S(_) => return,
            }
        }
    }

    /// Handle a single function: process each instruction and build a new vector
    /// of instructions where invalid cases have been fixed.
    fn handle_function(function: &mut asm::FunctionDef) {
        let instructions = function.get_mut_instructions();

        let mut new_instructions: Vec<asm::Instruction> = Vec::new();

        // Drain the instructions to process them one by one
        for instr in instructions.drain(..) {
            // fix_instruction returns true if a fix was applied
            let fix_up = Self::fix_instruction(instr, &mut new_instructions);
            // if no fix was needed, push the original instruction
            if !fix_up {
                new_instructions.push(instr);
            }
        }

        // Replace the original instruction vector with the fixed one
        *instructions = new_instructions;
    }

    /// Check the type of instruction and call the appropriate fix function.
    /// Returns true if a fix was applied.
    fn fix_instruction(
        instr: asm::Instruction,
        new_instructions: &mut Vec<asm::Instruction>,
    ) -> bool {
        use asm::Instruction::*;

        match instr {
            Mov { src, dst } => Self::fix_mov(src, dst, new_instructions),
            Cmp { src, dst } => Self::fix_cmp(src, dst, new_instructions),
            Binary { op, src, dst } => match op {
                asm::BinaryOP::Add | asm::BinaryOP::Sub => {
                    Self::fix_add_sub(op, src, dst, new_instructions)
                }
                asm::BinaryOP::Mul => Self::fix_mul(src, dst, new_instructions),
            },
            Idiv(src) => Self::fix_div(src, new_instructions),
            // other instructions do not need fixing
            _ => false,
        }
    }

    /// Fix MOV instructions when both operands are stack addresses.
    /// Stack-to-stack MOV is illegal in x86_64, so use a temporary register.
    fn fix_mov(
        src: asm::Operand,
        dst: asm::Operand,
        new_instructions: &mut Vec<asm::Instruction>,
    ) -> bool {
        if Self::is_mem(dst) && Self::is_mem(src) {
            // Move src into temporary register R10
            new_instructions.push(asm::Instruction::Mov {
                src,
                dst: asm::Operand::Reg(asm::Register::R10),
            });
            // Move from R10 into dst
            new_instructions.push(asm::Instruction::Mov {
                src: asm::Operand::Reg(asm::Register::R10),
                dst,
            });
            true
        } else {
            false
        }
    }

    /// Fix ADD or SUB when both operands are stack addresses.
    /// Uses a temporary register R10 to hold one operand.
    fn fix_add_sub(
        op: asm::BinaryOP,
        src: asm::Operand,
        dst: asm::Operand,
        new_instructions: &mut Vec<asm::Instruction>,
    ) -> bool {
        if Self::is_mem(dst) && Self::is_mem(src) {
            new_instructions.push(asm::Instruction::Mov {
                src,
                dst: Reg(Register::R10),
            });
            new_instructions.push(asm::Instruction::Binary {
                op,
                src: Reg(Register::R10),
                dst,
            });
            true
        } else {
            false
        }
    }

    /// Fix MUL instructions when destination is a stack address.
    /// Use a temporary register R11 to hold the destination, then move back.
    fn fix_mul(
        src: asm::Operand,
        dst: asm::Operand,
        new_instructions: &mut Vec<asm::Instruction>,
    ) -> bool {
        if Self::is_mem(dst) {
            new_instructions.push(asm::Instruction::Mov {
                src: dst,
                dst: Reg(Register::R11),
            });
            new_instructions.push(asm::Instruction::Binary {
                op: asm::BinaryOP::Mul,
                src,
                dst: Reg(Register::R11),
            });
            new_instructions.push(asm::Instruction::Mov {
                src: Reg(Register::R11),
                dst,
            });
            true
        } else {
            false
        }
    }

    /// Fix IDIV instructions if the divisor is an immediate.
    /// IDIV cannot take an immediate, so move it to R10 first.
    fn fix_div(src: asm::Operand, new_instructions: &mut Vec<asm::Instruction>) -> bool {
        if Self::is_immediate(src) {
            new_instructions.push(asm::Instruction::Mov {
                src,
                dst: Reg(Register::R10),
            });
            new_instructions.push(asm::Instruction::Idiv(Reg(Register::R10)));
            true
        } else {
            false
        }
    }

    /// Fix CMP instructions if both operands are stack addresses or the second is an immediate.
    /// CMP cannot have stack-to-stack or src-immediate dst combination.
    fn fix_cmp(
        src: asm::Operand,
        dst: asm::Operand,
        new_instructions: &mut Vec<asm::Instruction>,
    ) -> bool {
        if Self::is_mem(src) && Self::is_mem(dst) {
            // Use R10 as a temporary for src
            new_instructions.push(asm::Instruction::Mov {
                src,
                dst: asm::Operand::Reg(asm::Register::R10),
            });
            new_instructions.push(asm::Instruction::Cmp {
                src: asm::Operand::Reg(Register::R10),
                dst,
            });
            true
        } else if Self::is_immediate(dst) {
            // Use R11 as a temporary for dst
            new_instructions.push(asm::Instruction::Mov {
                src: dst,
                dst: asm::Operand::Reg(asm::Register::R11),
            });
            new_instructions.push(asm::Instruction::Cmp {
                src,
                dst: asm::Operand::Reg(Register::R11),
            });
            true
        } else {
            false
        }
    }

    /// Helper: check if an operand is a stack address
    fn is_mem(op: asm::Operand) -> bool {
        matches!(op, asm::Operand::Stack(_) | asm::Operand::Data(_))
    }

    /// Helper: check if an operand is an immediate value
    fn is_immediate(op: asm::Operand) -> bool {
        matches!(op, asm::Operand::Immediate(_))
    }
}
