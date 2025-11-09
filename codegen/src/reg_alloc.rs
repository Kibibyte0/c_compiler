use shared_context::Identifier;
use shared_context::OperandSize;
use shared_context::asm_symbol_table::AsmSymbolEntry;
use shared_context::asm_symbol_table::AsmSymbolTable;

use crate::asm;
use std::collections::HashMap;

// Stores the mapping from Tacky-level pseudo-registers to real registers or stack offsets.
pub(super) struct RegisterAllocation<'ctx> {
    pseudo_reg_map: HashMap<Identifier, i64>, // maps each variable to a register or stack slot
    asm_symbol_table: &'ctx AsmSymbolTable,   // used to resolve which variables are static
    sp_offset: i64,                           // current stack pointer offset (for spilled vars)
}

impl<'ctx> RegisterAllocation<'ctx> {
    /// Create a new RegisterAllocation instance
    pub fn new(asm_symbol_table: &'ctx AsmSymbolTable) -> Self {
        Self {
            pseudo_reg_map: HashMap::new(), // Maps pseudo-register IDs to stack offsets
            asm_symbol_table,
            sp_offset: 0, // Tracks the current stack offset
        }
    }

    /// Round the stack pointer offset to the next multiple of 16.
    /// x86-64 ABI requires 16-byte alignment for stack before function calls.
    fn get_sp_offset_rounded_to_16(&self) -> i64 {
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
        function.get_mut_instructions()[0] = asm::Instruction::Binary {
            op: asm::BinaryOP::Sub,
            size: OperandSize::QuadWord,
            src: asm::Operand::Immediate(self.get_sp_offset_rounded_to_16()),
            dst: asm::Operand::Reg(asm::Register::SP),
        };

        // Reset stack pointer offset for next function
        self.sp_offset = 0;
    }

    /// Replace pseudo-register operands in an instruction with stack addresses.
    fn replace_pseudo_reg(&mut self, instruction: &mut asm::Instruction) {
        match instruction {
            asm::Instruction::Mov { size, dst, src } => {
                self.to_stack(dst, *size);
                self.to_stack(src, *size);
            }

            asm::Instruction::Unary { size, op: _, dst } => {
                self.to_stack(dst, *size);
            }

            asm::Instruction::Binary {
                size,
                op: _,
                src,
                dst,
            } => {
                self.to_stack(src, *size);
                self.to_stack(dst, *size);
            }

            asm::Instruction::Idiv(size, src) => {
                self.to_stack(src, *size);
            }

            asm::Instruction::Cmp { size, src, dst } => {
                self.to_stack(src, *size);
                self.to_stack(dst, *size);
            }

            asm::Instruction::SetCC(_, src) => {
                self.to_stack(src, OperandSize::LongWord);
            }

            asm::Instruction::Push(src) => {
                self.to_stack(src, OperandSize::QuadWord);
            }

            asm::Instruction::Movsx { src, dst } => {
                self.to_stack(src, OperandSize::QuadWord);
                self.to_stack(dst, OperandSize::QuadWord);
            }

            // Instructions without pseudo-register operands are ignored
            _ => return,
        }
    }

    /// Convert a pseudo-register operand to a stack location if needed.
    fn to_stack(&mut self, operand: &mut asm::Operand, size: OperandSize) {
        if let asm::Operand::Pseudo(id) = operand {
            // Already mapped? Replace and return.
            if let Some(offset) = self.pseudo_reg_map.get(id) {
                *operand = asm::Operand::Stack(*offset);
                return;
            }

            // Determine if this is a static/global or needs a stack slot.
            let needs_stack = match self.asm_symbol_table.get(*id) {
                // if it's not a static variable, then return true
                AsmSymbolEntry::Obj { size: _, is_static } => !(*is_static),
                AsmSymbolEntry::Fun { .. } => false,
            };

            if needs_stack {
                self.allocate_stack(*id, operand, size);
            } else {
                *operand = asm::Operand::Data(*id);
            }
        }
    }

    fn allocate_stack(&mut self, iden: Identifier, operand: &mut asm::Operand, size: OperandSize) {
        match size {
            OperandSize::LongWord => {
                self.sp_offset -= 4;
                self.pseudo_reg_map.insert(iden, self.sp_offset);
                *operand = asm::Operand::Stack(self.sp_offset);
            }
            OperandSize::QuadWord => {
                let alloc_amount = if (-self.sp_offset) % 8 == 0 { 8 } else { 12 };
                self.sp_offset -= alloc_amount;
                self.pseudo_reg_map.insert(iden, self.sp_offset);
                *operand = asm::Operand::Stack(self.sp_offset);
            }
        }
    }
}
