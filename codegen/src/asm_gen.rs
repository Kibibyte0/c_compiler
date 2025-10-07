use crate::AsmGen;
use crate::asm::Instruction;
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
        let mut function_instructions = Vec::new();

        // this will work as a placeholder for the actual stack allocation instruction
        // in the register allocatation pass, it will get replaced by the real value
        function_instructions.push(asm::Instruction::AllocateStack(0));

        // collect all the asm vectors in function_instructions
        for instruction in tacky_instructions {
            let mut new_instructions = Self::gen_instructions(instruction);
            function_instructions.append(&mut new_instructions);
        }

        asm::FunctionDef::new(asm::Identifier(name.0), function_instructions)
    }

    /// takes a tacky instruction and return a vector of the corresponding asm instructions
    fn gen_instructions(tacky_instruction: tacky::Instruction) -> Vec<asm::Instruction> {
        match tacky_instruction {
            tacky::Instruction::Ret(val) => Self::handle_ret(val),
            tacky::Instruction::Unary { op, src, dst } => Self::handle_unary(op, src, dst),
            tacky::Instruction::Binary {
                op,
                src1,
                src2,
                dst,
            } => Self::handle_binary(op, src1, src2, dst),

            tacky::Instruction::Jump(tar) => Self::handle_jump(tar),
            tacky::Instruction::JumpIfZero(pred, tar) => Self::handle_jump_if_zero(pred, tar),
            tacky::Instruction::JumpIfNotZero(pred, tar) => {
                Self::handle_jump_if_not_zero(pred, tar)
            }
            tacky::Instruction::Label(tar) => Self::handle_label(tar),

            tacky::Instruction::Copy { src, dst } => Self::handle_copy(src, dst),
        }
    }

    fn handle_ret(val: tacky::Value) -> Vec<asm::Instruction> {
        let mut new_instructions = Vec::new();
        new_instructions.push(asm::Instruction::Mov {
            dst: Reg(Register::AX),
            src: Self::convert_val(&val),
        });
        new_instructions.push(asm::Instruction::Ret);

        new_instructions
    }

    fn handle_copy(src: tacky::Value, dst: tacky::Value) -> Vec<Instruction> {
        let mut new_instructions = Vec::new();

        new_instructions.push(asm::Instruction::Mov {
            src: Self::convert_val(&src),
            dst: Self::convert_val(&dst),
        });

        new_instructions
    }

    fn convert_val(val: &tacky::Value) -> Operand {
        match val {
            tacky::Value::Var(identifier) => Operand::Pseudo(asm::Identifier(identifier.0.clone())),

            tacky::Value::Constant(int) => Operand::Immediate(*int),
        }
    }
}
