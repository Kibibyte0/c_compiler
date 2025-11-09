use codegen::asm;
use shared_context::OperandSize;

use crate::Emitter;

impl<'a> Emitter<'a> {
    /// convert an operand to it's x86_64 form, reg_size specifiy the size of the register in bytes
    /// if the operand is a register, entering an invalid size will default to $ bytes
    pub(crate) fn convert_operand(&self, operand: asm::Operand, reg_size: usize) -> String {
        let x86_operand = match operand {
            asm::Operand::Immediate(int) => format!("${int}"),
            asm::Operand::Stack(int) => format!("{int}(%rbp)"),
            asm::Operand::Reg(reg) => Emitter::convert_register(reg, reg_size),
            asm::Operand::Data(identifier) => {
                format!("{}(%rip)", self.format_identifier(identifier))
            }
            // becuase register allocation removes all pseudo registers
            // it will not get printed as x86 assembly
            asm::Operand::Pseudo(_) => "dummy string".to_string(),
        };

        x86_operand
    }

    pub(crate) fn convert_operand_size_to_suffix(size: OperandSize) -> &'static str {
        match size {
            OperandSize::LongWord => "l",
            OperandSize::QuadWord => "q",
        }
    }

    pub(crate) fn convert_operand_size_to_reg_size(size: OperandSize) -> usize {
        match size {
            OperandSize::LongWord => 4,
            OperandSize::QuadWord => 8,
        }
    }

    pub(crate) fn convert_register(register: asm::Register, reg_size: usize) -> String {
        match reg_size {
            1 => Self::convert_1_byte_reg(register),
            8 => Self::convert_8_byte_reg(register),
            // default to 4 byte register
            _ => Self::convert_4_byte_reg(register),
        }
    }

    fn convert_8_byte_reg(register: asm::Register) -> String {
        match register {
            asm::Register::AX => "%rax".to_string(),
            asm::Register::CX => "%rcx".to_string(),
            asm::Register::DX => "%rdx".to_string(),
            asm::Register::SI => "%rsi".to_string(),
            asm::Register::DI => "%rdi".to_string(),
            asm::Register::R8 => "%r8".to_string(),
            asm::Register::R9 => "%r9".to_string(),
            asm::Register::R10 => "%r10".to_string(),
            asm::Register::R11 => "%r11".to_string(),
            asm::Register::SP => "%rsp".to_string(),
        }
    }

    fn convert_4_byte_reg(register: asm::Register) -> String {
        match register {
            asm::Register::AX => "%eax".to_string(),
            asm::Register::CX => "%ecx".to_string(),
            asm::Register::DX => "%edx".to_string(),
            asm::Register::SI => "%esi".to_string(),
            asm::Register::DI => "%edi".to_string(),
            asm::Register::R8 => "%r8d".to_string(),
            asm::Register::R9 => "%r9d".to_string(),
            asm::Register::R10 => "%r10d".to_string(),
            asm::Register::R11 => "%r11d".to_string(),
            asm::Register::SP => "%esp".to_string(),
        }
    }

    fn convert_1_byte_reg(register: asm::Register) -> String {
        match register {
            asm::Register::AX => "%al".to_string(),
            asm::Register::CX => "%cl".to_string(),
            asm::Register::DX => "%dl".to_string(),
            asm::Register::SI => "%sil".to_string(),
            asm::Register::DI => "%dil".to_string(),
            asm::Register::R8 => "%r8b".to_string(),
            asm::Register::R9 => "%r9b".to_string(),
            asm::Register::R10 => "%r10b".to_string(),
            asm::Register::R11 => "%r11b".to_string(),
            asm::Register::SP => "%spl".to_string(),
        }
    }

    pub(crate) fn convert_cond(cond: asm::Cond) -> String {
        match cond {
            asm::Cond::E => "e".to_string(),
            asm::Cond::G => "g".to_string(),
            asm::Cond::L => "l".to_string(),
            asm::Cond::NE => "ne".to_string(),
            asm::Cond::GE => "ge".to_string(),
            asm::Cond::LE => "le".to_string(),
        }
    }

    pub(crate) fn convert_unary_op(operator: asm::UnaryOP) -> String {
        match operator {
            asm::UnaryOP::Neg => "neg".to_string(),
            asm::UnaryOP::Not => "not".to_string(),
        }
    }

    pub(crate) fn convert_binary_op(operator: asm::BinaryOP) -> String {
        match operator {
            asm::BinaryOP::Add => "add".to_string(),
            asm::BinaryOP::Sub => "sub".to_string(),
            asm::BinaryOP::Mul => "imul".to_string(),
        }
    }
}
