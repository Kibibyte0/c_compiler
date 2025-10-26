// Statement lowering for the IR generator.
//
// This module defines how high-level AST statements are translated
// into low-level Tacky (three address code) IR instructions.
// Each statement type — control flow, loops, returns, etc. — is lowered
// into explicit labels, jumps, and simple operations suitable for later
// optimization or code generation.

use crate::IRgen;
use crate::tacky;
use parser::ast::{self, Expression, ForInit, Statement};
use shared_context::Identifier;

impl<'src, 'ctx> IRgen<'src, 'ctx> {
    /// Lowers a single statement from the AST into a list of IR instructions.
    ///
    /// Each ast::StatementType variant is matched and expanded into its
    /// equivalent tacky::Instruction sequence. Nested statements and blocks
    /// are processed recursively.
    pub(crate) fn gen_statements(
        &mut self,
        stmt: ast::Statement,
        instructions: &mut Vec<tacky::Instruction>,
    ) {
        let (stmt_type, _) = stmt.into_parts();

        match stmt_type {
            // Return statement: evaluate expression and emit `Ret`.
            ast::StatementType::Return(expr) => {
                let val = self.gen_expression(expr, instructions);
                instructions.push(tacky::Instruction::Ret(val));
            }

            // Expression statement: evaluate expression for side effects.
            ast::StatementType::ExprStatement(expr) => {
                self.gen_expression(expr, instructions);
            }

            // Empty / null statement: ignore.
            ast::StatementType::Null => return,

            // Compound statement (block): recursively lower contained items.
            ast::StatementType::Compound(block) => self.gen_block(block, instructions),

            // Conditional branching
            ast::StatementType::IfStatement {
                condition,
                if_clause,
                else_clause,
            } => self.gen_if_statement(condition, *if_clause, else_clause, instructions),

            // Break and continue
            ast::StatementType::Break(label) => self.gen_break_statement(label, instructions),
            ast::StatementType::Continue(label) => self.gen_continue_statement(label, instructions),

            // Loop constructs
            ast::StatementType::DoWhile {
                condition,
                body,
                label,
            } => self.gen_do_while_statement(condition, *body, label, instructions),

            ast::StatementType::While {
                condition,
                body,
                label,
            } => self.gen_while_statement(condition, *body, label, instructions),

            ast::StatementType::For {
                init,
                condition,
                post,
                body,
                label,
            } => self.gen_for_statement(init, condition, post, *body, label, instructions),
        }
    }

    /// Lowers an `if` or `if-else` statement.
    ///
    /// This function dispatches to either the `with_else_clause` or
    /// `without_else_clause` version, depending on the presence of `else_clause`.
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

    /// Lowers an `if-else` statement into IR.
    ///
    /// ```
    /// if (cond) if_clause else else_clause
    /// ```
    /// becomes:
    /// ```
    ///   cond_val = <expr>
    ///   jump_if_zero cond_val, else_label
    ///   <if_clause>
    ///   jump end_label
    /// else_label:
    ///   <else_clause>
    /// end_label:
    /// ```
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

    /// Lowers an `if` statement without an `else` clause.
    ///
    /// ```
    /// if (cond) body
    /// ```
    /// becomes:
    /// ```
    ///   cond_val = <expr>
    ///   jump_if_zero cond_val, end_label
    ///   <body>
    /// end_label:
    /// ```
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

    /// Emits a jump to the loop’s `break` label.
    fn gen_break_statement(
        &mut self,
        label: Identifier,
        instructions: &mut Vec<tacky::Instruction>,
    ) {
        instructions.push(tacky::Instruction::Jump(self.convert_to_break_label(label)));
    }

    /// Emits a jump to the loop’s `continue` label.
    fn gen_continue_statement(
        &mut self,
        label: Identifier,
        instructions: &mut Vec<tacky::Instruction>,
    ) {
        instructions.push(tacky::Instruction::Jump(
            self.convert_to_continue_label(label),
        ));
    }

    /// Lowers a `do-while` loop.
    ///
    /// ```
    /// do { body } while (cond);
    /// ```
    /// becomes:
    /// ```
    /// start_label:
    ///   <body>
    /// continue_label:
    ///   cond_val = <expr>
    ///   jump_if_not_zero cond_val, start_label
    /// break_label:
    /// ```
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

    /// Lowers a `while` loop.
    ///
    /// ```
    /// while (cond) { body }
    /// ```
    /// becomes:
    /// ```
    /// continue_label:
    ///   cond_val = <expr>
    ///   jump_if_zero cond_val, break_label
    ///   <body>
    ///   jump continue_label
    /// break_label:
    /// ```
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

    /// Lowers a `for` loop.
    ///
    /// ```
    /// for (init; cond; post) { body }
    /// ```
    /// becomes:
    /// ```
    ///   <init>
    /// start_label:
    ///   <cond>?
    ///   jump_if_zero cond_val, break_label
    ///   <body>
    /// continue_label:
    ///   <post>
    ///   jump start_label
    /// break_label:
    /// ```
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

    /// Generates the initialization part of a `for` loop.
    fn gen_for_init(&mut self, init: ForInit, instructions: &mut Vec<tacky::Instruction>) {
        match init {
            ForInit::D(var_decl) => self.gen_variable_declaration(var_decl, instructions),
            ForInit::E(optional_expr) => {
                if let Some(expr) = optional_expr {
                    self.gen_expression(expr, instructions);
                }
            }
        }
    }

    /// Generates the conditional check for a `for` loop.
    fn gen_for_statement_condition(
        &mut self,
        optional_expr: Option<Expression>,
        instructions: &mut Vec<tacky::Instruction>,
        break_label: Identifier,
    ) {
        if let Some(expr) = optional_expr {
            let value = self.gen_expression(expr, instructions);
            instructions.push(tacky::Instruction::JumpIfZero(value, break_label));
        }
    }

    /// Generates the post-expression of a `for` loop.
    fn gen_for_statement_post(
        &mut self,
        optional_expr: Option<Expression>,
        instructions: &mut Vec<tacky::Instruction>,
    ) {
        if let Some(expr) = optional_expr {
            self.gen_expression(expr, instructions);
        }
    }
}
