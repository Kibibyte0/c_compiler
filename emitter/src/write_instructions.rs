use crate::Emitter;
use codegen::asm::{self, Operand};
use shared_context::{Identifier, OperandSize};
use std::io;

mod to_x86_asm;

impl<'a> Emitter<'a> {
    /// Writes a single `asm::Instruction` to the output buffer.
    pub(crate) fn write_instruction(
        &self,
        instr: asm::Instruction,
        out: &mut impl io::Write,
    ) -> io::Result<()> {
        match instr {
            asm::Instruction::Mov { size, dst, src } => self.write_mov(size, src, dst, out),
            asm::Instruction::Movsx { src, dst } => self.write_movsx(src, dst, out),
            asm::Instruction::Unary { size, op, dst } => {
                self.write_unary_instruction(size, op, dst, out)
            }
            asm::Instruction::Ret => self.write_return_instruction(out),
            asm::Instruction::Binary { size, op, src, dst } => {
                self.write_binary_instruction(size, op, src, dst, out)
            }
            asm::Instruction::Idiv(size, src) => self.write_idiv_instruction(size, src, out),
            asm::Instruction::Div(size, src) => self.write_div_instruction(size, src, out),
            asm::Instruction::Cdq(size) => self.write_cdq_instruction(size, out),
            asm::Instruction::Cmp { size, src, dst } => {
                self.write_cmp_instruction(size, src, dst, out)
            }
            asm::Instruction::Jmp(label) => self.write_jmp_instruction(label, out),
            asm::Instruction::JmpCC(cond, label) => self.write_jmpcc_instruction(cond, label, out),
            asm::Instruction::SetCC(cond, dst) => self.write_setcc_instruction(cond, dst, out),
            asm::Instruction::Label(label) => self.write_label(label, out),
            asm::Instruction::Call(name) => self.wrtie_call_instruction(name, out),
            asm::Instruction::Push(src) => self.write_push_instruction(src, out),
            asm::Instruction::Movzx { .. } => Ok(()), // this instruction will be replaced before reaching code emission
        }
    }

    fn write_mov(
        &self,
        size: OperandSize,
        src: asm::Operand,
        dst: asm::Operand,
        out: &mut impl io::Write,
    ) -> io::Result<()> {
        let reg_size = Self::convert_operand_size_to_reg_size(size);
        let suffix = Self::convert_operand_size_to_suffix(size);
        let opcode = format!("mov{}", suffix);
        let src = self.convert_operand(src, reg_size);
        let dst = self.convert_operand(dst, reg_size);
        self.format_two_operand_instruction(&opcode, &src, &dst, out)
    }

    fn write_movsx(
        &self,
        src: asm::Operand,
        dst: asm::Operand,
        out: &mut impl io::Write,
    ) -> io::Result<()> {
        let src = self.convert_operand(src, 4);
        let dst = self.convert_operand(dst, 8);
        self.format_two_operand_instruction("movslq", &src, &dst, out)
    }

    fn write_unary_instruction(
        &self,
        size: OperandSize,
        op: asm::UnaryOP,
        dst: asm::Operand,
        out: &mut impl io::Write,
    ) -> io::Result<()> {
        let reg_size = Self::convert_operand_size_to_reg_size(size);
        let suffix = Self::convert_operand_size_to_suffix(size);
        let operator = Emitter::convert_unary_op(op);
        let opcode = format!("{}{}", operator, suffix);
        let dst = self.convert_operand(dst, reg_size);
        self.format_one_operand_instruction(&opcode, &dst, out)
    }

    fn write_binary_instruction(
        &self,
        size: OperandSize,
        op: asm::BinaryOP,
        src: asm::Operand,
        dst: asm::Operand,
        out: &mut impl io::Write,
    ) -> io::Result<()> {
        let reg_size = Self::convert_operand_size_to_reg_size(size);
        let suffix = Self::convert_operand_size_to_suffix(size);
        let operator = Emitter::convert_binary_op(op);
        let opcode = format!("{}{}", operator, suffix);
        let src = self.convert_operand(src, reg_size);
        let dst = self.convert_operand(dst, reg_size);
        self.format_two_operand_instruction(&opcode, &src, &dst, out)
    }

