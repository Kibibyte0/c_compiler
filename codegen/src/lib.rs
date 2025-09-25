use parser::ast::{Expression, FunctionDefinition, Program, Statement};

use crate::asm_ast::{AsmFunctionDefinition, AsmProgram, Instruction, Operand};

pub mod asm_ast;
pub struct Codegen<'source> {
    program: Program<'source>,
}

impl<'source> Codegen<'source> {
    pub fn new(program: Program<'source>) -> Self {
        Self {
            program
        }
    }

    pub fn gen_program(&'source self) -> AsmProgram<'source> {
        AsmProgram {
            function: Codegen::gen_function(&self.program.function),
        }
    }

    fn gen_function(function: &'source FunctionDefinition) -> AsmFunctionDefinition<'source> {
        AsmFunctionDefinition {
            name: &function.name.0,
            instructions:  Codegen::gen_body(&function.body),
        }
    }

    fn gen_body(statement: &Statement) -> Vec<Instruction> {
        let mut instructions: Vec<Instruction> = Vec::new();
        match statement {
            Statement::Return(exp) =>
            Codegen::gen_return_statement(exp, &mut instructions),
        }
        instructions
    }

    fn gen_return_statement(exp: &Expression, instructions: &mut Vec<Instruction>) {
        let int = match exp {
            Expression::Constant(n) => n,
        };
        instructions.push(
            Instruction::Mov(Operand::Immediate(*int), Operand::Register)
        );
        instructions.push(Instruction::Ret);
    }
}