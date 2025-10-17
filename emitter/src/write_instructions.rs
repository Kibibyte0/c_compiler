use crate::Emitter;
use codegen::asm;
use std::fmt;

mod to_x86_asm;

impl<'a> Emitter<'a> {
    pub(crate) fn write_instruction(
        &self,
        instr: asm::Instruction,
        out: &mut impl std::fmt::Write,
    ) -> fmt::Result {
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
        }
    }

    fn write_mov(
        &self,
        src: asm::Operand,
        dst: asm::Operand,
        out: &mut impl std::fmt::Write,
    ) -> fmt::Result {
        let src = Emitter::convert_operand(src, true, 4);
        let dst = Emitter::convert_operand(dst, false, 4);

        let instr = self.format_two_operand_instruction("movl", &src, &dst);
        write!(out, "{}", instr)
    }

    fn write_unary_instruction(
        &self,
        op: asm::UnaryOP,
        dst: asm::Operand,
        out: &mut impl std::fmt::Write,
    ) -> fmt::Result {
        let op = Emitter::convert_unary_op(op);
        let dst = Emitter::convert_operand(dst, false, 4);

        let instr = self.format_one_operand_instruction(&op, &dst);
        write!(out, "{}", instr)
    }

    fn write_binary_instruction(
        &self,
        op: asm::BinaryOP,
        src: asm::Operand,
        dst: asm::Operand,
        out: &mut impl std::fmt::Write,
    ) -> fmt::Result {
        let op = Emitter::convert_binary_op(op);
        let src = Emitter::convert_operand(src, true, 4);
        let dst = Emitter::convert_operand(dst, false, 4);

        let instr = self.format_two_operand_instruction(&op, &src, &dst);
        write!(out, "{}", instr)
    }

    fn write_div_instruction(
        &self,
        src: asm::Operand,
        out: &mut impl std::fmt::Write,
    ) -> fmt::Result {
        let src = Emitter::convert_operand(src, false, 4);
        let instr = self.format_one_operand_instruction("idivl", &src);
        write!(out, "{}", instr)
    }

    fn write_cdq_instruction(&self, out: &mut impl std::fmt::Write) -> fmt::Result {
        writeln!(out, "{}cdq", self.indent)
    }

    fn write_stack_allocate_instruction(
        &self,
        size: i32,
        out: &mut impl std::fmt::Write,
    ) -> fmt::Result {
        let src = format!("${size},");

        let instr = self.format_two_operand_instruction("subq", &src, "%rsp");
        write!(out, "{}", instr)
    }

    fn write_return_instruction(&self, out: &mut impl std::fmt::Write) -> fmt::Result {
        let instr1 = self.format_two_operand_instruction("movq", "%rbp,", "%rsp");
        let instr2 = self.format_one_operand_instruction("popq", "%rbp");
        write!(out, "{}", instr1)?;
        write!(out, "{}", instr2)?;
        write!(out, "{}ret\n", self.indent)
    }

    fn write_cmp_instruction(
        &self,
        src1: asm::Operand,
        src2: asm::Operand,
        out: &mut impl std::fmt::Write,
    ) -> fmt::Result {
        let src1 = Emitter::convert_operand(src1, true, 4);
        let src2 = Emitter::convert_operand(src2, false, 4);

        let instr = self.format_two_operand_instruction("cmpl", &src1, &src2);
        write!(out, "{}", instr)
    }

    fn write_jmp_instruction(
        &self,
        label: asm::Identifier,
        out: &mut impl std::fmt::Write,
    ) -> fmt::Result {
        let tar = format!(".L{}", self.format_identifier(label));
        let instr = self.format_one_operand_instruction("jmp", &tar);
        write!(out, "{}", instr)
    }

    fn write_jmpcc_instruction(
        &self,
        cond: asm::Cond,
        label: asm::Identifier,
        out: &mut impl std::fmt::Write,
    ) -> fmt::Result {
        let op = format!("j{}", Emitter::convert_cond(cond));
        let tar = format!(".L{}", self.format_identifier(label));

        let instr = self.format_one_operand_instruction(&op, &tar);
        write!(out, "{}", instr)
    }

    fn write_setcc_instruction(
        &self,
        cond: asm::Cond,
        dst: asm::Operand,
        out: &mut impl std::fmt::Write,
    ) -> fmt::Result {
        let op = format!("set{}", Emitter::convert_cond(cond));
        let dst = Emitter::convert_operand(dst, false, 1);

        let instr = self.format_one_operand_instruction(&op, &dst);
        write!(out, "{}", instr)
    }

    fn write_label(&self, label: asm::Identifier, out: &mut impl std::fmt::Write) -> fmt::Result {
        let label = format!(".L{}", self.format_identifier(label));
        write!(out, "{}:\n", label)
    }

    // format instrucitons to be aligned
    pub(crate) fn format_one_operand_instruction(&self, op: &str, dst: &str) -> String {
        format!(
            "{}{:<opcode_width$} {}\n",
            self.indent,
            op,
            dst,
            opcode_width = self.opcode_width
        )
    }

    pub(crate) fn format_two_operand_instruction(&self, op: &str, src: &str, dst: &str) -> String {
        format!(
            "{}{:<opcode_width$} {:<operand_width$} {}\n",
            self.indent,
            op,
            src,
            dst,
            opcode_width = self.opcode_width,
            operand_width = self.operand_width
        )
    }
}
