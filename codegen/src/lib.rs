use asm_gen::AsmGen;
use ir_gen::tacky;
use reg_alloc::RegisterAllocation;
use shared_context::{
    SymbolRegistery, symbol_interner::SymbolInterner, type_interner::TypeInterner,
};

// These modules implement different parts of the code generation pipeline.
// Each focuses on a specific transformation step in the backend.
pub mod asm;
mod asm_gen;
mod debugging_printer;
mod fix_instructions;
mod reg_alloc;

// Overview of this stage:
//
// The code generation phase converts Tacky IR into final assembly code.
// It runs in three main passes:
//   1. Generate an assembly-level abstract syntax tree (AST).
//   2. Allocate hardware registers for pseudo-registers.
//   3. Fix or rewrite invalid instructions that violate constraints.

// Empty struct used as a namespace for instruction fix-up methods.
struct InstructionFix;

// Provides debugging utilities to print IR and assembly with resolved identifiers.
pub struct DebuggingPrinter<'a> {
    sy_interner: &'a SymbolInterner<'a>, // allows mapping identifiers to their string names
}

// Main entry point for the code generation pipeline.
//
// Takes a Tacky IR program and returns a final assembly program.
pub fn codegen<'ctx, 'src>(
    program_tacky: tacky::Program,
    ty_interner: &'ctx TypeInterner<'src>,
    symbol_reg: &'ctx SymbolRegistery,
) -> asm::Program {
    // 1. Convert Tacky IR into an assembly AST (still uses pseudo-registers).
    let mut program_asm = AsmGen::new(ty_interner, symbol_reg).gen_asm(program_tacky);

    // 2. Allocate real machine registers or stack slots to pseudo-registers.
    let mut codegen = RegisterAllocation::new(symbol_reg);
    codegen.allocate_registers(&mut program_asm);

    // 3. Fix invalid or non-encodable instructions.
    InstructionFix::fix_instructions(&mut program_asm);

    // Return the final, valid assembly program.
    program_asm
}
