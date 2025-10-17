use shared_context::{Identifier, interner::Interner};
use std::collections::HashMap;

pub mod asm;
mod asm_gen;
mod debugging_printer;
mod fix_instructions;
mod reg_alloc;

// this stage consist of three different passes:
// one to generate assembly AST
// two to allocate registers
// three to fix invalid instructions

pub struct RegisterAllocation {
    pseudo_reg_map: HashMap<Identifier, i32>,
    sp_offest: i32,
}

impl RegisterAllocation {
    pub fn new() -> Self {
        Self {
            pseudo_reg_map: HashMap::new(),
            sp_offest: 0,
        }
    }
}

pub struct AsmGen;
pub struct InstructionFix;
pub struct DebuggingPrinter<'a> {
    interner: &'a Interner<'a>,
}
