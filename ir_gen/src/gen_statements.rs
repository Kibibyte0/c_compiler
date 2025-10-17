use crate::IRgen;
use crate::tacky;
use parser::ast::{self, Expression, Statement};

impl<'a, 'b> IRgen<'a, 'b> {
    pub(crate) fn gen_statements(
        &mut self,
        stmt: ast::Statement,
        instructions: &mut Vec<tacky::Instruction>,
    ) {
        let (stmt_type, _) = stmt.into_parts();
        match stmt_type {
            ast::StatementType::Return(expr) => {
                let val = self.gen_expression(expr, instructions);
                let instr = tacky::Instruction::Ret(val);
                instructions.push(instr);
            }
            ast::StatementType::ExprStatement(expr) => {
                self.gen_expression(expr, instructions);
            }
            ast::StatementType::Null => return,
            ast::StatementType::Compound(block) => self.gen_block(block, instructions),
            ast::StatementType::IfStatement {
                condition,
                if_clause,
                else_clause,
            } => self.gen_if_statement(condition, *if_clause, else_clause, instructions),
        }
    }

    fn gen_if_statement(
        &mut self,
        condition: Expression,
        if_clause: Statement,
        else_clause: Option<Box<Statement>>,
        instructions: &mut Vec<tacky::Instruction>,
    ) {
        match else_clause {
            Some(clause) => {
                self.gen_if_statement_with_else_clause(condition, if_clause, *clause, instructions)
            }
            None => self.gen_if_statement_without_else_clause(condition, if_clause, instructions),
        }
    }

    fn gen_if_statement_with_else_clause(
        &mut self,
        condition: Expression,
        if_clause: Statement,
        else_clause: Statement,
        instructions: &mut Vec<tacky::Instruction>,
    ) {
        let else_label = self.make_label();
        let end_label = self.make_label();

        let cond_result = self.gen_expression(condition, instructions);
        instructions.push(tacky::Instruction::JumpIfZero(cond_result, else_label));

        self.gen_statements(if_clause, instructions);

        instructions.push(tacky::Instruction::Jump(end_label));

        instructions.push(tacky::Instruction::Label(else_label));

        self.gen_statements(else_clause, instructions);

        instructions.push(tacky::Instruction::Label(end_label));
    }

    fn gen_if_statement_without_else_clause(
        &mut self,
        condition: Expression,
        if_clause: Statement,
        instructions: &mut Vec<tacky::Instruction>,
    ) {
        let end_label = self.make_label();

        let cond_result = self.gen_expression(condition, instructions);
        instructions.push(tacky::Instruction::JumpIfZero(cond_result, end_label));

        self.gen_statements(if_clause, instructions);

        instructions.push(tacky::Instruction::Label(end_label));
    }
}
