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
}

//
// Function impl
//

impl FunctionDef {
    pub fn new(name: Identifier, body: Statement) -> Self {
        Self { name, body }
    }

    pub fn into_parts(self) -> (Identifier, Statement) {
        (self.name, self.body)
    }
}
