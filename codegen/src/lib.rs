use ir_gen::tacky;
use std::collections::HashMap;

use crate::asm::{
    Operand::{self, Reg},
    Register,
};

pub mod asm;
mod fix_instructions;
mod reg_alloc;

pub struct ASMgen {
    pseudo_reg_map: HashMap<asm::Identifier, i32>,
    sp_offest: i32,
}

impl ASMgen {
    pub fn new() -> Self {
        Self {
            pseudo_reg_map: HashMap::new(),
            sp_offest: 0,
        }
    }

    pub fn gen_asm(&self, program: tacky::Program) -> asm::Program {
        let function = program.into_parts();
        asm::Program::new(self.gen_function_def(function))
    }

    fn gen_function_def(&self, function: tacky::FunctionDef) -> asm::FunctionDef {
        let (name, tacky_instructions) = function.into_parts();
        let mut asm_instructions: Vec<asm::Instruction> = Vec::new();

        // this will work as a placeholder for the actual allocation
        // in the register allocatation pass
        asm_instructions.push(asm::Instruction::AllocateStack(0));

        self.gen_instructions(tacky_instructions, &mut asm_instructions);

        asm::FunctionDef::new(asm::Identifier(name.0), asm_instructions)
    }

    fn gen_instructions(
        &self,
        tacky_instructions: Vec<tacky::Instruction>,
        asm_instructions: &mut Vec<asm::Instruction>,
    ) {
        for tacky_instruction in tacky_instructions {
            match tacky_instruction {
                tacky::Instruction::Ret(val) => {
                    self.gen_ret_instructions(val, asm_instructions);
                }

                tacky::Instruction::Unary { op, src, dst } => {
                    self.gen_unary_instructions(op, src, dst, asm_instructions);
                }

                tacky::Instruction::Binary {
                    op,
                    src1,
                    src2,
                    dst,
                } => {
                    self.gen_binary_instructions(op, src1, src2, dst, asm_instructions);
                }
            }
        }
    }

    fn gen_ret_instructions(
        &self,
        val: tacky::Value,
        asm_instructions: &mut Vec<asm::Instruction>,
    ) {
        asm_instructions.push(asm::Instruction::Mov {
            dst: Reg(Register::AX),
            src: ASMgen::convert_val(&val),
        });
        asm_instructions.push(asm::Instruction::Ret);
    }

    fn gen_binary_instructions(
        &self,
        op: tacky::BinaryOP,
        src1: tacky::Value,
        src2: tacky::Value,
        dst: tacky::Value,
        asm_instructions: &mut Vec<asm::Instruction>,
    ) {
        match op {
            tacky::BinaryOP::Add | tacky::BinaryOP::Sub | tacky::BinaryOP::Mul => {
                asm_instructions.push(asm::Instruction::Mov {
                    dst: ASMgen::convert_val(&dst),
                    src: ASMgen::convert_val(&src1),
                });
                asm_instructions.push(asm::Instruction::Binary {
                    op: ASMgen::convert_binary_op(op),
                    src: ASMgen::convert_val(&src2),
                    dst: ASMgen::convert_val(&dst),
                });
            }
            tacky::BinaryOP::Div | tacky::BinaryOP::Mod => {
                self.gen_div_mod_instructions(op, src1, src2, dst, asm_instructions);
            }
        }
    }

    fn gen_div_mod_instructions(
        &self,
        op: tacky::BinaryOP,
        src1: tacky::Value,
        src2: tacky::Value,
        dst: tacky::Value,
        asm_instructions: &mut Vec<asm::Instruction>,
    ) {
        asm_instructions.push(asm::Instruction::Mov {
            src: ASMgen::convert_val(&src1),
            dst: Reg(Register::AX),
        });
        asm_instructions.push(asm::Instruction::Cdq);
        asm_instructions.push(asm::Instruction::Idiv(ASMgen::convert_val(&src2)));

        let ret_reg = match op {
            tacky::BinaryOP::Mod => Reg(Register::DX),
            _ => Reg(Register::AX),
        };

        asm_instructions.push(asm::Instruction::Mov {
            src: ret_reg,
            dst: ASMgen::convert_val(&dst),
        });
    }

    fn gen_unary_instructions(
        &self,
        op: tacky::UnaryOP,
        src: tacky::Value,
        dst: tacky::Value,
        asm_instructions: &mut Vec<asm::Instruction>,
    ) {
        asm_instructions.push(asm::Instruction::Mov {
            dst: ASMgen::convert_val(&dst),
            src: ASMgen::convert_val(&src),
        });
        asm_instructions.push(asm::Instruction::Unary {
            op: ASMgen::convert_unary_op(op),
            dst: ASMgen::convert_val(&dst),
        });
    }

    fn convert_val(val: &tacky::Value) -> Operand {
        match val {
            tacky::Value::Var(identifier) => Operand::Pseudo(asm::Identifier(identifier.0.clone())),

            tacky::Value::Constant(int) => Operand::Immediate(*int),
        }
    }

    fn convert_binary_op(op: tacky::BinaryOP) -> asm::BinaryOP {
        match op {
            tacky::BinaryOP::Add => asm::BinaryOP::Add,
            tacky::BinaryOP::Sub => asm::BinaryOP::Sub,
            tacky::BinaryOP::Mul => asm::BinaryOP::Mul,
            // there are more tacky BinaryOP variant than asm BinaryOP
            // this arm will never be reached so it have some arbitrary value
            _ => asm::BinaryOP::Add,
        }
    }

    fn convert_unary_op(op: tacky::UnaryOP) -> asm::UnaryOP {
        match op {
            tacky::UnaryOP::Not => asm::UnaryOP::Not,
            tacky::UnaryOP::Neg => asm::UnaryOP::Neg,
        }
    }
}
