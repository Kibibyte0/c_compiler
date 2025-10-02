mod expressions;
pub use expressions::{BinaryOP, Expression, UnaryOP};

pub struct Program {
    function: FunctionDef,
}

pub struct FunctionDef {
    name: Identifier,
    body: Statement,
}

pub enum Statement {
    Return(Expression),
}

pub struct Identifier(pub String);

//
// Program impl
//

impl Program {
    pub fn new(function: FunctionDef) -> Self {
        Self { function }
    }

    pub fn into_parts(self) -> FunctionDef {
        self.function
    }

    pub fn print(&self) {
        self.print_with_indent(0);
    }

    fn print_with_indent(&self, indent: usize) {
        println!("{}Program", " ".repeat(indent));
        self.function.print_with_indent(indent + 2);
    }
}

//
// Function impl
//

impl Default for FunctionDef {
    fn default() -> Self {
        Self {
            name: Identifier("func".to_string()),
            body: Statement::Return(Expression::Constant(1)),
        }
    }
}

impl FunctionDef {
    pub fn new(name: Identifier, body: Statement) -> Self {
        Self { name, body }
    }

    pub fn into_parts(self) -> (Identifier, Statement) {
        (self.name, self.body)
    }

    fn print_with_indent(&self, indent: usize) {
        println!("{}FunctionDef", " ".repeat(indent));
        println!("{}name: {}", " ".repeat(indent + 2), self.name.0);
        self.body.print_with_indent(indent + 2);
    }
}

impl Statement {
    fn print_with_indent(&self, indent: usize) {
        match self {
            Statement::Return(expr) => {
                println!("{}Return", " ".repeat(indent));
                expr.print_with_indent(indent + 2);
            }
        }
    }
}
