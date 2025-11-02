use crate::Emitter;
use codegen::asm::{self, Operand};
use shared_context::Identifier;
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
            asm::Instruction::Mov { dst, src } => self.write_mov(src, dst, out),
            asm::Instruction::Unary { op, dst } => self.write_unary_instruction(op, dst, out),
            asm::Instruction::AllocateStack(size) => {
                self.write_stack_allocate_instruction(size, out)
            }
            asm::Instruction::Ret => self.write_return_instruction(out),
            asm::Instruction::Binary { op, src, dst } => {
                self.write_binary_instruction(op, src, dst, out)
            }
            asm::Instruction::Idiv(src) => self.write_div_instruction(src, out),
            asm::Instruction::Cdq => self.write_cdq_instruction(out),
            asm::Instruction::Cmp { src, dst } => self.write_cmp_instruction(src, dst, out),
            asm::Instruction::Jmp(label) => self.write_jmp_instruction(label, out),
            asm::Instruction::JmpCC(cond, label) => self.write_jmpcc_instruction(cond, label, out),
            asm::Instruction::SetCC(cond, dst) => self.write_setcc_instruction(cond, dst, out),
            asm::Instruction::Label(label) => self.write_label(label, out),
            asm::Instruction::Call(name) => self.wrtie_call_instruction(name, out),
            asm::Instruction::DeallocateStack(size) => {
                self.write_stack_deallocate_instruction(size, out)
            }
            asm::Instruction::Push(src) => self.write_push_instruction(src, out),
        }
    }

    fn write_mov(
        &self,
        src: asm::Operand,
        dst: asm::Operand,
        out: &mut impl io::Write,
    ) -> io::Result<()> {
        let src = self.convert_operand(src, 4);
        let dst = self.convert_operand(dst, 4);
        self.format_two_operand_instruction("movl", &src, &dst, out)
    }

    fn write_unary_instruction(
        &self,
        op: asm::UnaryOP,
        dst: asm::Operand,
        out: &mut impl io::Write,
    ) -> io::Result<()> {
        let op = Emitter::convert_unary_op(op);
        let dst = self.convert_operand(dst, 4);
        self.format_one_operand_instruction(&op, &dst, out)
    }

    fn write_binary_instruction(
        &self,
        op: asm::BinaryOP,
        src: asm::Operand,
        dst: asm::Operand,
        out: &mut impl io::Write,
    ) -> io::Result<()> {
        let op = Emitter::convert_binary_op(op);
        let src = self.convert_operand(src, 4);
        let dst = self.convert_operand(dst, 4);
        self.format_two_operand_instruction(&op, &src, &dst, out)
    }

    fn write_div_instruction(&self, src: asm::Operand, out: &mut impl io::Write) -> io::Result<()> {
        let src = self.convert_operand(src, 4);
        self.format_one_operand_instruction("idivl", &src, out)
    }

    fn write_cdq_instruction(&self, out: &mut impl io::Write) -> io::Result<()> {
        out.write_all(b"\tcdq\n")
    }

    fn write_stack_allocate_instruction(
        &self,
        size: i32,
        out: &mut impl io::Write,
    ) -> io::Result<()> {
        let src = format!("${size}");
        self.format_two_operand_instruction("subq", &src, "%rsp", out)
    }

    fn write_stack_deallocate_instruction(
        &self,
        size: i32,
        out: &mut impl io::Write,
    ) -> io::Result<()> {
        let src = format!("${size}");
        self.format_two_operand_instruction("addq", &src, "%rsp", out)
    }

    fn write_return_instruction(&self, out: &mut impl io::Write) -> io::Result<()> {
        self.format_two_operand_instruction("movq", "%rbp", "%rsp", out)?;
        self.format_one_operand_instruction("popq", "%rbp", out)?;
        out.write_all(b"\tret\n")
    }

    fn write_cmp_instruction(
        &self,
        src1: asm::Operand,
        src2: asm::Operand,
        out: &mut impl io::Write,
    ) -> io::Result<()> {
        let src1 = self.convert_operand(src1, 4);
        let src2 = self.convert_operand(src2, 4);
        self.format_two_operand_instruction("cmpl", &src1, &src2, out)
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
        if !self.symbol_table.get(name).unwrap().attributes.is_defined() {
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
