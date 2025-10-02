pub mod tacky;
use parser::ast;

mod gen_expressions;

pub struct IRgen {
    var_count: usize, // counter to generate automatic variables
}

impl IRgen {
    pub fn new() -> Self {
        Self { var_count: 0 }
    }

    // generate temporary variables
    fn make_temp_var(&mut self) -> String {
        let s = format!("tmp.{}", self.var_count);
        self.var_count += 1;
        s
    }

    pub fn gen_tacky(&mut self, program: ast::Program) -> tacky::Program {
        let function = program.into_parts();
        tacky::Program::new(self.gen_function_def(function))
    }

    fn gen_function_def(&mut self, function: ast::FunctionDef) -> tacky::FunctionDef {
        let (name, body) = function.into_parts();
        let mut instructions: Vec<tacky::Instruction> = Vec::new();
        self.gen_statements(body, &mut instructions);

        tacky::FunctionDef::new(tacky::Identifier(name.0), instructions)
    }

    fn gen_statements(
        &mut self,
        statement: ast::Statement,
        instructions: &mut Vec<tacky::Instruction>,
    ) {
        match statement {
            ast::Statement::Return(exp) => {
                let value = tacky::Instruction::Ret(self.gen_expression(exp, instructions));
                instructions.push(value);
            }
        }
    }
}
