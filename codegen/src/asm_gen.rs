use crate::AsmGen;
use crate::{asm, asm::Operand, asm::Operand::Reg, asm::Register};
use ir_gen::tacky;

mod gen_control_flow;
mod gen_operations;

impl AsmGen {
    /// consume the tacky tree and produce an asm tree
    pub fn gen_asm(program: tacky::Program) -> asm::Program {
        let function = program.into_parts();
        asm::Program::new(Self::gen_function_def(function))
    }

    fn gen_function_def(function: tacky::FunctionDef) -> asm::FunctionDef {
        let (name, tacky_instructions) = function.into_parts();
        let mut asm_instructions = Vec::new();

        // this will work as a placeholder for the actual stack allocation instruction
        // in the register allocatation pass, it will get replaced by the real value
        asm_instructions.push(asm::Instruction::AllocateStack(0));

        Self::gen_instructions(tacky_instructions, &mut asm_instructions);

        asm::FunctionDef::new(name, asm_instructions)
    }

    /// takes a tacky instruction and return a vector of the corresponding asm instructions
    fn gen_instructions(
        tacky_instructions: Vec<tacky::Instruction>,
        asm_instructions: &mut Vec<asm::Instruction>,
    ) {
        for tacky_instruction in tacky_instructions {
            match tacky_instruction {
                tacky::Instruction::Ret(val) => Self::handle_ret(val, asm_instructions),
                tacky::Instruction::Unary { op, src, dst } => {
                    Self::handle_unary(op, src, dst, asm_instructions)
                }
                tacky::Instruction::Binary {
                    op,
                    src1,
                    src2,
                    dst,
                } => Self::handle_binary(op, src1, src2, dst, asm_instructions),

                tacky::Instruction::Jump(tar) => Self::handle_jump(tar, asm_instructions),
                tacky::Instruction::JumpIfZero(pred, tar) => {
                    Self::handle_jump_if_zero(pred, tar, asm_instructions)
                }
                tacky::Instruction::JumpIfNotZero(pred, tar) => {
                    Self::handle_jump_if_not_zero(pred, tar, asm_instructions)
                }
                tacky::Instruction::Label(tar) => Self::handle_label(tar, asm_instructions),

                tacky::Instruction::Copy { src, dst } => {
                    Self::handle_copy(src, dst, asm_instructions)
                }
            }
        }
    }

    fn handle_ret(val: tacky::Value, asm_instructions: &mut Vec<asm::Instruction>) {
        asm_instructions.push(asm::Instruction::Mov {
            dst: Reg(Register::AX),
            src: Self::convert_val(val),
        });
        asm_instructions.push(asm::Instruction::Ret);
    }

    fn handle_copy(
        src: tacky::Value,
        dst: tacky::Value,
        asm_instructions: &mut Vec<asm::Instruction>,
    ) {
        asm_instructions.push(asm::Instruction::Mov {
            src: Self::convert_val(src),
            dst: Self::convert_val(dst),
        });
    }

    fn convert_val(val: tacky::Value) -> Operand {
        match val {
            tacky::Value::Var(identifier) => Operand::Pseudo(identifier),

            tacky::Value::Constant(int) => Operand::Immediate(int),
        }
    }
}
