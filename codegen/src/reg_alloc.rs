use crate::ASMgen;
use crate::asm;

impl ASMgen {
    // alocate real memory addresses for all the pseudo registers
    pub fn allocate_registers(&mut self, program: &mut asm::Program) {
        let function = program.get_mut_function();
        let instructions = function.get_mut_instructions();

        for instruction in instructions {
            self.replace_pseudo_reg(instruction);
        }

        // put the real allocation space in the placeholder at the start of the vector
        // reset the stack pointer offset to zero after allocating addresses for the current function.
        function.get_mut_instructions()[0] = asm::Instruction::AllocateStack(-self.sp_offest);
        self.sp_offest = 0;
    }

    // takes an instruction, and if it have any pseudo registers in the operands
    // it replace them with stack addresses, otherwise nothing happens
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

            // the rest of the instruction does not have operands
            _ => return,
        }
    }

    // takes an operand, and if it's a pseudo register, it gets replaced by a stack address
    fn to_stack(&mut self, operand: &mut asm::Operand) {
        match operand {
            asm::Operand::Pseudo(id) => {
                if let Some(int) = self.pseudo_reg_map.get(id) {
                    *operand = asm::Operand::Stack(*int);
                } else {
                    self.sp_offest -= 4;
                    self.pseudo_reg_map.insert(id.clone(), self.sp_offest);
                    *operand = asm::Operand::Stack(self.sp_offest);
                }
            }

            _ => return,
        }
    }
}
