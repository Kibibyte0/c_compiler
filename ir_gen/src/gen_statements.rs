use crate::IRgen;
use crate::tacky;
use parser::ast::{self, Expression, Spanned, Statement};

impl IRgen {
    pub(crate) fn gen_statements(
        &mut self,
        sp_stmt: Spanned<ast::Statement>,
        instructions: &mut Vec<tacky::Instruction>,
    ) {
        match sp_stmt.discard_sp() {
            ast::Statement::Return(sp_exp) => {
                let val = self.gen_expression(sp_exp, instructions);
                let instr = tacky::Instruction::Ret(val);
                instructions.push(instr);
            }
            ast::Statement::ExprStatement(sp_exp) => {
                self.gen_expression(sp_exp, instructions);
            }
            ast::Statement::Null => return,
            ast::Statement::Compound(sp_block) => self.gen_block(sp_block, instructions),
            ast::Statement::IfStatement {
                condition,
                if_clause,
                else_clause,
            } => self.gen_if_statement(condition, *if_clause, else_clause, instructions),
        }
    }

    fn gen_if_statement(
        &mut self,
        condition: Spanned<Expression>,
        if_clause: Spanned<Statement>,
        else_clause: Option<Box<Spanned<Statement>>>,
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
        condition: Spanned<Expression>,
        if_clause: Spanned<Statement>,
        else_clause: Spanned<Statement>,
        instructions: &mut Vec<tacky::Instruction>,
    ) {
        let else_label = self.make_label();
        let end_label = self.make_label();

        let cond_result = self.gen_expression(condition, instructions);
        instructions.push(tacky::Instruction::JumpIfZero(
            cond_result,
            else_label.clone(),
        ));

        self.gen_statements(if_clause, instructions);

        instructions.push(tacky::Instruction::Jump(end_label.clone()));

        instructions.push(tacky::Instruction::Label(else_label));

        self.gen_statements(else_clause, instructions);

        instructions.push(tacky::Instruction::Label(end_label));
    }

    fn gen_if_statement_without_else_clause(
        &mut self,
        condition: Spanned<Expression>,
        if_clause: Spanned<Statement>,
        instructions: &mut Vec<tacky::Instruction>,
    ) {
        let end_label = self.make_label();

        let cond_result = self.gen_expression(condition, instructions);
        instructions.push(tacky::Instruction::JumpIfZero(
            cond_result,
            end_label.clone(),
        ));

        self.gen_statements(if_clause, instructions);

        instructions.push(tacky::Instruction::Label(end_label));
    }
}
