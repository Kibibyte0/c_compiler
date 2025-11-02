use crate::ast::*;
use shared_context::interner::Interner;
use shared_context::{Identifier, SpannedIdentifier};

/// DebugTreePrinter traverses the AST and prints a readable, indented
/// tree representation of program structures (functions, statements, expressions).
pub struct DebugTreePrinter<'a> {
    interner: &'a Interner<'a>, // used to resolve identifiers to their string names
}

impl<'a> DebugTreePrinter<'a> {
    /// Creates a new DebugTreePrinter with a reference to an Interner.
    pub fn new(interner: &'a Interner) -> Self {
        Self { interner }
    }

    /// Prints the entire program
    pub fn print(&self, program: Program) {
        for decl in program.into_parts() {
            self.print_declaration(decl, 0);
        }
    }

    /// Generates indentation string based on depth level
    fn indent(&self, level: usize) -> String {
        "  ".repeat(level)
    }

    /// Prints a function declaration with its parameters and body
    fn print_function(&self, function: FunctionDecl, level: usize) {
        let (name, params, body, storage_class, _) = function.into_parts();
        println!(
            "{}FunctionDecl {:?} \"{}\"",
            self.indent(level),
            storage_class,
            self.format_spanned_identifier(name)
        );

        // Print function parameters
        println!("{}Params", self.indent(level + 1));
        for param in params {
            println!(
                "{}Param \"{}\"",
                self.indent(level + 2),
                self.format_spanned_identifier(param)
            );
        }

        // Print function body if present
        if let Some(block) = body {
            self.print_block(block, level + 1);
        } else {
            println!("{}<No Body>", self.indent(level + 1));
        }
    }

    /// Prints a block of statements or declarations
    fn print_block(&self, block: Block, level: usize) {
        let (items, _) = block.into_parts();
        println!("{}Block", self.indent(level));
        for item in items {
            self.print_block_item(item, level + 1);
        }
    }

    /// Prints either a declaration or statement inside a block
    fn print_block_item(&self, item: BlockItem, level: usize) {
        match item {
            BlockItem::D(decl) => self.print_declaration(decl, level),
            BlockItem::S(stmt) => self.print_statement(stmt, level),
        }
    }

    /// Prints a declaration (variable or function)
    fn print_declaration(&self, decl: Declaration, level: usize) {
        match decl {
            Declaration::VarDecl(v) => self.print_variable_decl(v, level),
            Declaration::FunDecl(f) => self.print_function(f, level),
        }
    }

    /// Prints a variable declaration and its initializer (if present)
    fn print_variable_decl(&self, decl: VariableDecl, level: usize) {
        let (name, init, storage_class, _) = decl.into_parts();
        println!(
            "{}VariableDecl {:?} \"{}\"",
            self.indent(level),
            storage_class,
            self.format_spanned_identifier(name)
        );
        if let Some(expr) = init {
            println!("{}Init", self.indent(level + 1));
            self.print_expression(expr, level + 2);
        }
    }

    /// Prints a statement (various types handled)
    fn print_statement(&self, stmt: Statement, level: usize) {
        let (stmt_type, _) = stmt.into_parts();
        match stmt_type {
            StatementType::Return(expr) => self.print_return_stmt(expr, level),
            StatementType::ExprStatement(expr) => self.print_expr_stmt(expr, level),
            StatementType::Null => self.print_null_stmt(level),
            StatementType::Compound(block) => self.print_block(block, level),
            StatementType::IfStatement {
                condition,
                if_clause,
                else_clause,
            } => self.print_if_stmt(condition, *if_clause, else_clause, level),
            StatementType::Break(label) => self.print_break_stmt(label, level),
            StatementType::Continue(label) => self.print_continue_stmt(label, level),
            StatementType::While {
                condition,
                body,
                label,
            } => self.print_while_stmt(condition, *body, label, level),
            StatementType::DoWhile {
                condition,
                body,
                label,
            } => self.print_do_while_stmt(condition, *body, label, level),
            StatementType::For {
                init,
                condition,
                post,
                body,
                label,
            } => self.print_for_stmt(init, condition, post, *body, label, level),
        }
    }

