mod tacky;
use parser::ast;

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

    pub fn emit_tacky<'source>(
        &mut self,
        program: ast::Program<'source>,
    ) -> tacky::Program<'source> {
        let function = program.into_parts();
        tacky::Program::new(self.emit_function_def(function))
    }

    fn emit_function_def<'source> (
        &mut self,
        function: ast::FunctionDef<'source>,
    ) -> tacky::FunctionDef<'source> {
        let (name, body) = function.into_parts();
        let mut instructions: Vec<tacky::Instruction> = Vec::new();
        self.emit_statements(body, &mut instructions);

        tacky::FunctionDef::new(tacky::Identifier(name.0), instructions)
    }

    fn emit_statements(
        &mut self,
        statement: ast::Statement,
        instructions: &mut Vec<tacky::Instruction>,
    ) {
        match statement {
            ast::Statement::Return(exp) => {
                let value = tacky::Instruction::Ret(self.emit_expression(exp, instructions));
                instructions.push(value);
            }
        }
    }

    fn emit_expression(
        &mut self,
        exp: ast::Expression,
        instructions: &mut Vec<tacky::Instruction>,
    ) -> tacky::Value {
        match exp {
            ast::Expression::Constant(int) => tacky::Value::Constant(int),

            ast::Expression::Unary(op, inner) => {
                let src = self.emit_expression(*inner, instructions);
                let dst_name = self.make_temp_var();
                let dst = tacky::Value::Var(dst_name.clone());
                let tacky_op = IRgen::convert_unary_op(op);
                instructions.push(tacky::Instruction::Unary {
                    op: tacky_op,
                    dst,
                    src,
                });
                tacky::Value::Var(dst_name)
            }
        }
    }

    fn convert_unary_op(op: ast::UnaryOP) -> tacky::UnaryOP {
        match op {
            ast::UnaryOP::BitwiseComplement => tacky::UnaryOP::BitwiseComplement,
            ast::UnaryOP::Negation => tacky::UnaryOP::Negation,
        }
    }
}
