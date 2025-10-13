use crate::asm::Instruction;
use crate::asm_gen::AsmGen;
use crate::{asm, asm::Operand::Reg, asm::Register};
use ir_gen::tacky;

// this file contain impl for binary and unary operations

impl AsmGen {
    pub(super) fn handle_binary(
        op: tacky::BinaryOP,
        src1: tacky::Value,
        src2: tacky::Value,
        dst: tacky::Value,
    ) -> Vec<Instruction> {
        match op {
            tacky::BinaryOP::Div | tacky::BinaryOP::Mod => {
                Self::handle_div_mod(op, src1, src2, dst)
            }
            tacky::BinaryOP::GreaterThan
            | tacky::BinaryOP::GreaterThanOrEq
            | tacky::BinaryOP::LessThan
            | tacky::BinaryOP::LessThanOrEq
            | tacky::BinaryOP::Equal
            | tacky::BinaryOP::NotEqual => Self::handle_comparison(op, src1, src2, dst),
            _ => Self::handle_regular_form(op, src1, src2, dst),
        }
    }

    pub(super) fn handle_unary(
        op: tacky::UnaryOP,
        src: tacky::Value,
        dst: tacky::Value,
    ) -> Vec<Instruction> {
        match op {
            tacky::UnaryOP::LogicalNot => Self::handle_logical_not(src, dst),
            _ => {
                let mut new_instructions = Vec::new();
                new_instructions.push(asm::Instruction::Mov {
                    dst: Self::convert_val(&dst),
                    src: Self::convert_val(&src),
                });
                new_instructions.push(asm::Instruction::Unary {
                    op: Self::convert_unary_op(op),
                    dst: Self::convert_val(&dst),
                });
                new_instructions
            }
        }
    }

    /// handle binary operation that have a regular form
    fn handle_regular_form(
        op: tacky::BinaryOP,
        src1: tacky::Value,
        src2: tacky::Value,
        dst: tacky::Value,
    ) -> Vec<Instruction> {
        let mut new_instructions = Vec::new();
        new_instructions.push(asm::Instruction::Mov {
            dst: Self::convert_val(&dst),
            src: Self::convert_val(&src1),
        });
        new_instructions.push(asm::Instruction::Binary {
            op: Self::convert_binary_op(op),
            src: Self::convert_val(&src2),
            dst: Self::convert_val(&dst),
        });
        new_instructions
    }

    fn handle_div_mod(
        op: tacky::BinaryOP,
        src1: tacky::Value,
        src2: tacky::Value,
        dst: tacky::Value,
    ) -> Vec<Instruction> {
        let mut new_instructions = Vec::new();
        new_instructions.push(asm::Instruction::Mov {
            src: Self::convert_val(&src1),
            dst: Reg(Register::AX),
        });
        new_instructions.push(asm::Instruction::Cdq);
        new_instructions.push(asm::Instruction::Idiv(Self::convert_val(&src2)));

        let ret_reg = match op {
            tacky::BinaryOP::Mod => Reg(Register::DX),
            _ => Reg(Register::AX),
        };

        new_instructions.push(asm::Instruction::Mov {
            src: ret_reg,
            dst: Self::convert_val(&dst),
        });

        new_instructions
    }

    fn handle_logical_not(src: tacky::Value, dst: tacky::Value) -> Vec<Instruction> {
        let mut new_instructions = Vec::new();
        new_instructions.push(asm::Instruction::Cmp {
            src: asm::Operand::Immediate(0),
            dst: Self::convert_val(&src),
        });
        new_instructions.push(asm::Instruction::Mov {
            src: asm::Operand::Immediate(0),
            dst: Self::convert_val(&dst),
        });
        new_instructions.push(asm::Instruction::SetCC(
            asm::Cond::E,
            Self::convert_val(&dst),
        ));
        new_instructions
    }

    fn handle_comparison(
        op: tacky::BinaryOP,
        src1: tacky::Value,
        src2: tacky::Value,
        dst: tacky::Value,
    ) -> Vec<Instruction> {
        let mut new_instructions = Vec::new();
        new_instructions.push(asm::Instruction::Cmp {
            src: Self::convert_val(&src2),
            dst: Self::convert_val(&src1),
        });
        new_instructions.push(asm::Instruction::Mov {
            src: asm::Operand::Immediate(0),
            dst: Self::convert_val(&dst),
        });
        new_instructions.push(asm::Instruction::SetCC(
            Self::convert_comparison_op(op),
            Self::convert_val(&dst),
        ));
        new_instructions
    }

    fn convert_binary_op(op: tacky::BinaryOP) -> asm::BinaryOP {
        match op {
            tacky::BinaryOP::Add => asm::BinaryOP::Add,
            tacky::BinaryOP::Sub => asm::BinaryOP::Sub,
            tacky::BinaryOP::Mul => asm::BinaryOP::Mul,
            // there are more tacky BinaryOP variant than asm BinaryOP
            // this arm will never be reached so it have some arbitrary value
            _ => asm::BinaryOP::Add,
        }
    }

    fn convert_comparison_op(op: tacky::BinaryOP) -> asm::Cond {
        match op {
            tacky::BinaryOP::GreaterThan => asm::Cond::G,
            tacky::BinaryOP::GreaterThanOrEq => asm::Cond::GE,
            tacky::BinaryOP::LessThan => asm::Cond::L,
            tacky::BinaryOP::LessThanOrEq => asm::Cond::LE,
            tacky::BinaryOP::Equal => asm::Cond::E,
            tacky::BinaryOP::NotEqual => asm::Cond::NE,
            // this will never be reached
            _ => asm::Cond::E,
        }
    }

    fn convert_unary_op(op: tacky::UnaryOP) -> asm::UnaryOP {
        match op {
            tacky::UnaryOP::Not => asm::UnaryOP::Not,
            tacky::UnaryOP::Neg => asm::UnaryOP::Neg,
            // this will never be reached
            _ => asm::UnaryOP::Neg,
        }
    }
}
