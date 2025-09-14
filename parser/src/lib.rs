use lexer::Lexer;

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
