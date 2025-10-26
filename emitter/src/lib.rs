use codegen::asm;
use shared_context::Identifier;
use shared_context::interner::Interner;
use shared_context::symbol_table::SymbolTable;
use std::fs::File;
use std::io::Write;

mod write_instructions;

/// Emitter is responsible for generating assembly code from the codegen
/// abstract representation
pub struct Emitter<'a> {
    /// Reference to the interner, used to resolve symbols to strings.
    interner: &'a Interner<'a>,
    /// Reference to the symbol table, which stores variable/function symbols.
    symbol_table: &'a SymbolTable,
    /// Width in characters of opcodes when formatting output.
    opcode_width: usize,
    /// Width in characters of operands when formatting output.
    operand_width: usize,
    /// String representing indentation to prepend to each line.
    indent: String,
}

impl<'a> Emitter<'a> {
    /// Constructs a new `Emitter` with the given configuration.
    pub fn new(
        opcode_width: usize,
        operand_width: usize,
        indent_level: usize,
        interner: &'a Interner<'a>,
        symbol_table: &'a SymbolTable,
    ) -> Self {
        // Each indentation level corresponds to 4 spaces
        let indent = "    ".repeat(indent_level);
        Self {
            interner,
            symbol_table,
            opcode_width,
            operand_width,
            indent,
        }
    }

    /// Writes a complete asm::Program to the given file path.
    ///
    /// Returns an `io::Error` if writing to the file fails.
    pub fn write_program(
        &self,
        program: asm::Program,
        output_file_path: &str,
    ) -> std::io::Result<()> {
        // Create the file for writing
        let mut file = File::create(output_file_path)?;
        let mut code = String::new();

        // Decompose the program into individual functions
        let functions = program.into_parts();

        for function in functions {
            // Write each function's definition to the output buffer
            self.write_function_def(function, &mut code)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        }

        // Write any necessary epilogue at the end of the program
        self.write_program_epilogue(&mut code);

        // Flush the generated code to the file
        file.write_all(code.as_bytes())?;
        Ok(())
    }

    /// Converts an Identifier to a string using the interner.
    fn format_identifier(&self, identifier: Identifier) -> String {
        format!("{}", self.interner.lookup(identifier.get_symbol()))
    }

    /// Writes a program-level epilogue, e.g., section directives.
    fn write_program_epilogue(&self, code: &mut String) {
        let s = format!("{}.section .note.GNU-stack,\"\",@progbits\n", self.indent);
        code.push_str(&s);
    }

    /// Writes a single function definition to the output buffer.
    fn write_function_def(
        &self,
        function: asm::FunctionDef,
        code: &mut String,
    ) -> std::fmt::Result {
        let (name, instructions) = function.into_parts();
        self.write_function_def_prolouge(code, name);

        for instr in instructions {
            // Write each instruction for this function
            self.write_instruction(instr, code)?;
        }

        Ok(())
    }

    /// Writes the prologue of a function, including label and stack setup.
    fn write_function_def_prolouge(&self, code: &mut String, name: Identifier) {
        // Declare function as global
        code.push_str(&format!(
            "{}.globl {}\n",
            self.indent,
            self.format_identifier(name)
        ));
        // Function label
        code.push_str(&format!("{}:\n", self.format_identifier(name)));

        // Standard prologue instructions
        let instr1 = self.format_one_operand_instruction("pushq", "%rbp");
        code.push_str(&format!("{}", instr1));

        let instr2 = self.format_two_operand_instruction("movq", "%rsp,", "%rbp");
        code.push_str(&format!("{}", instr2));
    }
}
