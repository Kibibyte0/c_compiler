use crate::asm_gen::AsmGen;
use crate::{asm, asm::Operand::Reg, asm::Register};
use ir_gen::tacky;

// This file contains implementations for lowering binary and unary operations.

impl AsmGen {
    /// Dispatches a Tacky binary operation to the correct handler.
    /// Some operations (div/mod) require special handling, comparisons generate `cmp` + `setcc`.
    pub(super) fn handle_binary(
        op: tacky::BinaryOP,
        src1: tacky::Value,
        src2: tacky::Value,
        dst: tacky::Value,
        asm_instructions: &mut Vec<asm::Instruction>,
    ) {
        match op {
            tacky::BinaryOP::Div | tacky::BinaryOP::Mod => {
                Self::handle_div_mod(op, src1, src2, dst, asm_instructions)
            }
            tacky::BinaryOP::GreaterThan
            | tacky::BinaryOP::GreaterThanOrEq
            | tacky::BinaryOP::LessThan
            | tacky::BinaryOP::LessThanOrEq
            | tacky::BinaryOP::Equal
            | tacky::BinaryOP::NotEqual => {
                Self::handle_comparison(op, src1, src2, dst, asm_instructions)
            }
            _ => Self::handle_regular_form(op, src1, src2, dst, asm_instructions),
        }
    }

    /// Dispatches a Tacky unary operation.
    /// `LogicalNot` is handled specially, other unary ops are lowered to `mov + unary`.
    pub(super) fn handle_unary(
        op: tacky::UnaryOP,
        src: tacky::Value,
        dst: tacky::Value,
        asm_instructions: &mut Vec<asm::Instruction>,
    ) {
        match op {
            tacky::UnaryOP::LogicalNot => Self::handle_logical_not(src, dst, asm_instructions),
            _ => {
                asm_instructions.push(asm::Instruction::Mov {
                    dst: Self::convert_val(dst),
                    src: Self::convert_val(src),
                });
                asm_instructions.push(asm::Instruction::Unary {
                    op: Self::convert_unary_op(op),
                    dst: Self::convert_val(dst),
                });
            }
        }
    }

    /// Lower a regular arithmetic operation (`+`, `-`, `*`) as `mov src1, dst` + `op src2, dst`.
    fn handle_regular_form(
        op: tacky::BinaryOP,
        src1: tacky::Value,
        src2: tacky::Value,
        dst: tacky::Value,
        asm_instructions: &mut Vec<asm::Instruction>,
    ) {
        asm_instructions.push(asm::Instruction::Mov {
            dst: Self::convert_val(dst),
            src: Self::convert_val(src1),
        });
        asm_instructions.push(asm::Instruction::Binary {
            op: Self::convert_binary_op(op),
            src: Self::convert_val(src2),
            dst: Self::convert_val(dst),
        });
    }

    /// Lower division and modulus, which use AX/DX registers in x86-64.
    fn handle_div_mod(
        op: tacky::BinaryOP,
        src1: tacky::Value,
        src2: tacky::Value,
        dst: tacky::Value,
        asm_instructions: &mut Vec<asm::Instruction>,
    ) {
        asm_instructions.push(asm::Instruction::Mov {
            src: Self::convert_val(src1),
            dst: Reg(Register::AX),
        });
        asm_instructions.push(asm::Instruction::Cdq); // Sign-extend AX to DX:AX
        asm_instructions.push(asm::Instruction::Idiv(Self::convert_val(src2)));

        let ret_reg = match op {
            tacky::BinaryOP::Mod => Reg(Register::DX),
            _ => Reg(Register::AX),
        };

        asm_instructions.push(asm::Instruction::Mov {
            src: ret_reg,
            dst: Self::convert_val(dst),
        });
    }

    /// Lower logical NOT (`!`) operation as `cmp + mov 0 + setcc`.
    fn handle_logical_not(
        src: tacky::Value,
        dst: tacky::Value,
        asm_instructions: &mut Vec<asm::Instruction>,
    ) {
        asm_instructions.push(asm::Instruction::Cmp {
            src: asm::Operand::Immediate(0),
            dst: Self::convert_val(src),
        });
        asm_instructions.push(asm::Instruction::Mov {
            src: asm::Operand::Immediate(0),
            dst: Self::convert_val(dst),
        });
        asm_instructions.push(asm::Instruction::SetCC(
            asm::Cond::E,
            Self::convert_val(dst),
        ));
    }

    /// Lower comparison operations (`>`, `<`, `==`, etc.) as `cmp + mov 0 + setcc`.
    fn handle_comparison(
        op: tacky::BinaryOP,
        src1: tacky::Value,
        src2: tacky::Value,
        dst: tacky::Value,
        asm_instructions: &mut Vec<asm::Instruction>,
    ) {
        asm_instructions.push(asm::Instruction::Cmp {
            src: Self::convert_val(src2),
            dst: Self::convert_val(src1),
        });
        asm_instructions.push(asm::Instruction::Mov {
            src: asm::Operand::Immediate(0),
            dst: Self::convert_val(dst),
        });
        asm_instructions.push(asm::Instruction::SetCC(
            Self::convert_comparison_op(op),
            Self::convert_val(dst),
        ));
    }

    /// Convert Tacky binary operator to ASM binary operator.
    fn convert_binary_op(op: tacky::BinaryOP) -> asm::BinaryOP {
        match op {
            tacky::BinaryOP::Add => asm::BinaryOP::Add,
            tacky::BinaryOP::Sub => asm::BinaryOP::Sub,
            tacky::BinaryOP::Mul => asm::BinaryOP::Mul,
            _ => asm::BinaryOP::Add, // unreachable for div/mod/comparison
        }
    }

    /// Convert Tacky comparison operator to ASM condition code.
    fn convert_comparison_op(op: tacky::BinaryOP) -> asm::Cond {
        match op {
            tacky::BinaryOP::GreaterThan => asm::Cond::G,
            tacky::BinaryOP::GreaterThanOrEq => asm::Cond::GE,
            tacky::BinaryOP::LessThan => asm::Cond::L,
            tacky::BinaryOP::LessThanOrEq => asm::Cond::LE,
            tacky::BinaryOP::Equal => asm::Cond::E,
            tacky::BinaryOP::NotEqual => asm::Cond::NE,
            _ => asm::Cond::E, // unreachable
        }
    }

    /// Convert Tacky unary operator to ASM unary operator.
    fn convert_unary_op(op: tacky::UnaryOP) -> asm::UnaryOP {
        match op {
            tacky::UnaryOP::Not => asm::UnaryOP::Not,
            tacky::UnaryOP::Neg => asm::UnaryOP::Neg,
            _ => asm::UnaryOP::Neg, // unreachable
        }
    }
}
