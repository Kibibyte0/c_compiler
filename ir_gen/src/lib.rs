use std::cell::RefCell;

mod tacky;
use parser::ast;

thread_local! {
    static TEMP_VAR_COUNTER: RefCell<usize> = RefCell::new(0);
}

pub struct IRgen<'source> {
    program: ast::Program<'source>,
}

impl<'source> IRgen<'source> {
    pub fn new(program: ast::Program<'source>) -> Self {
        Self { program }
    }

    // generate temporary variables
    fn make_temp_var() -> String {
        TEMP_VAR_COUNTER.with(|counter| {
            let id = *counter.borrow();
            *counter.borrow_mut() += 1;
            format!("tmp.{}", id)
        })
    }

    pub fn emit_tacky(&self) -> tacky::Program<'source> {
        let function = self.program.get_function();
        tacky::Program::new(self.emit_function_def(function))
    }

    fn emit_function_def(
        &self,
        function: &ast::FunctionDef<'source>,
    ) -> tacky::FunctionDef<'source> {
        let name = function.get_name();
        let mut instructions: Vec<tacky::Instruction> = Vec::new();
        self.emit_statements(function.get_body(), &mut instructions);

        tacky::FunctionDef::new(name, instructions)
    }

    fn emit_statements(
        &self,
        statement: &ast::Statement,
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
        &self,
        exp: &ast::Expression,
        instructions: &mut Vec<tacky::Instruction>
    ) -> tacky::Value {
        match exp {
            ast::Expression::Constant(int) => {
                tacky::Value::Constant(*int)
            }

            ast::Expression::Unary(op, inner) => {
                let src = self.emit_expression(inner, instructions);
                let dst_name = IRgen::make_temp_var();
                let dst = tacky::Value::Var(dst_name.clone());
                let tacky_op = self.convert_unary_op(op);
                instructions.push(tacky::Instruction::Unary { op: tacky_op, dst, src });
                tacky::Value::Var(dst_name)
            }
        }
    }

    fn convert_unary_op(&self, op: &ast::UnaryOP) -> tacky::UnaryOP {
        match op {
            ast::UnaryOP::BitwiseComplement => tacky::UnaryOP::BitwiseComplement,
            ast::UnaryOP::Negation => tacky::UnaryOP::Negation,
        }
    }
}
