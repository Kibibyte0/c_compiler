use ir_gen::tacky;
use std::collections::HashMap;

mod asm_ast;
mod reg_alloc;

pub struct ASMgen {
    pseudo_reg_map: HashMap<asm_ast::Identifier, i32>,
    sp_offest: i32,
}

impl ASMgen {
    pub fn new() -> Self {
        Self {
            pseudo_reg_map: HashMap::new(),
            sp_offest: 0,
        }
    }

    pub fn emit_asm(&self, program: tacky::Program) -> asm_ast::Program {
        let function = program.into_parts();
        asm_ast::Program::new(self.emit_function_def(function))
    }

    fn emit_function_def(&self, function: tacky::FunctionDef) -> asm_ast::FunctionDef {
        let (name, tacky_instructions) = function.into_parts();
        let mut asm_instructions: Vec<asm_ast::Instruction> = Vec::new();
        self.emit_instructions(tacky_instructions, &mut asm_instructions);

        asm_ast::FunctionDef::new(asm_ast::Identifier(name.0), asm_instructions)
    }

    fn emit_instructions(
        &self,
        tacky_instructions: Vec<tacky::Instruction>,
        asm_instructions: &mut Vec<asm_ast::Instruction>,
    ) {
        for tacky_instruction in tacky_instructions {
            match tacky_instruction {
                tacky::Instruction::Ret(val) => {
                    self.emit_ret_instructions(val, asm_instructions);
                }

                tacky::Instruction::Unary { op, dst, src } => {
                    self.emit_unary_instructions(op, dst, src, asm_instructions);
                }
            }
        }
    }

    fn emit_ret_instructions(
        &self,
        val: tacky::Value,
        asm_instructions: &mut Vec<asm_ast::Instruction>,
    ) {
        asm_instructions.push(asm_ast::Instruction::Mov {
            dst: asm_ast::Operand::Reg(asm_ast::Register::AX),
            src: self.convert_val(&val),
        });
        asm_instructions.push(asm_ast::Instruction::Ret);
    }

    fn emit_unary_instructions(
        &self,
        op: tacky::UnaryOP,
        dst: tacky::Value,
        src: tacky::Value,
        asm_instructions: &mut Vec<asm_ast::Instruction>,
    ) {
        asm_instructions.push(asm_ast::Instruction::Mov {
            dst: self.convert_val(&dst),
            src: self.convert_val(&src),
        });
        asm_instructions.push(asm_ast::Instruction::Unary {
            operator: self.convert_unary_op(op),
            operand: self.convert_val(&dst),
        });
    }

    fn convert_val(&self, val: &tacky::Value) -> asm_ast::Operand {
        match val {
            tacky::Value::Var(identifier) => {
                asm_ast::Operand::Pseudo(asm_ast::Identifier(identifier.0.clone()))
            }

            tacky::Value::Constant(int) => asm_ast::Operand::Immediate(*int),
        }
    }

    fn convert_unary_op(&self, op: tacky::UnaryOP) -> asm_ast::UnaryOP {
        match op {
            tacky::UnaryOP::BitwiseComplement => asm_ast::UnaryOP::Not,
            tacky::UnaryOP::Negation => asm_ast::UnaryOP::Neg,
        }
    }
}
