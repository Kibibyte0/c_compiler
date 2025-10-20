use crate::IRgen;
use crate::tacky;
use parser::ast::ForInit;
use parser::ast::{self, Expression, Statement};
use shared_context::Identifier;

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
            ast::StatementType::Break(label) => self.gen_break_statement(label, instructions),
            ast::StatementType::Continue(label) => self.gen_continue_statement(label, instructions),
            ast::StatementType::DoWhile {
                condition,
                body,
                label,
            } => {
                self.gen_do_while_statement(condition, *body, label, instructions);
            }
            ast::StatementType::While {
                condition,
                body,
                label,
            } => {
                self.gen_while_statement(condition, *body, label, instructions);
            }
            ast::StatementType::For {
                init,
                condition,
                post,
                body,
                label,
            } => {
                self.gen_for_statement(init, condition, post, *body, label, instructions);
            }
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

    fn gen_break_statement(
        &mut self,
        label: Identifier,
        instructions: &mut Vec<tacky::Instruction>,
    ) {
        instructions.push(tacky::Instruction::Jump(self.convert_to_break_label(label)));
    }

    fn gen_continue_statement(
        &mut self,
        label: Identifier,
        instructions: &mut Vec<tacky::Instruction>,
    ) {
        instructions.push(tacky::Instruction::Jump(
            self.convert_to_continue_label(label),
        ));
    }

    fn gen_do_while_statement(
        &mut self,
        condition: Expression,
        body: Statement,
        label: Identifier,
        instructions: &mut Vec<tacky::Instruction>,
    ) {
        let start_label = self.make_label();
        let continue_label = self.convert_to_continue_label(label);
        let break_label = self.convert_to_break_label(label);

        instructions.push(tacky::Instruction::Label(start_label));
        self.gen_statements(body, instructions);
        instructions.push(tacky::Instruction::Label(continue_label));
        let value = self.gen_expression(condition, instructions);
        instructions.push(tacky::Instruction::JumpIfNotZero(value, start_label));
        instructions.push(tacky::Instruction::Label(break_label));
    }

    fn gen_while_statement(
        &mut self,
        condition: Expression,
        body: Statement,
        label: Identifier,
        instructions: &mut Vec<tacky::Instruction>,
    ) {
        let continue_label = self.convert_to_continue_label(label);
        let break_label = self.convert_to_break_label(label);

        instructions.push(tacky::Instruction::Label(continue_label));
        let value = self.gen_expression(condition, instructions);
        instructions.push(tacky::Instruction::JumpIfZero(value, break_label));
        self.gen_statements(body, instructions);
        instructions.push(tacky::Instruction::Jump(continue_label));
        instructions.push(tacky::Instruction::Label(break_label));
    }

    fn gen_for_statement(
        &mut self,
        init: ForInit,
        condition: Option<Expression>,
        post: Option<Expression>,
        body: Statement,
        label: Identifier,
        instructions: &mut Vec<tacky::Instruction>,
    ) {
        let start_label = self.make_label();
        let continue_label = self.convert_to_continue_label(label);
        let break_label = self.convert_to_break_label(label);

        self.gen_for_init(init, instructions);
        instructions.push(tacky::Instruction::Label(start_label));
        self.gen_for_statement_condition(condition, instructions, break_label);
        self.gen_statements(body, instructions);
        instructions.push(tacky::Instruction::Label(continue_label));
        self.gen_for_statement_post(post, instructions);
        instructions.push(tacky::Instruction::Jump(start_label));
        instructions.push(tacky::Instruction::Label(break_label));
    }

    fn gen_for_init(&mut self, init: ForInit, instructions: &mut Vec<tacky::Instruction>) {
        match init {
            ForInit::D(decl) => self.gen_declaration(decl, instructions),
            ForInit::E(optional_expr) => match optional_expr {
                Some(expr) => {
                    self.gen_expression(expr, instructions);
                }
                None => return,
            },
        }
    }

    fn gen_for_statement_condition(
        &mut self,
        optional_expr: Option<Expression>,
        instructions: &mut Vec<tacky::Instruction>,
        break_label: Identifier,
    ) {
        match optional_expr {
            Some(expr) => {
                let value = self.gen_expression(expr, instructions);
                instructions.push(tacky::Instruction::JumpIfZero(value, break_label));
            }
            None => return,
        }
    }

    fn gen_for_statement_post(
        &mut self,
        optional_expr: Option<Expression>,
        instructions: &mut Vec<tacky::Instruction>,
    ) {
        match optional_expr {
            Some(expr) => {
                self.gen_expression(expr, instructions);
            }
            None => return,
        }
    }
}