    /// Prints a `return` statement
    fn print_return_stmt(&self, expr: Expression, level: usize) {
        println!("{}Return", self.indent(level));
        self.print_expression(expr, level + 1);
    }

    /// Prints an expression statement
    fn print_expr_stmt(&self, expr: Expression, level: usize) {
        println!("{}ExprStatement", self.indent(level));
        self.print_expression(expr, level + 1);
    }

    /// Prints a null statement (no-op)
    fn print_null_stmt(&self, level: usize) {
        println!("{}NullStatement", self.indent(level));
    }

    /// Prints an `if` statement with optional `else`
    fn print_if_stmt(
        &self,
        condition: Expression,
        if_clause: Statement,
        else_clause: Option<Box<Statement>>,
        level: usize,
    ) {
        println!("{}If", self.indent(level));
        println!("{}Condition", self.indent(level + 1));
        self.print_expression(condition, level + 2);
        println!("{}Then", self.indent(level + 1));
        self.print_statement(if_clause, level + 2);
        if let Some(else_stmt) = else_clause {
            println!("{}Else", self.indent(level + 1));
            self.print_statement(*else_stmt, level + 2);
        }
    }

    /// Prints a `break` statement with label
    fn print_break_stmt(&self, label: Identifier, level: usize) {
        println!(
            "{}Break \"{}\"",
            self.indent(level),
            self.format_identifier(label)
        );
    }

    /// Prints a `continue` statement with label
    fn print_continue_stmt(&self, label: Identifier, level: usize) {
        println!(
            "{}Continue \"{}\"",
            self.indent(level),
            self.format_identifier(label)
        );
    }

    /// Prints a `while` loop
    fn print_while_stmt(&self, cond: Expression, body: Statement, label: Identifier, level: usize) {
        println!(
            "{}While: {}",
            self.indent(level),
            self.format_identifier(label)
        );
        println!("{}Condition", self.indent(level + 1));
        self.print_expression(cond, level + 2);
        println!("{}Body", self.indent(level + 1));
        self.print_statement(body, level + 2);
    }

    /// Prints a `do-while` loop
    fn print_do_while_stmt(
        &self,
        cond: Expression,
        body: Statement,
        label: Identifier,
        level: usize,
    ) {
        println!(
            "{}DoWhile: {}",
            self.indent(level),
            self.format_identifier(label)
        );
        println!("{}Body", self.indent(level + 1));
        self.print_statement(body, level + 2);
        println!("{}Condition", self.indent(level + 1));
        self.print_expression(cond, level + 2);
    }

    /// Prints a `for` loop including init, condition, post, and body
    fn print_for_stmt(
        &self,
        init: ForInit,
        condition: Option<Expression>,
        post: Option<Expression>,
        body: Statement,
        label: Identifier,
        level: usize,
    ) {
        println!(
            "{}For: {}",
            self.indent(level),
            self.format_identifier(label)
        );

        // Print initializer
        println!("{}Init", self.indent(level + 1));
        match init {
            ForInit::D(decl) => self.print_variable_decl(decl, level + 2),
            ForInit::E(Some(expr)) => self.print_expression(expr, level + 2),
            ForInit::E(None) => println!("{}None", self.indent(level + 2)),
        }

        // Print condition
        println!("{}Condition", self.indent(level + 1));
        if let Some(expr) = condition {
            self.print_expression(expr, level + 2);
        } else {
            println!("{}None", self.indent(level + 2));
        }

        // Print post-expression
        println!("{}Post", self.indent(level + 1));
        if let Some(expr) = post {
            self.print_expression(expr, level + 2);
        } else {
            println!("{}None", self.indent(level + 2));
        }

        // Print body
        println!("{}Body", self.indent(level + 1));
        self.print_statement(body, level + 2);
    }

