use super::ASMgen;
use super::asm_ast;

impl ASMgen {
    pub fn allocate_registers(&mut self, program: &mut asm_ast::Program) {
        let function = program.get_function();
        let instructions = function.get_instructions();

        for instruction in instructions {
            self.replace_pseudo_reg(instruction);
        }
    }

    fn replace_pseudo_reg(&mut self, instruction: &mut asm_ast::Instruction) {
        match instruction {
            asm_ast::Instruction::Mov { dst, src } => {
                self.to_stack(dst);
                self.to_stack(src);
            }

            asm_ast::Instruction::Unary { operator:_, operand } => {
                self.to_stack(operand);
            }

            _ => return
        }
    }

    fn to_stack(&mut self, op: &mut asm_ast::Operand) {
        match op {
            asm_ast::Operand::Pseudo(id) => {
                if let Some(int) = self.pseudo_reg_map.get(id) {
                    *op = asm_ast::Operand::Stack(*int);
                } else {
                    self.sp_offest -= 4;
                    self.pseudo_reg_map.insert(id.clone(), self.sp_offest);
                    *op = asm_ast::Operand::Stack(self.sp_offest);
                }
            }

            _ => return
        }
    }
}
