// Identifier is just a wrapper for a string (function name)
pub struct Identifier(pub String);

// The whole program, holds one function definition
pub struct Program {
    pub function: FunctionDefinition,
}

// A function definition: name + body
pub struct FunctionDefinition {
    pub name: Identifier,
    pub body: Statement,
}

// Statements in the language
pub enum Statement {
    Return(Expression),
}

// Expressions in the language
pub enum Expression {
    Constant(i32),
}

// implement pretty printing for the AST
impl Program {
    pub fn dump(&self, indent: usize) {
        let pad = " ".repeat(indent);
        println!("{pad}Program(");
        self.function.dump(indent + 4);
        println!("{pad})");
    }
}

impl FunctionDefinition {
    pub fn dump(&self, indent: usize) {
        let pad = " ".repeat(indent);
        println!("{pad}Function(");
        println!("{pad}    name=\"{}\",", self.name.0);
        println!("{pad}    body=");
        self.body.dump(indent + 8);
        println!("{pad})");
    }
}

impl Statement {
    pub fn dump(&self, indent: usize) {
        let pad = " ".repeat(indent);
        match self {
            Statement::Return(expr) => {
                println!("{pad}Return(");
                expr.dump(indent + 4);
                println!("{pad})");
            }
        }
    }
}

impl Expression {
    pub fn dump(&self, indent: usize) {
        let pad = " ".repeat(indent);
        match self {
            Expression::Constant(val) => {
                println!("{pad}Constant({})", val);
            }
        }
    }
}