    /// Prints an expression
    fn print_expression(&self, expr: Expression, level: usize) {
        let (expr_type, _) = expr.into_parts();
        match expr_type {
            ExpressionType::Constant(n) => self.print_constant_expr(n, level),
            ExpressionType::Unary { operator, operand } => {
                self.print_unary_expr(operator, *operand, level)
            }
            ExpressionType::Binary {
                operator,
                operand1,
                operand2,
            } => self.print_binary_expr(operator, *operand1, *operand2, level),
            ExpressionType::Var(id) => self.print_var_expr(id, level),
            ExpressionType::Assignment { lvalue, rvalue } => {
                self.print_assignment_expr(*lvalue, *rvalue, level)
            }
            ExpressionType::Conditional { cond, cons, alt } => {
                self.print_conditional_expr(*cond, *cons, *alt, level)
            }
            ExpressionType::FunctionCall { name, args } => self.print_call_expr(name, args, level),
        }
    }

    /// Prints a constant value
    fn print_constant_expr(&self, value: i32, level: usize) {
        println!("{}Const {}", self.indent(level), value);
    }

    /// Prints a unary operation
    fn print_unary_expr(&self, op: UnaryOP, operand: Expression, level: usize) {
        println!("{}UnaryOp \"{:?}\"", self.indent(level), op);
        self.print_expression(operand, level + 1);
    }

    /// Prints a binary operation
    fn print_binary_expr(&self, op: BinaryOP, left: Expression, right: Expression, level: usize) {
        println!("{}BinaryOp \"{:?}\"", self.indent(level), op);
        self.print_expression(left, level + 1);
        self.print_expression(right, level + 1);
    }

    /// Prints a variable reference
    fn print_var_expr(&self, id: SpannedIdentifier, level: usize) {
        println!(
            "{}Var \"{}\"",
            self.indent(level),
            self.format_spanned_identifier(id)
        );
    }

    /// Prints an assignment expression
    fn print_assignment_expr(&self, lvalue: Expression, rvalue: Expression, level: usize) {
        println!("{}Assignment", self.indent(level));
        println!("{}LValue", self.indent(level + 1));
        self.print_expression(lvalue, level + 2);
        println!("{}RValue", self.indent(level + 1));
        self.print_expression(rvalue, level + 2);
    }

    /// Prints a ternary conditional expression
    fn print_conditional_expr(
        &self,
        cond: Expression,
        cons: Expression,
        alt: Expression,
        level: usize,
    ) {
        println!("{}Conditional", self.indent(level));
        println!("{}Condition", self.indent(level + 1));
        self.print_expression(cond, level + 2);
        println!("{}Then", self.indent(level + 1));
        self.print_expression(cons, level + 2);
        println!("{}Else", self.indent(level + 1));
        self.print_expression(alt, level + 2);
    }

    /// Prints a function call with arguments
    fn print_call_expr(&self, name: SpannedIdentifier, args: Vec<Box<Expression>>, level: usize) {
        println!(
            "{}FunctionCall \"{}\"",
            self.indent(level),
            self.format_spanned_identifier(name)
        );
        for (i, arg) in args.into_iter().enumerate() {
            println!("{}Arg {}", self.indent(level + 1), i);
            self.print_expression(*arg, level + 2);
        }
    }

    /// Formats a simple identifier as "name.id"
    fn format_identifier(&self, identifier: Identifier) -> String {
        let (symbol, id) = identifier.into_parts();
        format!("{}.{}", self.interner.lookup(symbol), id)
    }

    /// Formats a spanned identifier by delegating to format_identifier
    fn format_spanned_identifier(&self, identifier: SpannedIdentifier) -> String {
        let (ident, _) = identifier.into_parts();
        self.format_identifier(ident)
    }
}
