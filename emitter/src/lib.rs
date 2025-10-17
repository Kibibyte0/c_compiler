use codegen::asm;
use shared_context::Identifier;
use shared_context::interner::Interner;
use std::fs::File;
use std::io::Write;

mod write_instructions;

pub struct Emitter<'a> {
    interner: &'a Interner<'a>,
    opcode_width: usize,
    operand_width: usize,
    indent: String,
}

impl<'a> Emitter<'a> {
    pub fn new(
        opcode_width: usize,
        operand_width: usize,
        indent_level: usize,
        interner: &'a Interner<'a>,
    ) -> Self {
        // each indentation level will be four spaces
        let indent = "    ".repeat(indent_level);
        Self {
            interner,
            opcode_width,
            operand_width,
            indent,
        }
    }

    pub fn write_program(
        &self,
        program: asm::Program,
        output_file_path: &str,
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

    fn format_identifier(&self, identifier: Identifier) -> String {
        format!("{}", self.interner.lookup(identifier.get_symbol()))
    }

    fn write_program_epilogue(&self, code: &mut String) {
        let s = format!("{}.section .note.GNU-stack,\"\",@progbits\n", self.indent);
        code.push_str(&s);
    }

    fn write_function_def(
        &self,
        function: asm::FunctionDef,
        code: &mut String,
    ) -> std::fmt::Result {
        let (name, instructions) = function.into_parts();
        self.write_function_def_prolouge(code, name);

        for instr in instructions {
            self.write_instruction(instr, code)?;
        }

        Ok(())
    }

    fn write_function_def_prolouge(&self, code: &mut String, name: asm::Identifier) {
        code.push_str(&format!(
            "{}.globl {}\n",
            self.indent,
            self.format_identifier(name)
        ));
        code.push_str(&format!("{}:\n", self.format_identifier(name)));

        let instr1 = self.format_one_operand_instruction("pushq", "%rbp");
        code.push_str(&format!("{}", instr1));

        let instr2 = self.format_two_operand_instruction("movq", "%rsp,", "%rbp");
        code.push_str(&format!("{}", instr2));
    }
}
