use crate::{Parser, ast};

// a pretty printer for debugging the AST

impl<'source> Parser<'source> {
    pub fn print(program: ast::Program) {
        println!("Program");

        let indent = 2;
        let function = program.into_parts();
        Parser::print_function(function, indent);
    }

    fn print_function(function: ast::FunctionDef, indent: usize) {
        let (name, body) = function.into_parts();

        println!("{}Def {}", " ".repeat(indent), name.0);
        Parser::print_statement(body, indent + 2);
    }

    fn print_statement(statement: ast::Statement, indent: usize) {
        match statement {
            ast::Statement::Return(expr) => {
                println!("{}Return", " ".repeat(indent));
                Parser::print_expr(expr, indent + 2);
            }
        }
    }

    fn print_expr(expr: ast::Expression, indent: usize) {
        match expr {
            ast::Expression::Constant(int) => println!("{}Constant({})", " ".repeat(indent), int),
            ast::Expression::Unary { operator, operand } => {
                Parser::print_unary_expr(operator, *operand, indent);
            }
            ast::Expression::Binary {
                operator,
                operand1,
                operand2,
            } => {
                Parser::print_binary_expr(operator, *operand1, *operand2, indent);
            }
        }
    }

    fn print_unary_expr(operator: ast::UnaryOP, operand: ast::Expression, indent: usize) {
        println!("{}Unary({:?})", " ".repeat(indent), operator);
        Parser::print_expr(operand, indent + 2);
    }

    fn print_binary_expr(
        operator: ast::BinaryOP,
        operand1: ast::Expression,
        operand2: ast::Expression,
        indent: usize,
    ) {
        println!("{}Binary({:?})", " ".repeat(indent), operator);
        Parser::print_expr(operand1, indent + 2);
        Parser::print_expr(operand2, indent + 2);
    }
}
