use shared_context::Identifier;
use shared_context::interner::Interner;

use crate::ast::*;

pub struct DebuggingPrinter<'a> {
    interner: &'a Interner<'a>,
}

impl<'a> DebuggingPrinter<'a> {
    pub fn new(interner: &'a Interner) -> Self {
        Self { interner }
    }

    pub fn print(&self, program: Program) {
        println!("Program");
        let indent = 2;
        self.print_program(program, indent);
    }

    fn print_program(&self, program: Program, indent_level: usize) {
        let function = program.into_parts();
        self.print_function(function, indent_level);
    }

    fn print_function(&self, function: FunctionDef, indent_level: usize) {
        let indent = " ".repeat(indent_level);

        let (name, block, _) = function.into_parts();
        println!("{}Def {}", indent, self.interner.lookup(name.get_symbol()));
        self.print_block(block, indent_level + 2);
    }

    fn print_block(&self, block: Block, indent_level: usize) {
        let indent = " ".repeat(indent_level);
        let (block_items, _) = block.into_parts();

        println!("{}block(", indent);

        for item in block_items {
            self.print_block_item(item, indent_level + 2);
        }

        println!("{})", indent);
    }

    fn print_block_item(&self, block_item: BlockItem, indent_level: usize) {
        match block_item {
            BlockItem::D(sp_decl) => self.print_declaration(sp_decl, indent_level),
            BlockItem::S(sp_stmt) => self.print_statement(sp_stmt, indent_level),
        }
    }

    fn print_declaration(&self, decl: Declaration, indent_level: usize) {
        let indent = " ".repeat(indent_level);
        let (name, init, _) = decl.into_parts();
        println!("{}Declare({})", indent, self.format_identifier(name));

        if let Some(expr) = init {
            self.print_expr(expr, indent_level + 2);
        }
    }

    fn print_statement(&self, stmt: Statement, indent_level: usize) {
        let indent = " ".repeat(indent_level);
        let (stmt_type, _) = stmt.into_parts();

        match stmt_type {
            StatementType::Return(expr) => {
                println!("{}Return", indent);
                self.print_expr(expr, indent_level + 2);
            }
            StatementType::ExprStatement(expr) => {
                self.print_expr(expr, indent_level);
            }
            StatementType::Null => {
                println!("{}Null", indent);
            }
            StatementType::Compound(block) => self.print_block(block, indent_level),
            StatementType::IfStatement {
                condition,
                if_clause,
                else_clause,
            } => {
                println!("{}If", indent);
                self.print_if_statement(condition, *if_clause, else_clause, indent_level + 2);
            }
        }
    }

    fn print_if_statement(
        &self,
        condition: Expression,
        if_clause: Statement,
        else_clause: Option<Box<Statement>>,
        indent_level: usize,
    ) {
        let indent = " ".repeat(indent_level);

        println!("{}Condtion", indent);
        self.print_expr(condition, indent_level + 2);

        println!("{}if_clause", indent);
        self.print_statement(if_clause, indent_level + 2);

        if let Some(clause) = else_clause {
            println!("{}else_clause", indent);
            self.print_statement(*clause, indent_level + 2);
        }
    }

    fn print_expr(&self, expr: Expression, indent_level: usize) {
        let indent = " ".repeat(indent_level);
        let (expr_type, _) = expr.into_parts();

        match expr_type {
            ExpressionType::Constant(n) => println!("{}Constant({})", indent, n),

            ExpressionType::Unary { operator, operand } => {
                println!("{}Unary({:?})", indent, operator);
                self.print_expr(*operand, indent_level + 2);
            }

            ExpressionType::Binary {
                operator,
                operand1,
                operand2,
            } => {
                println!("{}Binary({:?})", indent, operator);
                self.print_expr(*operand1, indent_level + 2);
                self.print_expr(*operand2, indent_level + 2);
            }

            ExpressionType::Var(id) => {
                println!("{}Var({})", indent, self.format_identifier(id));
            }

            ExpressionType::Assignment { lvalue, rvalue } => {
                println!("{}Assign", indent);
                self.print_expr(*lvalue, indent_level + 2);
                self.print_expr(*rvalue, indent_level + 2);
            }
            ExpressionType::Conditional { cond, cons, alt } => {
                println!("{}Condtional", indent);
                self.print_conditional(*cond, *cons, *alt, indent_level + 2);
            }
        }
    }

    fn print_conditional(
        &self,
        cond: Expression,
        cons: Expression,
        alt: Expression,
        indent_level: usize,
    ) {
        let indent = " ".repeat(indent_level);

        println!("{}Condition", indent);
        self.print_expr(cond, indent_level + 2);

        println!("{}Consequence", indent);
        self.print_expr(cons, indent_level + 2);

        println!("{}Alternative", indent);
        self.print_expr(alt, indent_level + 2);
    }

    fn format_identifier(&self, identifier: Identifier) -> String {
        let (symbol, id, _) = identifier.into_parts();

        format!("{}.{}", self.interner.lookup(symbol), id)
    }
}
