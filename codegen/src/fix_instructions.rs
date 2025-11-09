use shared_context::OperandSize;

use crate::InstructionFix;
use crate::asm::{self, Instruction, Operand::Reg, Register};

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
            Mov { size, src, dst } => Self::fix_mov(size, src, dst, new_instructions),
            Cmp { size, src, dst } => Self::fix_cmp(size, src, dst, new_instructions),
            Binary { size, op, src, dst } => match op {
                asm::BinaryOP::Add | asm::BinaryOP::Sub => {
                    Self::fix_add_sub(size, op, src, dst, new_instructions)
                }
                asm::BinaryOP::Mul => Self::fix_mul(size, src, dst, new_instructions),
            },
            Push(src) => Self::fix_push(src, new_instructions),
            Movsx { src, dst } => Self::fix_movsx(src, dst, new_instructions),
            Idiv(size, src) => Self::fix_div(size, src, new_instructions),
            // other instructions do not need fixing
            _ => false,
        }
    }

    /// Fix Push instruction, when the source is an immediate that can't fit into 32 bits
    fn fix_push(src: asm::Operand, new_instructions: &mut Vec<asm::Instruction>) -> bool {
        use OperandSize::QuadWord;
        use asm::Instruction::Push;
        use asm::Register::R10;

        let needs_fix = Self::is_immediate(src);

        let fixed_src =
            Self::mov_operand(src, R10, QuadWord, needs_fix, new_instructions).unwrap_or(src);

        if needs_fix {
            new_instructions.push(Push(fixed_src));
        }

        needs_fix
    }

    /// Fix Movsx instruction when either the source is immediate, the destination is Stack, or both
    /// return True if an instruction fix happens, false otherwise
    fn fix_movsx(
        src: asm::Operand,
        dst: asm::Operand,
        new_instructions: &mut Vec<asm::Instruction>,
    ) -> bool {
        use Instruction::{Mov, Movsx};
        use OperandSize::{LongWord, QuadWord};
        use Register::{R10, R11};

        let src_need_fix = Self::is_immediate(src);
        let dst_need_fix = Self::is_mem(dst);
        let need_fix = dst_need_fix || src_need_fix;

        let fixed_src =
            Self::mov_operand(src, R10, LongWord, src_need_fix, new_instructions).unwrap_or(src);
        let fixed_dst = if dst_need_fix { Reg(R11) } else { dst };

        if need_fix {
            new_instructions.push(Movsx {
                src: fixed_src,
                dst: fixed_dst,
            });

            if dst_need_fix {
                new_instructions.push(Mov {
                    size: QuadWord,
                    src: Reg(R11),
                    dst,
                });
            }
        }

        need_fix
    }

    /// Fix MOV instructions when both operands are stack addresses.
    /// Stack-to-stack MOV is illegal in x86_64, so use a temporary register.
    fn fix_mov(
        size: OperandSize,
        src: asm::Operand,
        dst: asm::Operand,
        new_instructions: &mut Vec<asm::Instruction>,
    ) -> bool {
        use Instruction::Mov;
        use Register::R10;

        let need_fix = (Self::is_mem(dst) && Self::is_mem(src))
            || (Self::is_large_immediate(src) && Self::is_mem(dst));
        let fixed_src =
            Self::mov_operand(src, R10, size, need_fix, new_instructions).unwrap_or(src);

        if need_fix {
            new_instructions.push(Mov {
                size,
                src: fixed_src,
                dst,
            });
        }
        need_fix
    }

    /// Fix ADD or SUB when both operands are stack addresses.
    /// Uses a temporary register R10 to hold one operand.
    fn fix_add_sub(
        size: OperandSize,
        op: asm::BinaryOP,
        src: asm::Operand,
        dst: asm::Operand,
        new_instructions: &mut Vec<asm::Instruction>,
    ) -> bool {
        use Instruction::Binary;
        use Register::R10;

        let need_fix = (Self::is_mem(dst) && Self::is_mem(src)) || Self::is_large_immediate(src);
        let fixed_src =
            Self::mov_operand(src, R10, size, need_fix, new_instructions).unwrap_or(src);

        if need_fix {
            new_instructions.push(Binary {
                size,
                op,
                src: fixed_src,
                dst,
            });
        }
        need_fix
    }

    /// Fix MUL instructions when the destination is a stack address
    /// or when the source is an immediate larger than i32::MAX.
    /// Uses temporary registers R10 (for source) and R11 (for destination)
    /// to ensure valid operands, moving the result back if needed.
    fn fix_mul(
        size: OperandSize,
        src: asm::Operand,
        dst: asm::Operand,
        new_instructions: &mut Vec<asm::Instruction>,
    ) -> bool {
        use asm::Instruction::{Binary, Mov};
        use asm::Register::{R10, R11};

        let src_needs_fix = Self::is_large_immediate(src);
        let dst_needs_fix = Self::is_mem(dst);
        let needs_fix = src_needs_fix || dst_needs_fix;

        let fixed_src =
            Self::mov_operand(src, R10, size, src_needs_fix, new_instructions).unwrap_or(src);
        let fixed_dst =
            Self::mov_operand(dst, R11, size, dst_needs_fix, new_instructions).unwrap_or(dst);

        if needs_fix {
            new_instructions.push(Binary {
                size,
                op: asm::BinaryOP::Mul,
                src: fixed_src,
                dst: fixed_dst,
            });

            if dst_needs_fix {
                new_instructions.push(Mov {
                    size,
                    src: Reg(R11),
                    dst,
                });
            }
        }

        needs_fix
    }

    /// Fix IDIV instructions when the divisor is an immediate value.
    /// Since IDIV cannot take an immediate operand, the immediate is first moved
    /// into the temporary register R10, which is then used as the divisor.
    fn fix_div(
        size: OperandSize,
        src: asm::Operand,
        new_instructions: &mut Vec<asm::Instruction>,
    ) -> bool {
        use asm::Instruction::Idiv;
        use asm::Register::R10;

        let needs_fix = Self::is_immediate(src);

        let fixed_src =
            Self::mov_operand(src, R10, size, needs_fix, new_instructions).unwrap_or(src);

        if needs_fix {
            new_instructions.push(Idiv(size, fixed_src));
        }

        needs_fix
    }

    /// Fix CMP instructions when both operands are memory addresses,
    /// when the destination is an immediate,
    /// or when the source is an immediate larger than i32::MAX.
    /// Uses temporary registers R10 and R11 as needed to hold operands.
    fn fix_cmp(
        size: OperandSize,
        src: asm::Operand,
        dst: asm::Operand,
        new_instructions: &mut Vec<asm::Instruction>,
    ) -> bool {
        use asm::Instruction::Cmp;
        use asm::Register::{R10, R11};

        let src_dst_mem = Self::is_mem(src) && Self::is_mem(dst);
        let dst_imm = Self::is_immediate(dst);
        let src_large_imm = Self::is_large_immediate(src);

        let needs_fix = src_dst_mem || dst_imm || src_large_imm;

        let fixed_src = Self::mov_operand(
            src,
            R10,
            size,
            src_dst_mem || src_large_imm,
            new_instructions,
        )
        .unwrap_or(src);
        let fixed_dst = Self::mov_operand(dst, R11, size, dst_imm, new_instructions).unwrap_or(dst);

        if needs_fix {
            new_instructions.push(Cmp {
                size,
                src: fixed_src,
                dst: fixed_dst,
            });
        }

        needs_fix
    }

    /// fix an operand by moving it into a register
    fn mov_operand(
        operand: asm::Operand,
        reg: Register,
        size: OperandSize,
        need_fix: bool,
        new_instructions: &mut Vec<asm::Instruction>,
    ) -> Option<asm::Operand> {
        if need_fix {
            new_instructions.push(Instruction::Mov {
                size,
                src: operand,
                dst: Reg(reg),
            });
            Some(Reg(reg))
        } else {
            None
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

    /// check if an Operand is an immediate that can't fit into 4 bytes
    fn is_large_immediate(op: asm::Operand) -> bool {
        if let asm::Operand::Immediate(int) = op {
            if int > i32::MAX as i64 { true } else { false }
        } else {
            false
        }
    }
}
