use codegen::asm_ast;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

pub struct Emitter {
    opcode_width: usize,
    operand_width: usize,
    indent: String,
}

impl Emitter {
    pub fn new(opcode_width: usize, operand_width: usize, indent_level: usize) -> Self {
        // each indentation level will be four spaces
        let indent = "    ".repeat(indent_level);
        Self {
            opcode_width,
            operand_width,
            indent,
        }
    }

    pub fn write_program(
        &self,
        program: asm_ast::Program,
        output_file_path: PathBuf,
    ) -> std::io::Result<()> {
        let mut file = File::create(output_file_path)?;
        let mut code = String::new();

        let function = program.into_parts();

        self.write_function_def(function, &mut code)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

        self.write_program_epilogue(&mut code);
        file.write_all(code.as_bytes())?;
        Ok(())
    }

    fn write_program_epilogue(&self, code: &mut String) {
        let s = format!("{}.section .note.GNU-stack,\"\",@progbits\n", self.indent);
        code.push_str(&s);
    }

    fn write_function_def(
        &self,
        function: asm_ast::FunctionDef,
        code: &mut String,
    ) -> std::fmt::Result {
        let (name, instructions) = function.into_parts();
        self.write_function_def_prolouge(code, name);

        for instr in instructions {
            self.write_instruction(instr, code)?;
        }

        Ok(())
    }

    fn write_function_def_prolouge(&self, code: &mut String, name: asm_ast::Identifier) {
        code.push_str(&format!("{}.globl {}\n", self.indent, name.0));
        code.push_str(&format!("{}:\n", name.0));

        let instr1 = self.format_one_operand_instruction("pushq", "%rsp");
        code.push_str(&format!("{}", instr1));

        let instr2 = self.format_two_operand_instruction("movq", "%rsp,", "%rbp");
        code.push_str(&format!("{}", instr2));
    }

    fn write_instruction(
        &self,
        instr: asm_ast::Instruction,
        out: &mut dyn std::fmt::Write,
    ) -> std::fmt::Result {
        match instr {
            asm_ast::Instruction::Mov { dst, src } => {
                let mut src = src.as_x86();
                src.push(',');

                let instr = self.format_two_operand_instruction("movl", &src, &dst.as_x86());
                write!(out, "{}", instr)
            }
            asm_ast::Instruction::Unary { operator, operand } => {
                let instr =
                    self.format_one_operand_instruction(&operator.as_x86(), &operand.as_x86());

                write!(out, "{}", instr)
            }
            asm_ast::Instruction::AllocateStack(size) => {
                let src = format!("${size},");

                let instr = self.format_two_operand_instruction("subq", &src, "%rsp");
                write!(out, "{}", instr)
            }

            asm_ast::Instruction::Ret => {
                let instr1 = self.format_two_operand_instruction("movq", "%rbp,", "%rsp");
                let instr2 = self.format_one_operand_instruction("popq", "&rbp");
                write!(out, "{}", instr1)?;
                write!(out, "{}", instr2)?;
                write!(out, "{}ret\n", self.indent)
            }
        }
    }

    fn format_one_operand_instruction(&self, operator: &str, operand: &str) -> String {
        format!(
            "{}{:<opcode_width$} {}\n",
            self.indent,
            operator,
            operand,
            opcode_width = self.opcode_width
        )
    }

    fn format_two_operand_instruction(&self, operator: &str, op1: &str, op2: &str) -> String {
        format!(
            "{}{:<opcode_width$} {:<operand_width$} {}\n",
            self.indent,
            operator,
            op1,
            op2,
            opcode_width = self.opcode_width,
            operand_width = self.operand_width
        )
    }
}