    fn write_idiv_instruction(
        &self,
        size: OperandSize,
        src: asm::Operand,
        out: &mut impl io::Write,
    ) -> io::Result<()> {
        let reg_size = Self::convert_operand_size_to_reg_size(size);
        let suffix = Self::convert_operand_size_to_suffix(size);
        let opcode = format!("idiv{}", suffix);
        let src = self.convert_operand(src, reg_size);
        self.format_one_operand_instruction(&opcode, &src, out)
    }

    fn write_div_instruction(
        &self,
        size: OperandSize,
        src: asm::Operand,
        out: &mut impl io::Write,
    ) -> io::Result<()> {
        let reg_size = Self::convert_operand_size_to_reg_size(size);
        let suffix = Self::convert_operand_size_to_suffix(size);
        let opcode = format!("div{}", suffix);
        let src = self.convert_operand(src, reg_size);
        self.format_one_operand_instruction(&opcode, &src, out)
    }

    fn write_cdq_instruction(&self, size: OperandSize, out: &mut impl io::Write) -> io::Result<()> {
        match size {
            OperandSize::LongWord => writeln!(out, "\tcdq"),
            OperandSize::QuadWord => writeln!(out, "\tcqo"),
        }
    }

    fn write_return_instruction(&self, out: &mut impl io::Write) -> io::Result<()> {
        self.format_two_operand_instruction("movq", "%rbp", "%rsp", out)?;
        self.format_one_operand_instruction("popq", "%rbp", out)?;
        out.write_all(b"\tret\n")
    }

    fn write_cmp_instruction(
        &self,
        size: OperandSize,
        src1: asm::Operand,
        src2: asm::Operand,
        out: &mut impl io::Write,
    ) -> io::Result<()> {
        let reg_size = Self::convert_operand_size_to_reg_size(size);
        let suffix = Self::convert_operand_size_to_suffix(size);
        let opcode = format!("cmp{}", suffix);
        let src1 = self.convert_operand(src1, reg_size);
        let src2 = self.convert_operand(src2, reg_size);
        self.format_two_operand_instruction(&opcode, &src1, &src2, out)
    }

    fn write_jmp_instruction(&self, label: Identifier, out: &mut impl io::Write) -> io::Result<()> {
        let tar = format!(".L{}", self.format_identifier(label));
        self.format_one_operand_instruction("jmp", &tar, out)
    }

    fn write_jmpcc_instruction(
        &self,
        cond: asm::Cond,
        label: Identifier,
        out: &mut impl io::Write,
    ) -> io::Result<()> {
        let op = format!("j{}", Emitter::convert_cond(cond));
        let tar = format!(".L{}", self.format_identifier(label));
        self.format_one_operand_instruction(&op, &tar, out)
    }

    fn write_setcc_instruction(
        &self,
        cond: asm::Cond,
        dst: asm::Operand,
        out: &mut impl io::Write,
    ) -> io::Result<()> {
        let op = format!("set{}", Emitter::convert_cond(cond));
        let dst = self.convert_operand(dst, 1);
        self.format_one_operand_instruction(&op, &dst, out)
    }

    fn write_push_instruction(&self, src: Operand, out: &mut impl io::Write) -> io::Result<()> {
        let src = self.convert_operand(src, 8);
        self.format_one_operand_instruction("pushq", &src, out)
    }

    fn wrtie_call_instruction(&self, name: Identifier, out: &mut impl io::Write) -> io::Result<()> {
        let mut fun_name = self.format_identifier(name);
        if !self.symbol_reg.get_function(&name).is_def() {
            fun_name.push_str("@PLT");
        }
        self.format_one_operand_instruction("call", &fun_name, out)
    }

    fn write_label(&self, label: Identifier, out: &mut impl io::Write) -> io::Result<()> {
        let label = format!(".L{}", self.format_identifier(label));
        writeln!(out, "{label}:")
    }

    /// format a two operand instruction and write it into out
    pub(crate) fn format_one_operand_instruction(
        &self,
        op: &str,
        dst: &str,
        out: &mut impl io::Write,
    ) -> io::Result<()> {
        writeln!(out, "\t{}\t{}", op, dst)
    }

    /// format a single operand instruction and write it into out
    pub(crate) fn format_two_operand_instruction(
        &self,
        op: &str,
        src: &str,
        dst: &str,
        out: &mut impl io::Write,
    ) -> io::Result<()> {
        writeln!(out, "\t{}\t{}, {}", op, src, dst)
    }
}
