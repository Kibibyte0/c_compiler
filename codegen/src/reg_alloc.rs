use shared_context::symbol_table::IdenAttrs;
use shared_context::symbol_table::SymbolTable;

use crate::RegisterAllocation;
use crate::asm;
use std::collections::HashMap;

impl<'ctx> RegisterAllocation<'ctx> {
    /// Create a new RegisterAllocation instance
    pub fn new(symbol_table: &'ctx SymbolTable) -> Self {
        Self {
            pseudo_reg_map: HashMap::new(), // Maps pseudo-register IDs to stack offsets
            symbol_table,
            sp_offset: 0, // Tracks the current stack offset
        }
    }

    /// Round the stack pointer offset to the next multiple of 16.
    /// x86-64 ABI requires 16-byte alignment for stack before function calls.
    fn get_sp_offset_rounded(&self) -> i32 {
        let n = -self.sp_offset; // positive size in bytes
        if n % 16 == 0 {
            return n;
        } else {
            ((n + 15) / 16) * 16
        }
    }

    /// Top-level function: allocate memory for all pseudo registers in all functions.
    pub fn allocate_registers(&mut self, program: &mut asm::Program) {
        let asm_items = program.get_mut_functions();
        for item in asm_items {
            match item {
                asm::TopLevel::F(fun_def) => self.handle_function(fun_def),
                // since all static variable defintion are at the end
                // we return when we see the first static variable defintion
                asm::TopLevel::S(_) => return,
            }
        }
    }

    /// Allocate stack space for pseudo registers in a single function.
    pub fn handle_function(&mut self, function: &mut asm::FunctionDef) {
        let instructions = function.get_mut_instructions();

        // Replace pseudo-registers in each instruction with stack addresses
        for instruction in instructions {
            self.replace_pseudo_reg(instruction);
        }

        // Reserve actual stack space at the start of function
        function.get_mut_instructions()[0] =
            asm::Instruction::AllocateStack(self.get_sp_offset_rounded());

        // Reset stack pointer offset for next function
        self.sp_offset = 0;
    }

    /// Replace pseudo-register operands in an instruction with stack addresses.
    fn replace_pseudo_reg(&mut self, instruction: &mut asm::Instruction) {
        match instruction {
            asm::Instruction::Mov { dst, src } => {
                self.to_stack(dst);
                self.to_stack(src);
            }

            asm::Instruction::Unary { op: _, dst } => {
                self.to_stack(dst);
            }

            asm::Instruction::Binary { op: _, src, dst } => {
                self.to_stack(src);
                self.to_stack(dst);
            }

            asm::Instruction::Idiv(src) => {
                self.to_stack(src);
            }

            asm::Instruction::Cmp { src, dst } => {
                self.to_stack(src);
                self.to_stack(dst);
            }

            asm::Instruction::SetCC(_, src) => {
                self.to_stack(src);
            }

            asm::Instruction::Push(src) => {
                self.to_stack(src);
            }

            // Instructions without pseudo-register operands are ignored
            _ => return,
        }
    }

    /// Convert a pseudo-register operand to a stack location if needed.
    fn to_stack(&mut self, operand: &mut asm::Operand) {
        if let asm::Operand::Pseudo(id) = operand {
            // Already mapped? Replace and return.
            if let Some(offset) = self.pseudo_reg_map.get(id) {
                *operand = asm::Operand::Stack(*offset);
                return;
            }

            // Determine if this is a static/global or needs a stack slot.
            let needs_stack = match self.symbol_table.get(*id) {
                Some(entry) => !matches!(entry.attributes, IdenAttrs::StaticAttrs { .. }),
                None => true,
            };

            if needs_stack {
                self.sp_offset -= 4;
                self.pseudo_reg_map.insert(*id, self.sp_offset);
                *operand = asm::Operand::Stack(self.sp_offset);
            } else {
                *operand = asm::Operand::Data(*id);
            }
        }
    }
}
