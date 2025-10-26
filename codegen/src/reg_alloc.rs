use crate::RegisterAllocation;
use crate::asm;
use std::collections::HashMap;

impl RegisterAllocation {
    /// Create a new RegisterAllocation instance
    pub fn new() -> Self {
        Self {
            pseudo_reg_map: HashMap::new(), // Maps pseudo-register IDs to stack offsets
            sp_offset: 0,                   // Tracks the current stack offset
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
        let functions = program.get_mut_functions();
        for function in functions {
            self.handle_function(function);
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
        match operand {
            asm::Operand::Pseudo(id) => {
                if let Some(int) = self.pseudo_reg_map.get(id) {
                    // Already allocated -> replace with stack offset
                    *operand = asm::Operand::Stack(*int);
                } else {
                    // New pseudo register -> assign next stack slot
                    self.sp_offset -= 4; // allocate 4 bytes
                    self.pseudo_reg_map.insert(*id, self.sp_offset);
                    *operand = asm::Operand::Stack(self.sp_offset);
                }
            }

            _ => return, // Real registers or immediate values remain unchanged
        }
    }
}
