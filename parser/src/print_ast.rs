use crate::ast::*;
use shared_context::Identifier;
use shared_context::interner::Interner;

pub struct DebugTreePrinter<'a> {
    interner: &'a Interner<'a>,
}

impl<'a> DebugTreePrinter<'a> {
    pub fn new(interner: &'a Interner) -> Self {
        Self { interner }
    }

    pub fn print(&self, program: Program) {
        let function = program.into_parts();
        self.print_function(function, 0);
    }

    fn indent(&self, level: usize) -> String {
        "  ".repeat(level)
    }

    fn print_function(&self, function: FunctionDef, level: usize) {
        let (name, block, _) = function.into_parts();
        let name_str = self.format_spanned_identifier(name);
        println!("{}FunctionDef \"{}\"", self.indent(level), name_str);
        self.print_block(block, level + 1);
    }

    fn print_block(&self, block: Block, level: usize) {
        let (block_items, _) = block.into_parts();
        println!("{}Block", self.indent(level));
        for item in block_items {
            self.print_block_item(item, level + 1);
        }
    }

    fn print_block_item(&self, block_item: BlockItem, level: usize) {
        match block_item {
            BlockItem::D(decl) => self.print_declaration(decl, level),
            BlockItem::S(stmt) => self.print_statement(stmt, level),
        }
    }

    fn print_declaration(&self, decl: Declaration, level: usize) {
        let (name, init, _) = decl.into_parts();
        let name_str = self.format_spanned_identifier(name);
        println!("{}Declaration \"{}\"", self.indent(level), name_str);
        if let Some(expr) = init {
            println!("{}Init", self.indent(level + 1));
            self.print_expression(expr, level + 2);
        }
    }

    fn print_statement(&self, stmt: Statement, level: usize) {
        let (stmt_type, _) = stmt.into_parts();
        match stmt_type {
            StatementType::Return(expr) => self.print_return(expr, level),
            StatementType::ExprStatement(expr) => self.print_expr_statement(expr, level),
            StatementType::Null => self.print_null_statement(level),
            StatementType::Compound(block) => self.print_block(block, level),
            StatementType::IfStatement {
                condition,
                if_clause,
                else_clause,
            } => self.print_if(condition, *if_clause, else_clause, level),
            StatementType::Break(label) => self.print_break(label, level),
            StatementType::Continue(label) => self.print_continue(label, level),
            StatementType::While {
                condition,
                body,
                label,
            } => self.print_while(condition, *body, label, level),
            StatementType::DoWhile {
                condition,
                body,
                label,
            } => self.print_do_while(condition, *body, label, level),
            StatementType::For {
                init,
                condition,
                post,
                body,
                label,
            } => self.print_for(init, condition, post, *body, label, level),
        }
    }

    fn print_return(&self, expr: Expression, level: usize) {
        println!("{}Return", self.indent(level));
        self.print_expression(expr, level + 1);
    }

    fn print_expr_statement(&self, expr: Expression, level: usize) {
        println!("{}ExprStatement", self.indent(level));
        self.print_expression(expr, level + 1);
    }

    fn print_null_statement(&self, level: usize) {
        println!("{}NullStatement", self.indent(level));
    }

    fn print_if(
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

    fn print_break(&self, label: Identifier, level: usize) {
        let label_str = self.format_identifier(label);
        println!("{}Break \"{}\"", self.indent(level), label_str);
    }

    fn print_continue(&self, label: Identifier, level: usize) {
        let label_str = self.format_identifier(label);
        println!("{}Continue \"{}\"", self.indent(level), label_str);
    }

    fn print_while(&self, condition: Expression, body: Statement, label: Identifier, level: usize) {
        println!(
            "{}While: {}",
            self.indent(level),
            self.format_identifier(label)
        );
        println!("{}Condition", self.indent(level + 1));
        self.print_expression(condition, level + 2);
        println!("{}Body", self.indent(level + 1));
        self.print_statement(body, level + 2);
    }

    fn print_do_while(
        &self,
        condition: Expression,
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
        self.print_expression(condition, level + 2);
    }

    fn print_for(
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

        println!("{}Init", self.indent(level + 1));
        match init {
            ForInit::D(decl) => self.print_declaration(decl, level + 2),
            ForInit::E(Some(expr)) => self.print_expression(expr, level + 2),
            ForInit::E(None) => println!("{}None", self.indent(level + 2)),
        }

        println!("{}Condition", self.indent(level + 1));
        if let Some(cond) = condition {
            self.print_expression(cond, level + 2);
        } else {
            println!("{}None", self.indent(level + 2));
        }

        println!("{}Post", self.indent(level + 1));
        if let Some(post_expr) = post {
            self.print_expression(post_expr, level + 2);
        } else {
            println!("{}None", self.indent(level + 2));
        }

        println!("{}Body", self.indent(level + 1));
        self.print_statement(body, level + 2);
    }

    fn print_expression(&self, expr: Expression, level: usize) {
        let (expr_type, _) = expr.into_parts();

        match expr_type {
            ExpressionType::Constant(n) => {
                println!("{}Const {}", self.indent(level), n);
            }
            ExpressionType::Unary { operator, operand } => {
                println!("{}UnaryOp \"{:?}\"", self.indent(level), operator);
                self.print_expression(*operand, level + 1);
            }
            ExpressionType::Binary {
                operator,
                operand1,
                operand2,
            } => {
                println!("{}BinaryOp \"{:?}\"", self.indent(level), operator);
                self.print_expression(*operand1, level + 1);
                self.print_expression(*operand2, level + 1);
            }
            ExpressionType::Var(id) => {
                println!(
                    "{}Var \"{}\"",
                    self.indent(level),
                    self.format_spanned_identifier(id)
                );
            }
            ExpressionType::Assignment { lvalue, rvalue } => {
                println!("{}Assignment", self.indent(level));
                println!("{}LValue", self.indent(level + 1));
                self.print_expression(*lvalue, level + 2);
                println!("{}RValue", self.indent(level + 1));
                self.print_expression(*rvalue, level + 2);
            }
            ExpressionType::Conditional { cond, cons, alt } => {
                println!("{}Conditional", self.indent(level));
                println!("{}Condition", self.indent(level + 1));
                self.print_expression(*cond, level + 2);
                println!("{}Then", self.indent(level + 1));
                self.print_expression(*cons, level + 2);
                println!("{}Else", self.indent(level + 1));
                self.print_expression(*alt, level + 2);
            }
        }
    }

    fn format_identifier(&self, identifier: Identifier) -> String {
        let (symbol, id) = identifier.into_parts();
        format!("{}.{}", self.interner.lookup(symbol), id)
    }

    fn format_spanned_identifier(&self, identifier: SpannedIdentifier) -> String {
        let (identifier, _) = identifier.into_parts();
        let (symbol, id) = identifier.into_parts();
        format!("{}.{}", self.interner.lookup(symbol), id)
    }
}
