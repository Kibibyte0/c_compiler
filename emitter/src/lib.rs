use codegen::asm;
use shared_context::Identifier;
use shared_context::symbol_interner::SymbolInterner;
use shared_context::symbol_table::SymbolTable;
use std::fs::File;
use std::io;

mod write_instructions;
mod write_static_variables;

/// Emitter is responsible for generating assembly code from the codegen
/// abstract representation
pub struct Emitter<'a> {
    /// Reference to the interner, used to resolve symbols to strings.
    interner: &'a SymbolInterner<'a>,
    /// Reference to the symbol table, which stores variable/function symbols.
    symbol_table: &'a SymbolTable,
}

impl<'a> Emitter<'a> {
    /// Constructs a new `Emitter` with the given configuration.
    pub fn new(interner: &'a SymbolInterner<'a>, symbol_table: &'a SymbolTable) -> Self {
        // Each indentation level corresponds to 4 spaces
        Self {
            interner,
            symbol_table,
        }
    }

    /// Writes a complete asm::Program to the given file path.
    ///
    /// Returns an `io::Error` if writing to the file fails.
    pub fn write_program(&self, program: asm::Program, output_file_path: &str) -> io::Result<()> {
        // Create the file for writing
        let mut file = File::create(output_file_path)?;

        // Decompose the program into individual functions
        let items = program.into_parts();

        for item in items {
            match item {
                asm::TopLevel::F(fun_def) => {
                    // Write each function's definition to the output buffer
                    self.write_function_def(fun_def, &mut file)?;
                }
                asm::TopLevel::S(var_def) => self.write_static_variable(var_def, &mut file)?,
            }
        }

        // Write any necessary epilogue at the end of the program
        self.write_program_epilogue(&mut file)?;
        Ok(())
    }

    /// Writes a single function definition to the output buffer.
    fn write_function_def(
        &self,
        function: asm::FunctionDef,
        out: &mut impl io::Write,
    ) -> io::Result<()> {
        let (name, external, instructions) = function.into_parts();
        self.write_function_def_prolouge(name, external, out)?;

        for instr in instructions {
            // Write each instruction for this function
            self.write_instruction(instr, out)?;
        }

        Ok(())
    }

    /// Writes a program-level epilogue, e.g., section directives.
    fn write_program_epilogue(&self, out: &mut impl io::Write) -> io::Result<()> {
        writeln!(out, "\t.section .note.GNU-stack,\"\",@progbits")
    }

    /// Writes the prologue of a function, including label and stack setup.
    fn write_function_def_prolouge(
        &self,
        name: Identifier,
        external: bool,
        out: &mut impl io::Write,
    ) -> io::Result<()> {
        let fun_name = self.format_identifier(name);
        // Declare function as global if it has an external linkage
        if external {
            writeln!(out, "\t.globl {}", fun_name)?;
        }
        writeln!(out, "\t.text")?;

        // Function label
        writeln!(out, "{}:", fun_name)?;

        // Standard prologue instructions
        self.format_one_operand_instruction("pushq", "%rbp", out)?;
        self.format_two_operand_instruction("movq", "%rsp", "%rbp", out)
    }

    /// Converts an Identifier to a string using the interner.
    // local static variables needs to be formatted with their IDs to avoid conflicts in the same file
    fn format_identifier(&self, identifier: Identifier) -> String {
        // default id is 0 when variables are created during parsing
        // if it's not zero then this means a conflict happend, and the id needs
        // to be printed alongside the symbol to resolve the conflict
        if identifier.get_id() == 0 {
            format!("{}", self.interner.lookup(identifier.get_symbol()))
        } else {
            format!(
                "{}.{}",
                self.interner.lookup(identifier.get_symbol()),
                identifier.get_id()
            )
        }
    }
}
