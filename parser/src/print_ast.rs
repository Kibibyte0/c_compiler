use crate::ast::*;

pub struct DebuggingPrinter;

impl DebuggingPrinter {
    pub fn print(program: Spanned<Program>) {
        println!("Program");
        let indent = 2;
        Self::print_program(program, indent);
    }

    fn print_program(sp_program: Spanned<Program>, indent_level: usize) {
        let program = sp_program.discard_sp();
        let function = program.into_parts();
        Self::print_function(function, indent_level);
    }

    fn print_function(sp_function: Spanned<FunctionDef>, indent_level: usize) {
        let indent = " ".repeat(indent_level);
        let function = sp_function.discard_sp();

        let (sp_name, body) = function.into_parts();
        println!("{}Def {}", indent, sp_name.get_node_ref().get_name_ref());

        for sp_block_item in body {
            Self::print_block_item(sp_block_item, indent_level + 2);
        }
    }

    fn print_block_item(sp_block_item: Spanned<BlockItem>, indent_level: usize) {
        let block_item = sp_block_item.discard_sp();
        match block_item {
            BlockItem::D(sp_decl) => Self::print_declaration(sp_decl, indent_level),
            BlockItem::S(sp_stmt) => Self::print_statement(sp_stmt, indent_level),
        }
    }

    fn print_declaration(sp_decl: Spanned<Declaration>, indent_level: usize) {
        let indent = " ".repeat(indent_level);
        let decl = sp_decl.discard_sp();
        let (sp_name, sp_init) = decl.into_parts();
        println!(
            "{}Declare({})",
            indent,
            sp_name.get_node_ref().get_name_ref()
        );

        if let Some(sp_expr) = sp_init {
            Self::print_expr(sp_expr, indent_level + 2);
        }
    }

    fn print_statement(sp_stmt: Spanned<Statement>, indent_level: usize) {
        let indent = " ".repeat(indent_level);
        let stmt = sp_stmt.discard_sp();

        match stmt {
            Statement::Return(sp_expr) => {
                println!("{}Return", indent);
                Self::print_expr(sp_expr, indent_level + 2);
            }
            Statement::ExprStatement(sp_expr) => {
                Self::print_expr(sp_expr, indent_level);
            }
            Statement::Null => {
                println!("{}Null", indent);
            }
            Statement::IfStatement {
                condition,
                if_clause,
                else_clause,
            } => {
                println!("{}If", indent);
                Self::print_if_statement(condition, *if_clause, else_clause, indent_level + 2);
            }
        }
    }

    fn print_if_statement(
        condition: Spanned<Expression>,
        if_clause: Spanned<Statement>,
        else_clause: Option<Box<Spanned<Statement>>>,
        indent_level: usize,
    ) {
        let indent = " ".repeat(indent_level);

        println!("{}Condtion", indent);
        Self::print_expr(condition, indent_level + 2);

        println!("{}if_clause", indent);
        Self::print_statement(if_clause, indent_level + 2);

        if let Some(clause) = else_clause {
            println!("{}else_clause", indent);
            Self::print_statement(*clause, indent_level + 2);
        }
    }

    fn print_expr(sp_expr: Spanned<Expression>, indent_level: usize) {
        let indent = " ".repeat(indent_level);
        let expr = sp_expr.discard_sp();

        match expr {
            Expression::Constant(n) => println!("{}Constant({})", indent, n),

            Expression::Unary { operator, operand } => {
                println!("{}Unary({:?})", indent, operator);
                Self::print_expr(*operand, indent_level + 2);
            }

            Expression::Binary {
                operator,
                operand1,
                operand2,
            } => {
                println!("{}Binary({:?})", indent, operator);
                Self::print_expr(*operand1, indent_level + 2);
                Self::print_expr(*operand2, indent_level + 2);
            }

            Expression::Var(sp_ident) => {
                println!("{}Var({})", indent, sp_ident.get_node_ref().get_name_ref());
            }

            Expression::Assignment { lvalue, rvalue } => {
                println!("{}Assign", indent);
                Self::print_expr(*lvalue, indent_level + 2);
                Self::print_expr(*rvalue, indent_level + 2);
            }
            Expression::Conditional { cond, cons, alt } => {
                println!("{}Condtional", indent);
                Self::print_conditional(*cond, *cons, *alt, indent_level + 2);
            }
        }
    }

    fn print_conditional(
        cond: Spanned<Expression>,
        cons: Spanned<Expression>,
        alt: Spanned<Expression>,
        indent_level: usize,
    ) {
        let indent = " ".repeat(indent_level);

        println!("{}Condition", indent);
        Self::print_expr(cond, indent_level + 2);

        println!("{}Consequence", indent);
        Self::print_expr(cons, indent_level + 2);

        println!("{}Alternative", indent);
        Self::print_expr(alt, indent_level + 2);
    }
}
