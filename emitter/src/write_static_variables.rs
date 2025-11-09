use crate::Emitter;
use shared_context::{Identifier, StaticInit, StaticVariable, Type};
use std::io;

impl<'a> Emitter<'a> {
    /// Writes a static variable with proper section, alignment, and initializer.
    pub(crate) fn write_static_variable(
        &self,
        var_def: StaticVariable,
        out: &mut impl io::Write,
    ) -> io::Result<()> {
        let (name, external, var_type, init) = var_def.into_parts();

        // Determine assembly type and alignment
        let (asm_type, alignment) = self.static_var_properties(var_type);

        // Convert initializer to i64
        let static_init = self.static_init_value(init);

        // Emit global declaration if external
        if external {
            writeln!(out, "\t.globl {}", self.format_identifier(name))?;
        }

        // Emit either .bss (zero-initialized) or .data (non-zero)
        if static_init == 0 {
            self.emit_bss(out, name, alignment)
        } else {
            self.emit_data(out, name, asm_type, alignment, static_init)
        }
    }

    /// Determines assembly type and alignment based on variable type
    fn static_var_properties(&self, var_type: Type) -> (&'static str, usize) {
        match var_type {
            Type::Int => ("long", 4),
            _ => ("quad", 8), // adjust as needed for other types
        }
    }

    /// Converts `StaticInit` enum to an i64 for emission
    fn static_init_value(&self, init: StaticInit) -> i64 {
        match init {
            StaticInit::IntInit(i) => i as i64,
            StaticInit::LongInit(l) => l,
        }
    }

    /// Emits a zero-initialized variable in the .bss section
    fn emit_bss(
        &self,
        out: &mut impl io::Write,
        name: Identifier,
        alignment: usize,
    ) -> io::Result<()> {
        writeln!(
            out,
            "\t.bss\n\t.align {}\n{}:\n\t.zero {}",
            alignment,
            self.format_identifier(name),
            alignment
        )
    }

    /// Emits a non-zero-initialized variable in the .data section
    fn emit_data(
        &self,
        out: &mut impl io::Write,
        name: Identifier,
        asm_type: &str,
        alignment: usize,
        value: i64,
    ) -> io::Result<()> {
        writeln!(
            out,
            "\t.data\n\t.align {}\n{}:\n\t.{} {}",
            alignment,
            self.format_identifier(name),
            asm_type,
            value
        )
    }
}
